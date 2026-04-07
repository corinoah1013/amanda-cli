//! Alert system for threshold monitoring

use amanda_core::format_bytes;
use crate::monitor::{ProcessInfo, SystemSnapshot};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertConfig {
    /// CPU usage threshold (percent)
    pub cpu_threshold: Option<f32>,
    
    /// Memory usage threshold (percent)
    pub memory_threshold: Option<f32>,
    
    /// Watch for a specific process by name
    pub watch_process: Option<String>,
    
    /// Alert if any process uses more than X CPU
    pub process_cpu_threshold: Option<f32>,
    
    /// Alert if any process uses more than X memory (bytes)
    pub process_memory_threshold: Option<u64>,
}

impl AlertConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Triggered alert
#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Alert engine
pub struct AlertEngine {
    config: AlertConfig,
    /// Track which processes we've already alerted on to avoid spam
    alerted_pids: HashSet<u32>,
    /// Track if watched process was previously seen
    watched_process_seen: bool,
}

impl AlertEngine {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            alerted_pids: HashSet::new(),
            watched_process_seen: false,
        }
    }
    
    /// Check system state and generate alerts
    pub fn check(&mut self, snapshot: &SystemSnapshot, processes: &[ProcessInfo]) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let now = chrono::Utc::now();
        
        // Check system CPU
        if let Some(threshold) = self.config.cpu_threshold {
            if snapshot.cpu.usage_percent > threshold {
                alerts.push(Alert {
                    level: AlertLevel::Warning,
                    message: format!(
                        "System CPU usage is {:.1}% (threshold: {:.1}%)",
                        snapshot.cpu.usage_percent, threshold
                    ),
                    timestamp: now,
                });
            }
        }
        
        // Check system memory
        if let Some(threshold) = self.config.memory_threshold {
            let mem_percent = snapshot.memory.used_percent();
            if mem_percent > threshold {
                alerts.push(Alert {
                    level: AlertLevel::Warning,
                    message: format!(
                        "System memory usage is {:.1}% (threshold: {:.1}%)",
                        mem_percent, threshold
                    ),
                    timestamp: now,
                });
            }
        }
        
        // Check per-process thresholds
        for proc in processes {
            // Process CPU threshold
            if let Some(threshold) = self.config.process_cpu_threshold {
                if proc.cpu_percent > threshold && !self.alerted_pids.contains(&proc.pid) {
                    alerts.push(Alert {
                        level: AlertLevel::Info,
                        message: format!(
                            "Process '{}' (PID {}) is using {:.1}% CPU",
                            proc.name, proc.pid, proc.cpu_percent
                        ),
                        timestamp: now,
                    });
                    self.alerted_pids.insert(proc.pid);
                }
            }
            
            // Process memory threshold
            if let Some(threshold) = self.config.process_memory_threshold {
                if proc.memory_bytes > threshold && !self.alerted_pids.contains(&proc.pid) {
                    alerts.push(Alert {
                        level: AlertLevel::Info,
                        message: format!(
                            "Process '{}' (PID {}) is using {} memory",
                            proc.name, proc.pid, format_bytes(proc.memory_bytes)
                        ),
                        timestamp: now,
                    });
                    self.alerted_pids.insert(proc.pid);
                }
            }
        }
        
        // Check watched process
        if let Some(ref watch_name) = self.config.watch_process {
            let found = processes.iter().any(|p| {
                p.name.eq_ignore_ascii_case(watch_name) ||
                p.exe.as_ref().map(|e| e.contains(watch_name)).unwrap_or(false)
            });
            
            if found {
                if !self.watched_process_seen {
                    // Process just appeared
                    alerts.push(Alert {
                        level: AlertLevel::Info,
                        message: format!("Watched process '{}' is now running", watch_name),
                        timestamp: now,
                    });
                    self.watched_process_seen = true;
                }
            } else if self.watched_process_seen {
                // Process disappeared
                alerts.push(Alert {
                    level: AlertLevel::Warning,
                    message: format!("Watched process '{}' has terminated!", watch_name),
                    timestamp: now,
                });
                self.watched_process_seen = false;
            }
        }
        
        alerts
    }
}

