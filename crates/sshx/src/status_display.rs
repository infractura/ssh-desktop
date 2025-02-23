use std::io::Write;
use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
use crate::xpra_status::{XpraStatus, SessionStatus};

#[derive(Tabled)]
struct SessionRow {
    #[tabled(rename = "ID")]
    session_id: String,
    #[tabled(rename = "User")]
    user: String,
    #[tabled(rename = "Display")]
    display: String,
    #[tabled(rename = "Port")]
    port: String,
    #[tabled(rename = "Idle")]
    idle: String,
}

pub fn display_status(status: &XpraStatus, format: &str, active_only: bool) -> Result<()> {
    match format {
        "json" => display_json(status)?,
        "text" => display_text(status, active_only)?,
        _ => anyhow::bail!("Unsupported format: {}", format),
    }
    Ok(())
}

fn display_json(status: &XpraStatus) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(status)?);
    Ok(())
}

fn display_text(status: &XpraStatus, active_only: bool) -> Result<()> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // Display configuration
    writeln!(out, "\n{}", "Configuration:".bold())?;
    writeln!(out, "  Window Manager: {}", status.config.window_manager)?;
    writeln!(out, "  Display Range: :{} - :{}", 
        status.config.min_display, status.config.max_display)?;
    writeln!(out, "  Base Port: {}", status.config.base_port)?;
    writeln!(out, "  Idle Timeout: {}s", status.config.idle_timeout)?;
    writeln!(out, "  Max Sessions/User: {}", 
        if status.config.max_sessions == 0 { 
            "unlimited".to_string() 
        } else { 
            status.config.max_sessions.to_string() 
        }
    )?;

    // Display metrics
    writeln!(out, "\n{}", "Metrics:".bold())?;
    writeln!(out, "  Uptime: {}", status.metrics.uptime.cyan())?;
    writeln!(out, "  Total Sessions: {}", status.metrics.total_sessions)?;
    writeln!(out, "  Active Sessions: {}", 
        status.metrics.active_sessions.to_string().green())?;
    writeln!(out, "  Failed Sessions: {}", 
        status.metrics.failed_sessions.to_string().red())?;
    writeln!(out, "  Idle Terminations: {}", status.metrics.idle_terminations)?;

    // Display sessions table
    let sessions: Vec<SessionRow> = status.sessions.iter()
        .filter(|s| !active_only || s.idle_time < status.config.idle_timeout)
        .map(|s| SessionRow {
            session_id: s.session_id.clone(),
            user: s.user.clone(),
            display: format!(":{}", s.display),
            port: s.websocket_port.to_string(),
            idle: format_idle_time(s.idle_time),
        })
        .collect();

    if !sessions.is_empty() {
        writeln!(out, "\n{}", "Active Sessions:".bold())?;
        let table = Table::new(sessions).to_string();
        writeln!(out, "{}", table)?;
    } else {
        writeln!(out, "\n{}", "No active sessions".yellow())?;
    }

    Ok(())
}

fn format_idle_time(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}
