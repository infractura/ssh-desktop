use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use tracing::{error, info};

use crate::xpra_metrics::METRICS;
use crate::xpra_monitor::SESSION_MONITOR;

#[derive(Debug, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    metrics: MetricsLog,
    sessions: Vec<SessionLog>,
}

#[derive(Debug, Serialize)]
struct MetricsLog {
    total_sessions: u64,
    active_sessions: u64,
    failed_sessions: u64,
    idle_terminations: u64,
}

#[derive(Debug, Serialize)]
struct SessionLog {
    session_id: String,
    user: String,
    display: u16,
    idle_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct XpraLogger {
    log_dir: PathBuf,
    metrics_file: Arc<Mutex<File>>,
    history_file: Arc<Mutex<File>>,
}

impl XpraLogger {
    pub fn new(log_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&log_dir)?;
        
        let metrics_path = log_dir.join("metrics.log");
        let history_path = log_dir.join("history.log");
        
        let metrics_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&metrics_path)?;
            
        let history_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_path)?;

        Ok(Self {
            log_dir,
            metrics_file: Arc::new(Mutex::new(metrics_file)),
            history_file: Arc::new(Mutex::new(history_file)),
        })
    }

    pub fn start_logging(&self) {
        let logger = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(300)); // Log every 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = logger.log_metrics().await {
                    error!("Failed to log metrics: {}", e);
                }
            }
        });
    }

    async fn log_metrics(&self) -> anyhow::Result<()> {
        let metrics = METRICS.get_metrics();
        let sessions = SESSION_MONITOR.get_all_sessions().await;

        let entry = LogEntry {
            timestamp: Utc::now(),
            metrics: MetricsLog {
                total_sessions: metrics.total_sessions,
                active_sessions: metrics.active_sessions,
                failed_sessions: metrics.failed_sessions,
                idle_terminations: metrics.idle_terminations,
            },
            sessions: sessions.iter().map(|(id, info)| SessionLog {
                session_id: id.clone(),
                user: info.user.clone(),
                display: info.display,
                idle_seconds: info.last_activity.elapsed().as_secs(),
            }).collect(),
        };

        // Log to metrics file
        let mut metrics_file = self.metrics_file.lock().await;
        serde_json::to_writer(&mut *metrics_file, &entry)?;
        writeln!(metrics_file)?;

        Ok(())
    }

    pub async fn log_session_event(&self, event: SessionEvent) -> anyhow::Result<()> {
        let mut history_file = self.history_file.lock().await;
        serde_json::to_writer(&mut *history_file, &event)?;
        writeln!(history_file)?;
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct SessionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SessionEventType,
    pub session_id: String,
    pub user: String,
    pub display: u16,
}

#[derive(Debug, Serialize)]
pub enum SessionEventType {
    Created,
    Terminated,
    Failed,
    IdleTimeout,
}

// Global logger instance
lazy_static::lazy_static! {
    pub static ref LOGGER: XpraLogger = XpraLogger::new(
        PathBuf::from("/var/log/sshx/xpra")
    ).expect("Failed to initialize Xpra logger");
}
