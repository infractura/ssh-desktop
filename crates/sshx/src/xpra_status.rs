use std::collections::HashMap;
use serde::Serialize;
use tokio::time::{Duration, Instant};

use crate::xpra_metrics::METRICS;
use crate::xpra_monitor::SESSION_MONITOR;
use crate::xpra_config::CONFIG;

#[derive(Debug, Serialize)]
pub struct SessionStatus {
    pub session_id: String,
    pub user: String,
    pub display: u16,
    pub idle_time: u64,
    pub websocket_port: u16,
}

#[derive(Debug, Serialize)]
pub struct XpraStatus {
    pub config: ConfigStatus,
    pub sessions: Vec<SessionStatus>,
    pub metrics: MetricsStatus,
}

#[derive(Debug, Serialize)]
pub struct ConfigStatus {
    pub min_display: u16,
    pub max_display: u16,
    pub base_port: u16,
    pub window_manager: String,
    pub idle_timeout: u64,
    pub max_sessions: u32,
}

#[derive(Debug, Serialize)]
pub struct MetricsStatus {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub failed_sessions: u64,
    pub idle_terminations: u64,
    pub uptime: String,
}

pub async fn get_status() -> XpraStatus {
    let metrics = METRICS.get_metrics();
    
    XpraStatus {
        config: ConfigStatus {
            min_display: CONFIG.min_display,
            max_display: CONFIG.max_display,
            base_port: CONFIG.base_port,
            window_manager: CONFIG.window_manager.clone(),
            idle_timeout: CONFIG.idle_timeout,
            max_sessions: CONFIG.max_sessions,
        },
        sessions: get_session_status().await,
        metrics: MetricsStatus {
            total_sessions: metrics.total_sessions,
            active_sessions: metrics.active_sessions,
            failed_sessions: metrics.failed_sessions,
            idle_terminations: metrics.idle_terminations,
            uptime: format_duration(Duration::from_secs(metrics.uptime_secs)),
        },
    }
}

async fn get_session_status() -> Vec<SessionStatus> {
    let monitor = SESSION_MONITOR.clone();
    let sessions = monitor.get_all_sessions().await;
    
    sessions.into_iter()
        .map(|(id, info)| SessionStatus {
            session_id: id,
            user: info.user,
            display: info.display,
            idle_time: info.last_activity.elapsed().as_secs(),
            websocket_port: CONFIG.websocket_port(info.display),
        })
        .collect()
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
