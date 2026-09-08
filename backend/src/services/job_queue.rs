use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use std::time::Duration;
use tokio::sync::mpsc;

/// Тип задачи для обработки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    TranscodeVideo {
        input_path: String,
        output_dir: String,
        qualities: Vec<String>, // ["360p", "720p", "1080p"]
    },
    TranscodeAudio {
        input_path: String,
        output_dir: String,
        format: String, // "hls", "aac"
    },
    CreateTeaser {
        input_path: String,
        output_path: String,
        start_seconds: i32,
        duration_seconds: i32,
        media_type: String, // "audio" или "video"
    },
    GenerateThumbnail {
        input_path: String,
        output_path: String,
        timestamp_seconds: f32,
    },
    ApplyForensicWatermark {
        input_path: String,
        output_path: String,
        user_id: String,
    },
    ExtractMetadata {
        input_path: String,
        content_id: Uuid,
    },
}

/// Статус задачи
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Задача в очереди
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: i32, // 0 = highest, 10 = lowest
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub result: Option<serde_json::Value>,
}

/// Очередь задач на базе Redis
pub struct JobQueue {
    redis: redis::Client,
    queue_name: String,
    processing_queue: String,
    workers_count: usize,
}

impl JobQueue {
    pub fn new(redis_url: &str, queue_name: &str, workers_count: usize) -> Result<Self, redis::RedisError> {
        let redis = redis::Client::open(redis_url)?;
        
        Ok(Self {
            redis,
            queue_name: queue_name.to_string(),
            processing_queue: format!("{}:processing", queue_name),
            workers_count,
        })
    }

    /// Добавление задачи в очередь
    pub async fn enqueue(&self, job_type: JobType, priority: i32) -> Result<String, Box<dyn std::error::Error>> {
        let job = Job {
            id: Uuid::new_v4().to_string(),
            job_type,
            status: JobStatus::Pending,
            priority,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
            result: None,
        };

        let job_json = serde_json::to_string(&job)?;
        
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        // Используем sorted set для приоритетов
        let _: () = conn.zadd(&self.queue_name, &job_json, priority as f64).await?;
        
        log::info!("Job enqueued: {} with priority {}", job.id, priority);
        
        Ok(job.id)
    }

    /// Получение следующей задачи из очереди
    pub async fn dequeue(&self) -> Result<Option<Job>, Box<dyn std::error::Error>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        // Атомарно получаем и перемещаем в processing queue
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT 1" // Placeholder — в реальности используем Redis Lua script
        )
        .fetch_optional(&conn)
        .await
        .ok();

        // Используем ZPOPMIN для получения задачи с наивысшим приоритетом
        let job_json: Option<String> = conn.zpopmin(&self.queue_name, 1).await?
            .into_iter()
            .next()
            .map(|(s, _)| s);

        if let Some(json) = job_json {
            let mut job: Job = serde_json::from_str(&json)?;
            job.status = JobStatus::Processing;
            job.started_at = Some(Utc::now());

            // Сохраняем в processing queue
            let updated_json = serde_json::to_string(&job)?;
            let _: () = conn.zadd(&self.processing_queue, &updated_json, Utc::now().timestamp() as f64).await?;

            return Ok(Some(job));
        }

        Ok(None)
    }

    /// Завершение задачи
    pub async fn complete(&self, job_id: &str, result: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        // Находим задачу в processing queue
        let jobs: Vec<String> = conn.zrangebyscore(&self.processing_queue, "-inf", "+inf").await?;
        
        for job_json in jobs {
            let mut job: Job = serde_json::from_str(&job_json)?;
            if job.id == job_id {
                job.status = JobStatus::Completed;
                job.completed_at = Some(Utc::now());
                job.result = Some(result);

                let updated_json = serde_json::to_string(&job)?;
                
                // Удаляем из processing
                let _: () = conn.zrem(&self.processing_queue, &job_json).await?;
                
                // Сохраняем в completed queue (для истории)
                let _: () = conn.zadd("jobs:completed", &updated_json, Utc::now().timestamp() as f64).await?;
                
                log::info!("Job completed: {}", job_id);
                break;
            }
        }

        Ok(())
    }

    /// Обработка ошибки задачи
    pub async fn fail(&self, job_id: &str, error: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        let jobs: Vec<String> = conn.zrangebyscore(&self.processing_queue, "-inf", "+inf").await?;
        
        for job_json in jobs {
            let mut job: Job = serde_json::from_str(&job_json)?;
            if job.id == job_id {
                job.retry_count += 1;
                job.error = Some(error.to_string());

                if job.retry_count < job.max_retries {
                    // Возвращаем в очередь с увеличенным приоритетом (retry)
                    job.status = JobStatus::Pending;
                    job.started_at = None;
                    
                    let updated_json = serde_json::to_string(&job)?;
                    let _: () = conn.zrem(&self.processing_queue, &job_json).await?;
                    let _: () = conn.zadd(&self.queue_name, &updated_json, job.priority as f64 + 1.0).await?;
                    
                    log::warn!("Job {} failed, retrying ({}/{}): {}", job_id, job.retry_count, job.max_retries, error);
                } else {
                    // Превышен лимит retry — помечаем как failed
                    job.status = JobStatus::Failed;
                    job.completed_at = Some(Utc::now());
                    
                    let updated_json = serde_json::to_string(&job)?;
                    let _: () = conn.zrem(&self.processing_queue, &job_json).await?;
                    let _: () = conn.zadd("jobs:failed", &updated_json, Utc::now().timestamp() as f64).await?;
                    
                    log::error!("Job {} failed permanently: {}", job_id, error);
                }
                break;
            }
        }

        Ok(())
    }

    /// Получение статуса задачи
    pub async fn get_status(&self, job_id: &str) -> Result<Option<Job>, Box<dyn std::error::Error>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        // Проверяем во всех очередях
        let queues = vec![&self.queue_name, &self.processing_queue, "jobs:completed", "jobs:failed"];
        
        for queue in queues {
            let jobs: Vec<String> = conn.zrangebyscore(*queue, "-inf", "+inf").await?;
            
            for job_json in jobs {
                let job: Job = serde_json::from_str(&job_json)?;
                if job.id == job_id {
                    return Ok(Some(job));
                }
            }
        }

        Ok(None)
    }

    /// Получение статистики очереди
    pub async fn get_stats(&self) -> Result<QueueStats, Box<dyn std::error::Error>> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        
        let pending: i64 = conn.zcard(&self.queue_name).await?;
        let processing: i64 = conn.zcard(&self.processing_queue).await?;
        let completed: i64 = conn.zcard("jobs:completed").await?;
        let failed: i64 = conn.zcard("jobs:failed").await?;

        Ok(QueueStats {
            pending: pending as usize,
            processing: processing as usize,
            completed: completed as usize,
            failed: failed as usize,
            workers: self.workers_count,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub workers: usize,
}

/// Worker для обработки задач
pub struct JobWorker {
    queue: JobQueue,
    shutdown_rx: mpsc::Receiver<()>,
}

impl JobWorker {
    pub fn new(queue: JobQueue, shutdown_rx: mpsc::Receiver<()>) -> Self {
        Self { queue, shutdown_rx }
    }

    /// Запуск worker'а
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Job worker started");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    log::info!("Job worker shutting down");
                    break;
                }
                result = self.process_next_job() => {
                    match result {
                        Ok(true) => { /* Задача обработана */ }
                        Ok(false) => {
                            // Нет задач — ждём
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                        Err(e) => {
                            log::error!("Job processing error: {}", e);
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Обработка следующей задачи
    async fn process_next_job(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(job) = self.queue.dequeue().await? {
            log::info!("Processing job: {:?}", job.job_type);

            match self.execute_job(&job).await {
                Ok(result) => {
                    self.queue.complete(&job.id, result).await?;
                }
                Err(e) => {
                    self.queue.fail(&job.id, &e.to_string()).await?;
                }
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Выполнение задачи
    async fn execute_job(&self, job: &Job) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match &job.job_type {
            JobType::TranscodeVideo { input_path, output_dir, qualities } => {
                self.transcode_video(input_path, output_dir, qualities).await
            }
            JobType::TranscodeAudio { input_path, output_dir, format } => {
                self.transcode_audio(input_path, output_dir, format).await
            }
            JobType::CreateTeaser { input_path, output_path, start_seconds, duration_seconds, media_type } => {
                self.create_teaser(input_path, output_path, *start_seconds, *duration_seconds, media_type).await
            }
            JobType::GenerateThumbnail { input_path, output_path, timestamp_seconds } => {
                self.generate_thumbnail(input_path, output_path, *timestamp_seconds).await
            }
            JobType::ApplyForensicWatermark { input_path, output_path, user_id } => {
                self.apply_forensic_watermark(input_path, output_path, user_id).await
            }
            JobType::ExtractMetadata { input_path, content_id } => {
                self.extract_metadata(input_path, content_id).await
            }
        }
    }

    /// Транскодирование видео
    async fn transcode_video(&self, input: &str, output_dir: &str, qualities: &[String]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        for quality in qualities {
            let output = format!("{}/{}.m3u8", output_dir, quality);
            
            let (resolution, bitrate) = match quality.as_str() {
                "360p" => ("640x360", "500k"),
                "720p" => ("1280x720", "1500k"),
                "1080p" => ("1920x1080", "3000k"),
                _ => continue,
            };

            let output = tokio::process::Command::new("ffmpeg")
                .args(&[
                    "-i", input,
                    "-c:v", "libx264",
                    "-s", resolution,
                    "-b:v", bitrate,
                    "-c:a", "aac",
                    "-b:a", "128k",
                    "-f", "hls",
                    "-hls_time", "6",
                    "-hls_playlist_type", "vod",
                    "-hls_segment_filename", &format!("{}/{}_segment_%03d.ts", output_dir, quality),
                    &output,
                    "-y",
                ])
                .output()
                .await?;

            if !output.status.success() {
                return Err(format!("FFmpeg failed for quality {}: {}", quality, String::from_utf8_lossy(&output.stderr)).into());
            }
        }

        Ok(serde_json::json!({ "status": "success", "qualities": qualities }))
    }

    /// Транскодирование аудио
    async fn transcode_audio(&self, input: &str, output_dir: &str, format: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let output = format!("{}/playlist.m3u8", output_dir);

        let cmd = match format {
            "hls" => {
                tokio::process::Command::new("ffmpeg")
                    .args(&[
                        "-i", input,
                        "-c:a", "aac",
                        "-b:a", "256k",
                        "-f", "hls",
                        "-hls_time", "10",
                        "-hls_playlist_type", "vod",
                        "-hls_segment_filename", &format!("{}/segment_%03d.ts", output_dir),
                        &output,
                        "-y",
                    ])
                    .output()
                    .await?
            }
            _ => return Err(format!("Unsupported format: {}", format).into()),
        };

        if !cmd.status.success() {
            return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&cmd.stderr)).into());
        }

        Ok(serde_json::json!({ "status": "success", "format": format }))
    }

    /// Создание тизера
    async fn create_teaser(&self, input: &str, output: &str, start: i32, duration: i32, media_type: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let args = match media_type {
            "audio" => vec![
                "-i".to_string(), input.to_string(),
                "-ss".to_string(), start.to_string(),
                "-t".to_string(), duration.to_string(),
                "-acodec".to_string(), "libmp3lame".to_string(),
                "-b:a".to_string(), "192k".to_string(),
                output.to_string(),
                "-y".to_string(),
            ],
            "video" => vec![
                "-i".to_string(), input.to_string(),
                "-ss".to_string(), start.to_string(),
                "-t".to_string(), duration.to_string(),
                "-c:v".to_string(), "libx264".to_string(),
                "-c:a".to_string(), "aac".to_string(),
                output.to_string(),
                "-y".to_string(),
            ],
            _ => return Err(format!("Unsupported media type: {}", media_type).into()),
        };

        let result = tokio::process::Command::new("ffmpeg")
            .args(&args)
            .output()
            .await?;

        if !result.status.success() {
            return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&result.stderr)).into());
        }

        Ok(serde_json::json!({ "status": "success", "output": output }))
    }

    /// Генерация превью
    async fn generate_thumbnail(&self, input: &str, output: &str, timestamp: f32) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let result = tokio::process::Command::new("ffmpeg")
            .args(&[
                "-i", input,
                "-ss", &timestamp.to_string(),
                "-vframes", "1",
                "-q:v", "2",
                output,
                "-y",
            ])
            .output()
            .await?;

        if !result.status.success() {
            return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&result.stderr)).into());
        }

        Ok(serde_json::json!({ "status": "success", "output": output }))
    }

    /// Применение forensic watermark
    async fn apply_forensic_watermark(&self, input: &str, output: &str, user_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Используем FFmpeg с drawtext для видимого watermark (упрощённая версия)
        // В реальности — используем ForensicWatermarkService для стеганографии
        let watermark_text = format!("{} · {}", user_id, &user_id[..8.min(user_id.len())]);
        
        let result = tokio::process::Command::new("ffmpeg")
            .args(&[
                "-i", input,
                "-vf", &format!("drawtext=text='{}':fontsize=18:fontcolor=white@0.3:x=10:y=10", watermark_text),
                "-c:a", "copy",
                output,
                "-y",
            ])
            .output()
            .await?;

        if !result.status.success() {
            return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&result.stderr)).into());
        }

        Ok(serde_json::json!({ "status": "success", "output": output, "user_id": user_id }))
    }

    /// Извлечение метаданных
    async fn extract_metadata(&self, input: &str, content_id: &Uuid) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let result = tokio::process::Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                input,
            ])
            .output()
            .await?;

        if !result.status.success() {
            return Err(format!("FFprobe failed: {}", String::from_utf8_lossy(&result.stderr)).into());
        }

        let metadata: serde_json::Value = serde_json::from_slice(&result.stdout)?;

        Ok(serde_json::json!({
            "status": "success",
            "content_id": content_id,
            "metadata": metadata
        }))
    }
}
