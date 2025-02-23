use std::process::{Child, Command};
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{debug, error};

const BASE_WS_PORT: u16 = 14500;
const MAX_DISPLAYS: u16 = 500;

pub struct XpraDisplay {
    display: u16,
    process: Child,
    websocket_port: u16,
}

impl XpraDisplay {
    /// Create a new Xpra display with the given number and window manager
    pub async fn new(wm: &str) -> Result<Self> {
        // Get display number from pool
        let display = crate::xpra_pool::DISPLAY_POOL.allocate().await?;

        // Calculate websocket port - each display gets its own port
        let websocket_port = BASE_WS_PORT + display;

        // Ensure the port is available
        let listener = TcpListener::bind(("127.0.0.1", websocket_port)).await?;
        drop(listener);

        // Start xpra process
        let process = Command::new("xpra")
            .args([
                "start",
                &format!(":${display}"),
                &format!("--bind-ws=127.0.0.1:${websocket_port}"),
                "--start",
                wm,
                "--html=on",
                "--pulseaudio=no",
                "--daemon=no",
                "--exit-with-children=yes"
            ])
            .spawn()?;

        debug!(
            display = display,
            port = websocket_port,
            pid = process.id(),
            "Started new Xpra display"
        );

        Ok(Self {
            display,
            process,
            websocket_port,
        })
    }

    /// Get the display number
    pub fn display(&self) -> u16 {
        self.display
    }

    /// Get the websocket port
    pub fn websocket_port(&self) -> u16 {
        self.websocket_port
    }

    /// Check if the Xpra process is still running
    pub fn is_running(&mut self) -> bool {
        self.process.try_wait().map(|status| status.is_none()).unwrap_or(false)
    }
}

impl Drop for XpraDisplay {
    fn drop(&mut self) {
        // Return display number to pool
        tokio::spawn({
            let pool = crate::xpra_pool::DISPLAY_POOL.clone();
            let display = self.display;
            async move {
                pool.release(display).await;
            }
        });
        // Ensure xpra process is terminated
        if let Err(e) = self.process.kill() {
            error!(
                display = self.display,
                error = ?e,
                "Failed to kill Xpra process"
            );
        }
        
        // Wait for process to fully terminate
        if let Err(e) = self.process.wait() {
            error!(
                display = self.display,
                error = ?e,
                "Failed to wait for Xpra process termination"
            );
        }

        debug!(
            display = self.display,
            "Terminated Xpra display"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xpra_display_lifecycle() {
        let mut display = XpraDisplay::new(1, "gnome-flashback")
            .await
            .expect("Failed to create display");

        assert_eq!(display.display(), 1);
        assert_eq!(display.websocket_port(), BASE_WS_PORT + 1);
        assert!(display.is_running());

        // Display should be cleaned up when dropped
        drop(display);
    }
}
