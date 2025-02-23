use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpraConfig {
    /// Minimum display number to allocate
    #[serde(default = "default_min_display")]
    pub min_display: u16,

    /// Maximum display number to allocate
    #[serde(default = "default_max_display")]
    pub max_display: u16,

    /// Base port for WebSocket connections
    #[serde(default = "default_base_port")]
    pub base_port: u16,

    /// Default window manager to use
    #[serde(default = "default_window_manager")]
    pub window_manager: String,

    /// Session idle timeout in seconds (0 = no timeout)
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// Maximum sessions per user (0 = unlimited)
    #[serde(default = "default_max_sessions")]
    pub max_sessions: u32,
}

fn default_min_display() -> u16 { 100 }
fn default_max_display() -> u16 { 599 }
fn default_base_port() -> u16 { 14500 }
fn default_window_manager() -> String { "gnome-flashback".to_string() }
fn default_idle_timeout() -> u64 { 3600 } // 1 hour
fn default_max_sessions() -> u32 { 5 }

impl Default for XpraConfig {
    fn default() -> Self {
        Self {
            min_display: default_min_display(),
            max_display: default_max_display(),
            base_port: default_base_port(),
            window_manager: default_window_manager(),
            idle_timeout: default_idle_timeout(),
            max_sessions: default_max_sessions(),
        }
    }
}

impl XpraConfig {
    pub fn idle_duration(&self) -> Option<Duration> {
        if self.idle_timeout == 0 {
            None
        } else {
            Some(Duration::from_secs(self.idle_timeout))
        }
    }

    pub fn websocket_port(&self, display: u16) -> u16 {
        self.base_port + (display - self.min_display)
    }
}

// Global config instance
lazy_static::lazy_static! {
    pub static ref CONFIG: XpraConfig = XpraConfig::default();
}
