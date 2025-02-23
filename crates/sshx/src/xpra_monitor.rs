use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time;
use tracing::{debug, info, warn};
use crate::xpra_config::CONFIG;

#[derive(Debug)]
struct SessionInfo {
    user: String,
    display: u16,
    last_activity: Instant,
}

#[derive(Debug, Clone)]
pub struct SessionMonitor {
    sessions: Arc<Mutex<HashMap<String, SessionInfo>>>,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user: String,
    pub display: u16,
    pub last_activity: Instant,
}

impl SessionMonitor {
    pub fn new() -> Self {
        let monitor = Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        };

        // Start cleanup task if idle timeout is configured
        if let Some(timeout) = CONFIG.idle_duration() {
            monitor.start_cleanup_task(timeout);
        }

        monitor
    }

    pub async fn register_session(&self, session_id: String, user: String, display: u16) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), SessionInfo {
            user: user.clone(),
            display,
            last_activity: Instant::now(),
        });
        debug!(user, display, "Registered new Xpra session");

        // Log session creation
        if let Err(e) = LOGGER.log_session_event(SessionEvent {
            timestamp: Utc::now(),
            event_type: SessionEventType::Created,
            session_id,
            user,
            display,
        }).await {
            error!("Failed to log session creation: {}", e);
        }
    }

    pub async fn update_activity(&self, session_id: &str) {
        if let Some(session) = self.sessions.lock().await.get_mut(session_id) {
            session.last_activity = Instant::now();
        }
    }

    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.remove(session_id) {
            debug!(
                user = session.user,
                display = session.display,
                "Removed Xpra session"
            );
        }
    }

    pub async fn get_user_session_count(&self, user: &str) -> usize {
        self.sessions.lock().await
            .values()
            .filter(|s| s.user == user)
            .count()
    }

    pub async fn get_all_sessions(&self) -> HashMap<String, SessionInfo> {
        self.sessions.lock().await.clone()
    }

    fn start_cleanup_task(&self, timeout: Duration) {
        let monitor = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                monitor.cleanup_idle_sessions(timeout).await;
            }
        });
    }

    async fn cleanup_idle_sessions(&self, timeout: Duration) {
        let mut sessions = self.sessions.lock().await;
        let now = Instant::now();
        
        let idle_sessions: Vec<_> = sessions
            .iter()
            .filter(|(_, info)| now.duration_since(info.last_activity) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in idle_sessions {
            if let Some(session) = sessions.remove(&session_id) {
                info!(
                    user = session.user,
                    display = session.display,
                    "Terminated idle Xpra session"
                );
                
                // Log session termination
                if let Err(e) = LOGGER.log_session_event(SessionEvent {
                    timestamp: Utc::now(),
                    event_type: SessionEventType::IdleTimeout,
                    session_id,
                    user: session.user.clone(),
                    display: session.display,
                }).await {
                    error!("Failed to log session termination: {}", e);
                }
                
                // Release display number
                crate::xpra_pool::DISPLAY_POOL.release(session.display).await;
            }
        }
    }
}

// Global monitor instance
lazy_static::lazy_static! {
    pub static ref SESSION_MONITOR: SessionMonitor = SessionMonitor::new();
}
