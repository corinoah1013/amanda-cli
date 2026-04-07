//! Snapshot saving to .amaudit and .amrpt formats

use crate::alert::Alert;
use crate::monitor::{ProcessInfo, SystemSnapshot};
use amanda_core::{amaudit::AuditTrail, amrpt::Report};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Save system snapshot as .amaudit (audit trail format)
pub fn save_snapshot(
    path: impl AsRef<Path>,
    snapshot: &SystemSnapshot,
    processes: &[ProcessInfo],
) -> Result<()> {
    let path = path.as_ref();
    
    // Load existing trail or create new one
    let mut trail = if path.exists() {
        AuditTrail::load(path)?
    } else {
        AuditTrail::new("amanda-watch")
    };
    
    // Create snapshot entry
    let snapshot_data = serde_json::json!({
        "system": snapshot,
        "top_processes": processes.iter().take(10).collect::<Vec<_>>(),
        "process_count": processes.len(),
    });
    
    trail.add_entry("system_snapshot", "amanda-watch", snapshot_data)?;
    
    trail.save(path)?;
    Ok(())
}

/// Save report as .amrpt format
pub fn save_report(
    path: impl AsRef<Path>,
    snapshot: &SystemSnapshot,
    processes: &[ProcessInfo],
    alerts: &[Alert],
) -> Result<()> {
    let mut report = Report::new("amanda-watch", "system_monitor", "System Monitor Report");
    
    // System summary section
    let mut summary: HashMap<String, f64> = HashMap::new();
    summary.insert("cpu_usage".to_string(), snapshot.cpu.usage_percent as f64);
    summary.insert("memory_usage".to_string(), snapshot.memory.used_percent() as f64);
    summary.insert("process_count".to_string(), processes.len() as f64);
    
    report.add_metrics("System Summary", summary);
    
    // Hostname in text section
    let mut metadata = HashMap::new();
    metadata.insert("hostname".to_string(), snapshot.hostname.clone());
    
    report.add_section(amanda_core::amrpt::ReportSection {
        title: "Host Info".to_string(),
        section_type: amanda_core::amrpt::SectionType::Text,
        data: serde_json::to_value(metadata)?,
        metadata: None,
    });
    
    // Top processes table
    report.add_table("Top Processes", processes.to_vec())?;
    
    // Alerts section
    if !alerts.is_empty() {
        let alerts_data: Vec<_> = alerts.iter().map(|a| {
            serde_json::json!({
                "level": format!("{:?}", a.level),
                "message": a.message.clone(),
                "timestamp": a.timestamp,
            })
        }).collect();
        
        report.add_section(amanda_core::amrpt::ReportSection {
            title: "Alerts".to_string(),
            section_type: amanda_core::amrpt::SectionType::List,
            data: serde_json::to_value(alerts_data)?,
            metadata: None,
        });
    }
    
    report.finalize();
    report.save(path)?;
    
    Ok(())
}
