use std::io::Write;
use chrono::{DateTime, Duration, Utc};
use colored::*;
use tabled::{Table, Tabled};
use terminal_charts::{Chart, ChartBuilder, TimeSeries};
use crate::xpra_log_analyzer::{LogAnalysis, UserStats};

#[derive(Tabled)]
struct UserRow {
    #[tabled(rename = "User")]
    user: String,
    #[tabled(rename = "Sessions")]
    sessions: String,
    #[tabled(rename = "Avg Duration")]
    avg_duration: String,
    #[tabled(rename = "Idle Terms")]
    idle_terms: String,
}

pub fn display_analysis(analysis: &LogAnalysis, format: &str) -> anyhow::Result<()> {
    match format {
        "json" => display_json(analysis),
        "text" => display_text(analysis),
        _ => anyhow::bail!("Unsupported format: {}", format),
    }
}

fn display_json(analysis: &LogAnalysis) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(analysis)?);
    Ok(())
}

fn display_text(analysis: &LogAnalysis) -> anyhow::Result<()> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // Period header
    writeln!(out, "\n{}", "Analysis Period:".bold())?;
    writeln!(out, "  From: {}", analysis.period.start.format("%Y-%m-%d %H:%M:%S UTC"))?;
    writeln!(out, "  To:   {}", analysis.period.end.format("%Y-%m-%d %H:%M:%S UTC"))?;

    // Overall statistics
    writeln!(out, "\n{}", "Session Statistics:".bold())?;
    writeln!(out, "  Total Sessions:     {}", analysis.session_stats.total_sessions)?;
    writeln!(out, "  Average Duration:   {}", format_duration(analysis.session_stats.avg_duration))?;
    writeln!(out, "  Max Concurrent:     {}", analysis.session_stats.max_concurrent)?;
    writeln!(out, "  Idle Terminations:  {}", analysis.session_stats.idle_terminations)?;
    writeln!(out, "  Failed Sessions:    {}", analysis.session_stats.failed_sessions.to_string().red())?;

    // User statistics table
    let user_rows: Vec<UserRow> = analysis.user_stats
        .iter()
        .map(|(user, stats)| UserRow {
            user: user.clone(),
            sessions: stats.total_sessions.to_string(),
            avg_duration: format_duration(stats.avg_session_duration),
            idle_terms: stats.idle_terminations.to_string(),
        })
        .collect();

    if !user_rows.is_empty() {
        writeln!(out, "\n{}", "User Statistics:".bold())?;
        let table = Table::new(user_rows).to_string();
        writeln!(out, "{}", table)?;
    }

    // Hourly distribution chart
    writeln!(out, "\n{}", "Hourly Distribution:".bold())?;
    display_hourly_chart(&mut out, &analysis.hourly_distribution)?;

    Ok(())
}

fn display_hourly_chart(out: &mut impl Write, distribution: &[HourlyStats]) -> anyhow::Result<()> {
    let data: Vec<(f64, f64)> = distribution.iter()
        .map(|stat| (stat.hour as f64, stat.session_count as f64))
        .collect();

    let chart = ChartBuilder::new()
        .width(60)
        .height(15)
        .caption("Sessions by Hour")
        .series(TimeSeries::new(data))
        .build()?;

    writeln!(out, "{}", chart)?;
    Ok(())
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
