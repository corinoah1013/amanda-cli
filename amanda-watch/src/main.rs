//! # amanda-watch
//! 
//! Process and resource monitoring for Amanda OS.
//! Equivalent to `htop` but designed for scripting, automation, and pipelines.

use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use tracing::{info, warn};

mod alert;
mod filter;
mod monitor;
mod output;
mod snapshot;
mod tui;

use alert::{AlertConfig, AlertEngine};
use filter::ProcessFilter;
use monitor::SystemMonitor;
use output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "amanda-watch",
    about = "Process and resource monitoring for Amanda OS",
    version
)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormatArg,
    
    /// Number of top processes to show (0 = all)
    #[arg(short, long, default_value = "20")]
    top: usize,
    
    /// Sort by field
    #[arg(short, long, value_enum, default_value = "cpu")]
    sort: SortField,
    
    /// Filter by process name (regex supported)
    #[arg(long, value_name = "PATTERN")]
    filter_name: Option<String>,
    
    /// Filter by specific PID
    #[arg(long, value_name = "PID")]
    filter_pid: Option<u32>,
    
    /// Filter processes using more than X% CPU
    #[arg(long, value_name = "PERCENT")]
    filter_cpu_above: Option<f32>,
    
    /// Filter processes using more than X MB memory
    #[arg(long, value_name = "MB")]
    filter_mem_above: Option<u64>,
    
    /// Polling interval in seconds (0 = single shot)
    #[arg(short, long, default_value = "0")]
    interval: u64,
    
    /// Save system snapshot to .amaudit file
    #[arg(long, value_name = "FILE")]
    snapshot: Option<PathBuf>,
    
    /// Load alert configuration from file
    #[arg(long, value_name = "FILE")]
    alert_config: Option<PathBuf>,
    
    /// Alert threshold: CPU% (e.g., --alert-cpu 80)
    #[arg(long, value_name = "PERCENT")]
    alert_cpu: Option<f32>,
    
    /// Alert threshold: Memory% (e.g., --alert-mem 90)
    #[arg(long, value_name = "PERCENT")]
    alert_mem: Option<f32>,
    
    /// Include system resources information
    #[arg(long)]
    system: bool,
    
    /// Watch specific process by name and alert when it exits
    #[arg(long, value_name = "NAME")]
    watch_process: Option<String>,
    
    /// Export to .amrpt report file
    #[arg(long, value_name = "FILE")]
    report: Option<PathBuf>,

    /// Launch interactive TUI mode
    #[arg(long)]
    tui: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormatArg {
    Text,
    Json,
    Csv,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Text => OutputFormat::Text,
            OutputFormatArg::Json => OutputFormat::Json,
            OutputFormatArg::Csv => OutputFormat::Csv,
        }
    }
}

#[derive(Clone, Copy, ValueEnum, Default)]
enum SortField {
    #[default]
    Cpu,
    Mem,
    Pid,
    Name,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Build process filter
    let filter = build_filter(&cli)?;
    
    // Build alert configuration
    let alert_config = build_alert_config(&cli)?;
    let mut alert_engine = AlertEngine::new(alert_config);
    
    // Create system monitor
    let mut monitor = SystemMonitor::new();

    // TUI mode
    if cli.tui {
        let refresh = cli.interval.max(1);
        return tui::run_tui(monitor, filter, cli.sort.into(), cli.top, refresh).await;
    }

    // Single shot mode
    if cli.interval == 0 {
        if cli.watch_process.is_some() {
            warn!("--watch-process requires --interval to detect process termination; running single snapshot only");
        }

        info!("Running single system snapshot");

        let snapshot = monitor.snapshot().await;
        let processes = monitor.processes(&filter, cli.sort.into());
        
        // Check alerts
        let alerts = alert_engine.check(&snapshot, &processes);
        for alert in &alerts {
            warn!("ALERT: {}", alert.message);
        }
        
        // Output results
        let output = output::format_output(
            &snapshot,
            &processes,
            cli.format.clone().into(),
            cli.top,
            cli.system,
        )?;
        println!("{}", output);
        
        // Save snapshot if requested
        if let Some(path) = &cli.snapshot {
            snapshot::save_snapshot(path, &snapshot, &processes)?;
            info!("Snapshot saved to {}", path.display());
        }
        
        // Generate report if requested
        if let Some(path) = &cli.report {
            snapshot::save_report(path, &snapshot, &processes, &alerts)?;
            info!("Report saved to {}", path.display());
        }
        
        return Ok(());
    }
    
    // Continuous monitoring mode
    info!("Starting continuous monitoring (interval: {}s)", cli.interval);
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(cli.interval));
    
    loop {
        interval.tick().await;
        
        let snapshot = monitor.snapshot().await;
        let processes = monitor.processes(&filter, cli.sort.into());

        // Check alerts
        let alerts = alert_engine.check(&snapshot, &processes);
        for alert in &alerts {
            eprintln!("ALERT: {}", alert.message);
        }
        
        // Output results
        let output = output::format_output(
            &snapshot,
            &processes,
            cli.format.clone().into(),
            cli.top,
            cli.system,
        )?;
        println!("{}", output);
        
        // Save snapshot if requested (appends to file)
        if let Some(path) = &cli.snapshot {
            snapshot::save_snapshot(path, &snapshot, &processes)?;
        }
    }
}

fn build_filter(cli: &Cli) -> Result<ProcessFilter> {
    let mut filter = ProcessFilter::new();
    
    if let Some(name) = &cli.filter_name {
        filter = filter.with_name_pattern(name)?;
    }
    
    if let Some(pid) = cli.filter_pid {
        filter = filter.with_pid(pid);
    }
    
    if let Some(cpu) = cli.filter_cpu_above {
        filter = filter.with_cpu_above(cpu);
    }
    
    if let Some(mem) = cli.filter_mem_above {
        filter = filter.with_memory_above(mem * 1024 * 1024); // Convert MB to bytes
    }
    
    Ok(filter)
}

fn build_alert_config(cli: &Cli) -> Result<AlertConfig> {
    // Load from file if specified
    let mut config = if let Some(path) = &cli.alert_config {
        AlertConfig::load(path)?
    } else {
        AlertConfig::default()
    };
    
    // Override with CLI arguments
    if let Some(cpu) = cli.alert_cpu {
        config.cpu_threshold = Some(cpu);
    }
    
    if let Some(mem) = cli.alert_mem {
        config.memory_threshold = Some(mem);
    }
    
    if let Some(name) = &cli.watch_process {
        config.watch_process = Some(name.clone());
    }
    
    Ok(config)
}

impl From<SortField> for monitor::SortBy {
    fn from(field: SortField) -> Self {
        match field {
            SortField::Cpu => monitor::SortBy::Cpu,
            SortField::Mem => monitor::SortBy::Memory,
            SortField::Pid => monitor::SortBy::Pid,
            SortField::Name => monitor::SortBy::Name,
        }
    }
}
