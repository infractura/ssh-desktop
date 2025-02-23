use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use anyhow::Result;

#[derive(Debug, Serialize)]
pub struct LogAnalysis {
    pub period: AnalysisPeriod,
    pub session_stats: SessionStats,
    pub user_stats: HashMap<String, UserStats>,
    pub hourly_distribution: Vec<HourlyStats>,
}

#[derive(Debug, Serialize)]
pub struct AnalysisPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub total_sessions: u64,
    pub avg_duration: Duration,
    pub max_concurrent: u32,
    pub idle_terminations: u64,
    pub failed_sessions: u64,
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_sessions: u32,
    pub total_duration: Duration,
    pub avg_session_duration: Duration,
    pub idle_terminations: u32,
}

#[derive(Debug, Serialize)]
pub struct HourlyStats {
    pub hour: u32,
    pub session_count: u32,
}

pub struct LogAnalyzer {
    log_dir: PathBuf,
}

impl LogAnalyzer {
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    pub async fn analyze_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<LogAnalysis> {
        let mut analysis = LogAnalysis {
            period: AnalysisPeriod { start, end },
            session_stats: SessionStats {
                total_sessions: 0,
                avg_duration: Duration::zero(),
                max_concurrent: 0,
                idle_terminations: 0,
                failed_sessions: 0,
            },
            user_stats: HashMap::new(),
            hourly_distribution: vec![HourlyStats { hour: 0, session_count: 0 }; 24],
        };

        // Process history log
        self.process_history_log(&mut analysis, start, end).await?;
        
        // Process metrics log for concurrent session data
        self.process_metrics_log(&mut analysis, start, end).await?;

        Ok(analysis)
    }

    async fn process_history_log(
        &self,
        analysis: &mut LogAnalysis,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<()> {
        let history_path = self.log_dir.join("history.log");
        let content = tokio::fs::read_to_string(history_path).await?;

        let mut session_starts: HashMap<String, (DateTime<Utc>, String)> = HashMap::new();

        for line in content.lines() {
            let event: crate::xpra_logger::SessionEvent = serde_json::from_str(line)?;
            
            if event.timestamp < start || event.timestamp > end {
                continue;
            }

            match event.event_type {
                crate::xpra_logger::SessionEventType::Created => {
                    session_starts.insert(
                        event.session_id,
                        (event.timestamp, event.user)
                    );
                    
                    // Update hourly distribution
                    let hour = event.timestamp.hour() as usize;
                    analysis.hourly_distribution[hour].session_count += 1;
                }
                crate::xpra_logger::SessionEventType::Terminated |
                crate::xpra_logger::SessionEventType::IdleTimeout |
                crate::xpra_logger::SessionEventType::Failed => {
                    if let Some((start_time, user)) = session_starts.remove(&event.session_id) {
                        let duration = event.timestamp - start_time;
                        
                        // Update user stats
                        let user_stats = analysis.user_stats
                            .entry(user)
                            .or_insert_with(|| UserStats {
                                total_sessions: 0,
                                total_duration: Duration::zero(),
                                avg_session_duration: Duration::zero(),
                                idle_terminations: 0,
                            });
                        
                        user_stats.total_sessions += 1;
                        user_stats.total_duration = user_stats.total_duration + duration;
                        user_stats.avg_session_duration = user_stats.total_duration / 
                            user_stats.total_sessions as i32;
                        
                        if matches!(event.event_type, 
                            crate::xpra_logger::SessionEventType::IdleTimeout) {
                            user_stats.idle_terminations += 1;
                            analysis.session_stats.idle_terminations += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_metrics_log(
        &self,
        analysis: &mut LogAnalysis,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<()> {
        let metrics_path = self.log_dir.join("metrics.log");
        let content = tokio::fs::read_to_string(metrics_path).await?;

        let mut max_concurrent = 0;

        for line in content.lines() {
            let entry: crate::xpra_logger::LogEntry = serde_json::from_str(line)?;
            
            if entry.timestamp < start || entry.timestamp > end {
                continue;
            }

            max_concurrent = max_concurrent.max(entry.metrics.active_sessions as u32);
        }

        analysis.session_stats.max_concurrent = max_concurrent;

        Ok(())
    }
}
