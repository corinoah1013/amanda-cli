//! Output formatting (text, JSON, CSV)

use amanda_core::format_bytes;
use crate::monitor::{ProcessInfo, SystemSnapshot};
use anyhow::Result;

#[derive(Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

/// Format output based on selected format
pub fn format_output(
    snapshot: &SystemSnapshot,
    processes: &[ProcessInfo],
    format: OutputFormat,
    top: usize,
    include_system: bool,
) -> Result<String> {
    // Limit processes if top > 0
    let display_processes = if top > 0 && top < processes.len() {
        &processes[..top]
    } else {
        processes
    };
    
    match format {
        OutputFormat::Text => format_text(snapshot, display_processes, include_system),
        OutputFormat::Json => format_json(snapshot, display_processes, include_system),
        OutputFormat::Csv => format_csv(display_processes),
    }
}

fn format_text(
    snapshot: &SystemSnapshot,
    processes: &[ProcessInfo],
    include_system: bool,
) -> Result<String> {
    let mut output = String::new();
    
    // Header
    output.push_str(&format!("╔{:═^78}╗\n", ""));
    output.push_str(&format!("║{: ^78}║\n", "AMANDA-WATCH System Monitor"));
    output.push_str(&format!("╠{:═^78}╣\n", ""));
    output.push_str(&format!("║ Timestamp: {:<65}║\n", snapshot.timestamp.to_rfc3339()));
    output.push_str(&format!("║ Hostname:  {:<65}║\n", snapshot.hostname));
    output.push_str(&format!("╚{:═^78}╝\n\n", ""));
    
    // System resources
    if include_system {
        output.push_str(&format!("┌{:─^78}┐\n", " System Resources "));
        output.push_str(&format!("│ CPU: {:<6.1}% ({}){:>48}│\n", 
            snapshot.cpu.usage_percent,
            snapshot.cpu.brand.chars().take(20).collect::<String>(),
            ""
        ));
        output.push_str(&format!("│ Cores: {:<4} @ {} MHz{:>52}│\n", 
            snapshot.cpu.core_count,
            snapshot.cpu.frequency_mhz,
            ""
        ));
        output.push_str(&format!("│ Memory: {:>6.1}% ({}){:>47}│\n",
            snapshot.memory.used_percent(),
            format_bytes(snapshot.memory.used_bytes),
            ""
        ));
        output.push_str(&format!("│          {:>6} / {:<6}{:>47}│\n",
            format_bytes(snapshot.memory.used_bytes),
            format_bytes(snapshot.memory.total_bytes),
            ""
        ));
        if snapshot.memory.swap_total_bytes > 0 {
            output.push_str(&format!("│ Swap:   {:>6.1}% ({}){:>47}│\n",
                snapshot.memory.swap_used_percent(),
                format_bytes(snapshot.memory.swap_used_bytes),
                ""
            ));
        }
        output.push_str(&format!("│ Load: {:.2}, {:.2}, {:.2}{:>54}│\n",
            snapshot.load_average.one,
            snapshot.load_average.five,
            snapshot.load_average.fifteen,
            ""
        ));
        output.push_str(&format!("└{:─^78}┘\n\n", ""));
    }
    
    // Process table header
    output.push_str(&format!("┌{:─^78}┐\n", format!(" Processes ({}) ", processes.len())));
    output.push_str(&format!("│ {:>8} │ {:<20} │ {:>6} │ {:>10} │ {:>8} │ {:<10} │\n",
        "PID", "NAME", "CPU%", "MEMORY", "MEM%", "USER"
    ));
    output.push_str(&format!("├{:─^78}┤\n", ""));
    
    // Process rows
    for proc in processes {
        let name = if proc.name.len() > 20 {
            format!("{}...", &proc.name[..17])
        } else {
            proc.name.clone()
        };
        
        let user = proc.user.as_deref().unwrap_or("-");
        let user_display = if user.len() > 10 {
            format!("{}..", &user[..8])
        } else {
            user.to_string()
        };
        
        output.push_str(&format!(
            "│ {:>8} │ {:<20} │ {:>6.1} │ {:>10} │ {:>7.1}% │ {:<10} │\n",
            proc.pid,
            name,
            proc.cpu_percent,
            format_bytes(proc.memory_bytes),
            proc.memory_percent,
            user_display
        ));
    }
    
    output.push_str(&format!("└{:─^78}┘", ""));
    
    Ok(output)
}

fn format_json(
    snapshot: &SystemSnapshot,
    processes: &[ProcessInfo],
    include_system: bool,
) -> Result<String> {
    #[derive(serde::Serialize)]
    struct JsonOutput<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        system: Option<&'a SystemSnapshot>,
        processes: &'a [ProcessInfo],
        process_count: usize,
    }
    
    let output = JsonOutput {
        system: if include_system { Some(snapshot) } else { None },
        processes,
        process_count: processes.len(),
    };
    
    Ok(serde_json::to_string_pretty(&output)?)
}

fn format_csv(processes: &[ProcessInfo]) -> Result<String> {
    let mut output = String::new();
    
    // Header
    output.push_str("pid,name,cmd,cpu_percent,memory_bytes,memory_percent,status,user,run_time_secs\n");
    
    // Rows
    for proc in processes {
        let cmd = proc.cmd.join(" ").replace('"', "\"\"");
        let status = format!("{:?}", proc.status);
        let user = proc.user.as_deref().unwrap_or("");
        
        output.push_str(&format!(
            "{},\"{}",
            proc.pid,
            proc.name.replace('"', "\"\"")
        ));
        output.push_str(&format!(
            "\",\"{}",
            cmd
        ));
        output.push_str(&format!(
            "\",{},{},{},{},\"{}\",{}\n",
            proc.cpu_percent,
            proc.memory_bytes,
            proc.memory_percent,
            status,
            user.replace('"', "\"\""),
            proc.run_time_secs
        ));
    }
    
    Ok(output)
}

