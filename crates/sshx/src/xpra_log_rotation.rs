use std::fs::{self, File};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use tokio::time::{self, Duration};
use tracing::{error, info};
use glob::glob;

const MAX_LOG_AGE_DAYS: i64 = 30;
const MAX_LOG_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10MB

pub struct LogRotator {
    log_dir: PathBuf,
}

impl LogRotator {
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    pub fn start_rotation(&self) {
        let rotator = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(3600)); // Check hourly
            loop {
                interval.tick().await;
                if let Err(e) = rotator.rotate_logs().await {
                    error!("Failed to rotate logs: {}", e);
                }
            }
        });
    }

    async fn rotate_logs(&self) -> anyhow::Result<()> {
        let metrics_path = self.log_dir.join("metrics.log");
        let history_path = self.log_dir.join("history.log");

        // Check and rotate current log files
        self.check_and_rotate_file(&metrics_path).await?;
        self.check_and_rotate_file(&history_path).await?;

        // Clean up old rotated logs
        self.cleanup_old_logs().await?;

        Ok(())
    }

    async fn check_and_rotate_file(&self, path: &Path) -> anyhow::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_LOG_SIZE_BYTES {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let rotated_path = path.with_extension(format!("log.{}", timestamp));
            
            // Rename current log file
            fs::rename(path, &rotated_path)?;
            
            // Create new empty log file
            File::create(path)?;
            
            // Compress rotated log
            let rotator = self.clone();
            let rotated_path_clone = rotated_path.clone();
            tokio::spawn(async move {
                if let Err(e) = rotator.compress_log(&rotated_path_clone).await {
                    error!("Failed to compress rotated log: {}", e);
                }
            });

            info!(
                path = path.display(),
                rotated = rotated_path.display(),
                "Rotated log file"
            );
        }

        Ok(())
    }

    async fn compress_log(&self, path: &Path) -> anyhow::Result<()> {
        let input = fs::read(path)?;
        let compressed_path = path.with_extension("log.gz");
        
        // Compress using gzip
        let mut encoder = flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default()
        );
        std::io::copy(&mut &input[..], &mut encoder)?;
        let compressed = encoder.finish()?;
        
        // Write compressed file and remove original
        fs::write(&compressed_path, compressed)?;
        fs::remove_file(path)?;

        info!(
            original = path.display(),
            compressed = compressed_path.display(),
            "Compressed rotated log"
        );

        Ok(())
    }

    async fn cleanup_old_logs(&self) -> anyhow::Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(MAX_LOG_AGE_DAYS);
        
        for pattern in &["*.log.*", "*.log.gz"] {
            let glob_pattern = self.log_dir.join(pattern);
            for entry in glob(glob_pattern.to_str().unwrap())? {
                if let Ok(path) = entry {
                    if let Some(timestamp_str) = path.file_name()
                        .and_then(|n| n.to_str())
                        .and_then(|n| n.split('.').nth(2))
                    {
                        if let Ok(timestamp) = DateTime::parse_from_str(
                            &format!("{}+0000", timestamp_str),
                            "%Y%m%d_%H%M%S%z"
                        ) {
                            if timestamp < cutoff {
                                fs::remove_file(&path)?;
                                info!(path = path.display(), "Removed old log file");
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
