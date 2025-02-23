use std::pin::Pin;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tracing::{debug, error, info};

use crate::encrypt::Encrypt;
use crate::xpra::XpraDisplay;
use sshx_core::proto::{client_update::ClientMessage, TerminalData};
use sshx_core::Sid;

pub async fn xpra_task(
    id: Sid,
    encrypt: Encrypt,
    display: XpraDisplay,
    mut shell_rx: mpsc::Receiver<ShellData>,
    output_tx: mpsc::Sender<ClientMessage>,
) -> Result<()> {
    info!(
        display = display.display(),
        port = display.websocket_port(),
        "Starting Xpra WebSocket forwarder"
    );

    // Connect to Xpra's WebSocket server
    let ws_url = format!("ws://127.0.0.1:{}/xpra", display.websocket_port());
    let (ws_stream, _) = connect_async(ws_url).await?;
    
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let mut seq = 0u64;

    loop {
        tokio::select! {
            // Handle incoming messages from client
            Some(msg) = shell_rx.recv() => {
                match msg {
                    ShellData::Data(data) => {
                        // Forward decrypted data to Xpra
                        if let Err(e) = ws_write.send(data.into()).await {
                            error!("Failed to forward data to Xpra: {}", e);
                            break;
                        }
                    }
                    ShellData::Size(rows, cols) => {
                        // Handle resize events if needed
                        debug!(rows, cols, "Resize event received");
                    }
                    ShellData::Sync(server_seq) => {
                        // Update our sequence number if server is ahead
                        if server_seq > seq {
                            seq = server_seq;
                        }
                    }
                }
            }

            // Handle messages from Xpra
            Some(msg) = ws_read.next() => {
                match msg {
                    Ok(msg) => {
                        // Encrypt data before sending to client
                        let data = encrypt.segment(
                            0x100000000 | id.0 as u64,
                            seq,
                            &msg.into_data()
                        );

                        let term_data = TerminalData {
                            id: id.0,
                            data: data.into(),
                            seq,
                        };

                        if let Err(e) = output_tx.send(ClientMessage::Data(term_data)).await {
                            error!("Failed to send data to client: {}", e);
                            break;
                        }

                        seq += msg.len() as u64;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }

            // Check if Xpra is still running
            else => {
                if !display.is_running() {
                    info!("Xpra process terminated");
                    break;
                }
            }
        }
    }

    info!("Xpra WebSocket forwarder terminated");
    Ok(())
}

// Helper function to start a new Xpra session
pub async fn start_xpra_session(
    id: Sid,
    user: String,
    encrypt: Encrypt,
    shell_rx: mpsc::Receiver<ShellData>,
    output_tx: mpsc::Sender<ClientMessage>,
) -> Result<()> {
    use crate::xpra_config::CONFIG;
    use crate::xpra_monitor::SESSION_MONITOR;

    // Check session limit
    let session_count = SESSION_MONITOR.get_user_session_count(&user).await;
    if CONFIG.max_sessions > 0 && session_count >= CONFIG.max_sessions as usize {
        anyhow::bail!("Maximum number of Xpra sessions reached for user");
    }

    // Create new display
    let display = XpraDisplay::new(&CONFIG.window_manager).await?;
    
    // Register session
    let session_id = format!("xpra-{}", id.0);
    SESSION_MONITOR.register_session(session_id.clone(), user, display.display()).await;
    METRICS.session_started();

    // Run the Xpra task
    xpra_task(id, encrypt, display, shell_rx, output_tx).await
}
