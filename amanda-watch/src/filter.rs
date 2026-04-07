//! Process filtering functionality

use crate::monitor::ProcessInfo;
use regex::Regex;

/// Filter for processes
#[derive(Clone, Default)]
pub struct ProcessFilter {
    name_pattern: Option<Regex>,
    pid: Option<u32>,
    cpu_above: Option<f32>,
    memory_above: Option<u64>,
}

impl ProcessFilter {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Filter by name pattern (regex)
    pub fn with_name_pattern(mut self, pattern: &str) -> anyhow::Result<Self> {
        let regex = Regex::new(pattern)?;
        self.name_pattern = Some(regex);
        Ok(self)
    }
    
    /// Filter by specific PID
    pub fn with_pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }
    
    /// Filter by CPU usage threshold
    pub fn with_cpu_above(mut self, percent: f32) -> Self {
        self.cpu_above = Some(percent);
        self
    }
    
    /// Filter by memory usage threshold (bytes)
    pub fn with_memory_above(mut self, bytes: u64) -> Self {
        self.memory_above = Some(bytes);
        self
    }
    
    /// Check if a process matches this filter
    pub fn matches(&self, process: &ProcessInfo) -> bool {
        // Name pattern filter
        if let Some(ref pattern) = self.name_pattern {
            if !pattern.is_match(&process.name) {
                return false;
            }
        }
        
        // PID filter
        if let Some(pid) = self.pid {
            if process.pid != pid {
                return false;
            }
        }
        
        // CPU threshold filter
        if let Some(cpu) = self.cpu_above {
            if process.cpu_percent < cpu {
                return false;
            }
        }
        
        // Memory threshold filter
        if let Some(mem) = self.memory_above {
            if process.memory_bytes < mem {
                return false;
            }
        }
        
        true
    }
    
    /// Check if filter is empty (matches all)
    pub fn is_empty(&self) -> bool {
        self.name_pattern.is_none()
            && self.pid.is_none()
            && self.cpu_above.is_none()
            && self.memory_above.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::ProcessStatus;
    
    fn create_test_process(pid: u32, name: &str, cpu: f32, mem: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.to_string(),
            cmd: vec![name.to_string()],
            exe: Some(name.to_string()),
            cpu_percent: cpu,
            memory_bytes: mem,
            memory_percent: 0.0,
            status: crate::monitor::ProcessStatusStr("Run".to_string()),
            user: Some("user".to_string()),
            start_time: None,
            run_time_secs: 0,
        }
    }
    
    #[test]
    fn test_empty_filter_matches_all() {
        let filter = ProcessFilter::new();
        let proc = create_test_process(1, "test", 10.0, 1000);
        
        assert!(filter.matches(&proc));
    }
    
    #[test]
    fn test_name_filter() {
        let filter = ProcessFilter::new()
            .with_name_pattern("^test").unwrap();
        
        assert!(filter.matches(&create_test_process(1, "test-process", 0.0, 0)));
        assert!(!filter.matches(&create_test_process(2, "other", 0.0, 0)));
    }
    
    #[test]
    fn test_pid_filter() {
        let filter = ProcessFilter::new().with_pid(42);
        
        assert!(filter.matches(&create_test_process(42, "test", 0.0, 0)));
        assert!(!filter.matches(&create_test_process(1, "test", 0.0, 0)));
    }
    
    #[test]
    fn test_cpu_filter() {
        let filter = ProcessFilter::new().with_cpu_above(50.0);
        
        assert!(filter.matches(&create_test_process(1, "test", 75.0, 0)));
        assert!(!filter.matches(&create_test_process(1, "test", 25.0, 0)));
    }
    
    #[test]
    fn test_memory_filter() {
        let filter = ProcessFilter::new().with_memory_above(1024 * 1024 * 100); // 100MB
        
        assert!(filter.matches(&create_test_process(1, "test", 0.0, 1024 * 1024 * 200)));
        assert!(!filter.matches(&create_test_process(1, "test", 0.0, 1024 * 1024)));
    }
}
