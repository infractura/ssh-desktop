use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use lazy_static::lazy_static;

#[derive(Debug)]
pub struct XpraMetrics {
    total_sessions: AtomicU64,
    active_sessions: AtomicU64,
    failed_sessions: AtomicU64,
    idle_terminations: AtomicU64,
    start_time: Instant,
}

impl XpraMetrics {
    pub fn new() -> Self {
        Self {
            total_sessions: AtomicU64::new(0),
            active_sessions: AtomicU64::new(0),
            failed_sessions: AtomicU64::new(0),
            idle_terminations: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn session_started(&self) {
        self.total_sessions.fetch_add(1, Ordering::Relaxed);
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn session_ended(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn session_failed(&self) {
        self.failed_sessions.fetch_add(1, Ordering::Relaxed);
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn idle_terminated(&self) {
        self.idle_terminations.fetch_add(1, Ordering::Relaxed);
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get_metrics(&self) -> XpraMetricsSnapshot {
        XpraMetricsSnapshot {
            total_sessions: self.total_sessions.load(Ordering::Relaxed),
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
            failed_sessions: self.failed_sessions.load(Ordering::Relaxed),
            idle_terminations: self.idle_terminations.load(Ordering::Relaxed),
            uptime_secs: self.start_time.elapsed().as_secs(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct XpraMetricsSnapshot {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub failed_sessions: u64,
    pub idle_terminations: u64,
    pub uptime_secs: u64,
}

lazy_static! {
    pub static ref METRICS: XpraMetrics = XpraMetrics::new();
}
