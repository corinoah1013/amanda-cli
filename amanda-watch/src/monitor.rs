//! System monitoring with sysinfo

use serde::Serialize;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessStatus, RefreshKind, System};

/// System resource snapshot
#[derive(Debug, Clone, Serialize)]
pub struct SystemSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub hostname: String,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub load_average: LoadAverage,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub usage_percent: f32,
    pub core_count: usize,
    pub brand: String,
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

impl MemoryInfo {
    pub fn used_percent(&self) -> f32 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.used_bytes as f32 / self.total_bytes as f32) * 100.0
        }
    }
    
    pub fn swap_used_percent(&self) -> f32 {
        if self.swap_total_bytes == 0 {
            0.0
        } else {
            (self.swap_used_bytes as f32 / self.swap_total_bytes as f32) * 100.0
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

/// Process information
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: Option<String>,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_percent: f32,
    pub status: ProcessStatusStr,
    pub user: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub run_time_secs: u64,
}

/// Wrapper for ProcessStatus that implements Serialize
#[derive(Debug, Clone, Serialize)]
pub struct ProcessStatusStr(pub String);

impl From<ProcessStatus> for ProcessStatusStr {
    fn from(status: ProcessStatus) -> Self {
        Self(format!("{:?}", status))
    }
}

/// Sorting options for processes
#[derive(Clone, Copy)]
pub enum SortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

/// System monitor using sysinfo
pub struct SystemMonitor {
    system: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );
        
        Self { system }
    }
    
    /// Take a snapshot of system resources
    pub fn snapshot(&mut self) -> SystemSnapshot {
        // Refresh all data
        self.system.refresh_all();
        
        // Small delay for CPU measurement
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.system.refresh_cpu_all();
        
        let cpus = self.system.cpus();
        let cpu_usage = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };
        
        let cpu_info = CpuInfo {
            usage_percent: cpu_usage,
            core_count: cpus.len(),
            brand: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
            frequency_mhz: cpus.first().map(|c| c.frequency()).unwrap_or(0),
        };
        
        let memory_info = MemoryInfo {
            total_bytes: self.system.total_memory(),
            used_bytes: self.system.used_memory(),
            free_bytes: self.system.free_memory(),
            swap_total_bytes: self.system.total_swap(),
            swap_used_bytes: self.system.used_swap(),
        };
        
        let load = System::load_average();
        let load_average = LoadAverage {
            one: load.one,
            five: load.five,
            fifteen: load.fifteen,
        };
        
        SystemSnapshot {
            timestamp: chrono::Utc::now(),
            hostname: gethostname::gethostname().unwrap_or_else(|_| "unknown".to_string()),
            cpu: cpu_info,
            memory: memory_info,
            load_average,
        }
    }
    
    /// Get filtered and sorted processes
    pub fn processes(
        &mut self,
        filter: &super::filter::ProcessFilter,
        sort_by: SortBy,
    ) -> Vec<ProcessInfo> {
        self.system.refresh_all();
        
        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .filter_map(|(pid, process)| {
                let info = ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cmd: process.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect(),
                    exe: process.exe().map(|p| p.to_string_lossy().to_string()),
                    cpu_percent: process.cpu_usage(),
                    memory_bytes: process.memory(),
                    memory_percent: (process.memory() as f32 / self.system.total_memory() as f32) * 100.0,
                    status: process.status().into(),
                    user: process.effective_user_id().map(|u| u.to_string()),
                    start_time: chrono::DateTime::from_timestamp(process.start_time() as i64, 0),
                    run_time_secs: process.run_time(),
                };
                
                if filter.matches(&info) {
                    Some(info)
                } else {
                    None
                }
            })
            .collect();
        
        // Sort
        match sort_by {
            SortBy::Cpu => {
                processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap());
            }
            SortBy::Memory => {
                processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
            }
            SortBy::Pid => {
                processes.sort_by_key(|p| p.pid);
            }
            SortBy::Name => {
                processes.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        
        processes
    }
    
    /// Check if a process with the given name exists
    pub fn process_exists(&self, name: &str) -> bool {
        self.system.processes().values().any(|p| {
            p.name().to_string_lossy().eq_ignore_ascii_case(name)
        })
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// Reuse gethostname from amanda-core
use amanda_core::gethostname;
