use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use uuid::Uuid;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;

/// Multipart chunked upload — загрузка больших файлов без загрузки в RAM
/// Файл пишется напрямую на диск чанками

pub struct ChunkedUploadService {
    temp_dir: PathBuf,
    max_file_size: u64,
    chunk_size: usize,
}

impl ChunkedUploadService {
    pub fn new(temp_dir: PathBuf, max_file_size: u64) -> Self {
        Self {
            temp_dir,
            max_file_size,
            chunk_size: 1024 * 1024, // 1 MB chunks
        }
    }

    /// Начало загрузки — создание upload session
    pub async fn init_upload(
        &self,
        pool: &sqlx::PgPool,
        user_id: Uuid,
        filename: &str,
        total_size: u64,
        content_type: &str,
    ) -> Result<UploadSession, Box<dyn std::error::Error>> {
        // Проверка квоты пользователя
        let used_storage: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(file_size_bytes), 0) FROM contents WHERE author_id IN (SELECT id FROM authors WHERE user_id = $1)"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if (used_storage as u64) + total_size > self.max_file_size * 10 {
            return Err("Storage quota exceeded".into());
        }

        let upload_id = Uuid::new_v4();
        let temp_path = self.temp_dir.join(format!("upload_{}", upload_id));

        // Создаём директорию для чанков
        tokio::fs::create_dir_all(&temp_path).await?;

        // Сохраняем метаданные загрузки
        sqlx::query(
            r#"
            INSERT INTO upload_sessions (id, user_id, filename, total_size, content_type, temp_path, status, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW())
            "#
        )
        .bind(upload_id)
        .bind(user_id)
        .bind(filename)
        .bind(total_size as i64)
        .bind(content_type)
        .bind(temp_path.to_str())
        .execute(pool)
        .await?;

        Ok(UploadSession {
            upload_id,
            chunk_size: self.chunk_size as u64,
            total_chunks: ((total_size + self.chunk_size as u64 - 1) / self.chunk_size as u64) as i32,
        })
    }

    /// Загрузка одного чанка
    pub async fn upload_chunk(
        &self,
        pool: &sqlx::PgPool,
        upload_id: Uuid,
        chunk_index: i32,
        mut payload: web::Payload,
    ) -> Result<ChunkResult, Box<dyn std::error::Error>> {
        // Проверяем существование upload session
        let session: Option<(String, i64, i32)> = sqlx::query_as(
            "SELECT temp_path, total_size, total_chunks FROM upload_sessions WHERE id = $1 AND status = 'pending'"
        )
        .bind(upload_id)
        .fetch_optional(pool)
        .await?;

        let (temp_path_str, total_size, total_chunks) = session
            .ok_or("Upload session not found")?;

        let temp_path = PathBuf::from(temp_path_str);
        let chunk_path = temp_path.join(format!("chunk_{:06}", chunk_index));

        // Записываем чанк на диск (streaming, без загрузки в RAM)
        let mut file = tokio::fs::File::create(&chunk_path).await?;
        let mut bytes_written: u64 = 0;

        while let Some(chunk) = payload.next().await {
            let chunk = chunk?;
            bytes_written += chunk.len() as u64;

            if bytes_written > self.chunk_size as u64 * 2 {
                return Err("Chunk too large".into());
            }

            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        // Обновляем прогресс
        sqlx::query(
            "UPDATE upload_sessions SET uploaded_chunks = uploaded_chunks + 1, updated_at = NOW() WHERE id = $1"
        )
        .bind(upload_id)
        .execute(pool)
        .await?;

        // Проверяем, все ли чанки загружены
        let uploaded_chunks: i32 = sqlx::query_scalar(
            "SELECT uploaded_chunks FROM upload_sessions WHERE id = $1"
        )
        .bind(upload_id)
        .fetch_one(pool)
        .await?;

        let is_complete = uploaded_chunks >= total_chunks;

        Ok(ChunkResult {
            chunk_index,
            bytes_written,
            is_complete,
        })
    }

    /// Завершение загрузки — сборка файла из чанков
    pub async fn complete_upload(
        &self,
        pool: &sqlx::PgPool,
        upload_id: Uuid,
        user_id: Uuid,
    ) -> Result<CompletedUpload, Box<dyn std::error::Error>> {
        // Получаем session
        let session: Option<(String, String, i64, i32)> = sqlx::query_as(
            "SELECT temp_path, filename, total_size, total_chunks FROM upload_sessions WHERE id = $1 AND user_id = $2 AND status = 'pending'"
        )
        .bind(upload_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let (temp_path_str, filename, total_size, total_chunks) = session
            .ok_or("Upload session not found")?;

        let temp_path = PathBuf::from(temp_path_str);

        // Проверяем, все ли чанки на месте
        let mut missing_chunks = Vec::new();
        for i in 0..total_chunks {
            let chunk_path = temp_path.join(format!("chunk_{:06}", i));
            if !chunk_path.exists() {
                missing_chunks.push(i);
            }
        }

        if !missing_chunks.is_empty() {
            return Err(format!("Missing chunks: {:?}", missing_chunks).into());
        }

        // Собираем файл из чанков
        let final_path = temp_path.join(&filename);
        let mut output = tokio::fs::File::create(&final_path).await?;

        for i in 0..total_chunks {
            let chunk_path = temp_path.join(format!("chunk_{:06}", i));
            let chunk_data = tokio::fs::read(&chunk_path).await?;
            output.write_all(&chunk_data).await?;
        }

        output.flush().await?;

        // Вычисляем хэш файла
        let file_data = tokio::fs::read(&final_path).await?;
        let file_hash = sha2::Digest::digest(&sha2::Sha256::new_with_prefix(&file_data));
        let hash_hex = hex::encode(file_hash);

        // Обновляем session
        sqlx::query(
            "UPDATE upload_sessions SET status = 'completed', file_path = $1, file_hash = $2, completed_at = NOW() WHERE id = $3"
        )
        .bind(final_path.to_str())
        .bind(&hash_hex)
        .bind(upload_id)
        .execute(pool)
        .await?;

        // Удаляем чанки
        for i in 0..total_chunks {
            let chunk_path = temp_path.join(format!("chunk_{:06}", i));
            let _ = tokio::fs::remove_file(chunk_path).await;
        }

        Ok(CompletedUpload {
            upload_id,
            file_path: final_path.to_str().unwrap().to_string(),
            file_size: total_size as u64,
            file_hash: hash_hex,
        })
    }

    /// Отмена загрузки
    pub async fn cancel_upload(
        &self,
        pool: &sqlx::PgPool,
        upload_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let session: Option<(String,)> = sqlx::query_as(
            "SELECT temp_path FROM upload_sessions WHERE id = $1 AND user_id = $2 AND status = 'pending'"
        )
        .bind(upload_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if let Some((temp_path_str,)) = session {
            let temp_path = PathBuf::from(temp_path_str);
            let _ = tokio::fs::remove_dir_all(temp_path).await;
        }

        sqlx::query(
            "UPDATE upload_sessions SET status = 'cancelled' WHERE id = $1"
        )
        .bind(upload_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct UploadSession {
    pub upload_id: Uuid,
    pub chunk_size: u64,
    pub total_chunks: i32,
}

#[derive(serde::Serialize)]
pub struct ChunkResult {
    pub chunk_index: i32,
    pub bytes_written: u64,
    pub is_complete: bool,
}

#[derive(serde::Serialize)]
pub struct CompletedUpload {
    pub upload_id: Uuid,
    pub file_path: String,
    pub file_size: u64,
    pub file_hash: String,
}

/// Миграция для upload_sessions
pub fn create_upload_sessions_table() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS upload_sessions (
        id UUID PRIMARY KEY,
        user_id UUID NOT NULL REFERENCES users(id),
        filename VARCHAR(255) NOT NULL,
        total_size BIGINT NOT NULL,
        content_type VARCHAR(100) NOT NULL,
        temp_path TEXT NOT NULL,
        file_path TEXT,
        file_hash VARCHAR(64),
        status VARCHAR(20) NOT NULL DEFAULT 'pending',
        total_chunks INTEGER NOT NULL DEFAULT 0,
        uploaded_chunks INTEGER NOT NULL DEFAULT 0,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        completed_at TIMESTAMPTZ
    );

    CREATE INDEX IF NOT EXISTS idx_upload_sessions_user ON upload_sessions(user_id);
    CREATE INDEX IF NOT EXISTS idx_upload_sessions_status ON upload_sessions(status);
    "#
}
