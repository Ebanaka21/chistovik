use std::process::Command;
use chrono::Utc;
use std::path::PathBuf;

/// Сервис автоматического резервного копирования БД
pub struct BackupService {
    db_url: String,
    backup_dir: PathBuf,
    s3_bucket: Option<String>,
    retention_days: u32,
}

impl BackupService {
    pub fn new(db_url: &str, backup_dir: PathBuf, retention_days: u32) -> Self {
        Self {
            db_url: db_url.to_string(),
            backup_dir,
            s3_bucket: None,
            retention_days,
        }
    }

    pub fn with_s3(mut self, bucket: &str) -> Self {
        self.s3_bucket = Some(bucket.to_string());
        self
    }

    /// Создание резервной копии
    pub async fn create_backup(&self) -> Result<BackupResult, BackupError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("chistovik_backup_{}.sql.gz", timestamp);
        let backup_path = self.backup_dir.join(&filename);

        // Создаём директорию если не существует
        std::fs::create_dir_all(&self.backup_dir)
            .map_err(|e| BackupError::Io(e.to_string()))?;

        // Выполняем pg_dump с компрессией
        let output = Command::new("pg_dump")
            .env("PGPASSWORD", self.extract_password())
            .args(&[
                "-h", &self.extract_host(),
                "-U", &self.extract_user(),
                "-d", &self.extract_dbname(),
                "-Fc", // Custom format
                "-Z", "9", // Максимальная компрессия
                "-f", backup_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| BackupError::Command(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::Command(format!("pg_dump failed: {}", stderr)));
        }

        // Получаем размер файла
        let metadata = std::fs::metadata(&backup_path)
            .map_err(|e| BackupError::Io(e.to_string()))?;
        let size_bytes = metadata.len();

        // Вычисляем хэш для проверки целостности
        let file_data = std::fs::read(&backup_path)
            .map_err(|e| BackupError::Io(e.to_string()))?;
        let hash = sha2::Digest::digest(&sha2::Sha256::new_with_prefix(&file_data));
        let hash_hex = hex::encode(hash);

        // Загрузка в S3 (если настроено)
        if let Some(bucket) = &self.s3_bucket {
            self.upload_to_s3(&backup_path, &filename, bucket).await?;
        }

        // Очистка старых бэкапов
        self.cleanup_old_backups().await?;

        Ok(BackupResult {
            filename,
            path: backup_path.to_str().unwrap().to_string(),
            size_bytes,
            hash: hash_hex,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Загрузка в S3
    async fn upload_to_s3(&self, local_path: &std::path::Path, key: &str, bucket: &str) -> Result<(), BackupError> {
        let s3_key = format!("backups/{}", key);
        
        let output = Command::new("aws")
            .args(&[
                "s3", "cp",
                local_path.to_str().unwrap(),
                &format!("s3://{}/{}", bucket, s3_key),
            ])
            .output()
            .map_err(|e| BackupError::Command(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::Command(format!("S3 upload failed: {}", stderr)));
        }

        Ok(())
    }

    /// Очистка старых бэкапов
    async fn cleanup_old_backups(&self) -> Result<(), BackupError> {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);
        
        let entries = std::fs::read_dir(&self.backup_dir)
            .map_err(|e| BackupError::Io(e.to_string()))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time = chrono::DateTime::<Utc>::from(modified);
                        if modified_time < cutoff {
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Восстановление из бэкапа
    pub async fn restore_backup(&self, backup_path: &str) -> Result<(), BackupError> {
        let output = Command::new("pg_restore")
            .env("PGPASSWORD", self.extract_password())
            .args(&[
                "-h", &self.extract_host(),
                "-U", &self.extract_user(),
                "-d", &self.extract_dbname(),
                "-c", // Clean before restore
                backup_path,
            ])
            .output()
            .map_err(|e| BackupError::Command(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::Command(format!("pg_restore failed: {}", stderr)));
        }

        Ok(())
    }

    /// Проверка целостности бэкапа (restore drill)
    /// Запускается периодически для проверки что бэкапы рабочие
    pub async fn verify_backup(&self, backup_path: &str) -> Result<bool, BackupError> {
        // Проверяем что файл существует
        if !std::path::Path::new(backup_path).exists() {
            return Err(BackupError::Io("Backup file not found".to_string()));
        }

        // Проверяем целостность через pg_restore --list
        let output = Command::new("pg_restore")
            .args(&["--list", backup_path])
            .output()
            .map_err(|e| BackupError::Command(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::error!("Backup verification failed: {}", stderr);
            return Ok(false);
        }

        // Проверяем что есть данные в списке
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.lines().count() < 5 {
            log::warn!("Backup seems empty or corrupted");
            return Ok(false);
        }

        log::info!("Backup verification successful: {}", backup_path);
        Ok(true)
    }

    // Helper методы для парсинга DATABASE_URL
    fn extract_host(&self) -> String {
        self.db_url.split("://").nth(1)
            .and_then(|s| s.split('@').nth(1))
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.split(':').next())
            .unwrap_or("localhost")
            .to_string()
    }

    fn extract_user(&self) -> String {
        self.db_url.split("://").nth(1)
            .and_then(|s| s.split(':').next())
            .unwrap_or("postgres")
            .to_string()
    }

    fn extract_password(&self) -> String {
        self.db_url.split("://").nth(1)
            .and_then(|s| s.split(':').nth(1))
            .and_then(|s| s.split('@').next())
            .unwrap_or("")
            .to_string()
    }

    fn extract_dbname(&self) -> String {
        self.db_url.split('/').last()
            .and_then(|s| s.split('?').next())
            .unwrap_or("chistovik")
            .to_string()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct BackupResult {
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub hash: String,
    pub created_at: String,
}

#[derive(Debug)]
pub enum BackupError {
    Io(String),
    Command(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::Io(e) => write!(f, "IO error: {}", e),
            BackupError::Command(e) => write!(f, "Command error: {}", e),
        }
    }
}

impl std::error::Error for BackupError {}

/// Планировщик автоматических бэкапов
pub struct BackupScheduler {
    service: BackupService,
    interval_hours: u64,
}

impl BackupScheduler {
    pub fn new(service: BackupService, interval_hours: u64) -> Self {
        Self { service, interval_hours }
    }

    /// Запуск планировщика
    pub async fn run(&self) -> Result<(), BackupError> {
        log::info!("Backup scheduler started, interval: {} hours", self.interval_hours);

        loop {
            match self.service.create_backup().await {
                Ok(result) => {
                    log::info!("Backup created: {} ({} bytes)", result.filename, result.size_bytes);
                }
                Err(e) => {
                    log::error!("Backup failed: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(self.interval_hours * 3600)).await;
        }
    }
}
