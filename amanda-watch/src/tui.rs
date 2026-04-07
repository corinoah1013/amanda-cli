//! TUI mode — interactive terminal interface using ratatui

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use amanda_core::format_bytes;
use crate::filter::ProcessFilter;
use crate::monitor::{ProcessInfo, SortBy, SystemMonitor, SystemSnapshot};

struct AppState {
    snapshot: Option<SystemSnapshot>,
    processes: Vec<ProcessInfo>,
    table_state: TableState,
    sort_by: SortBy,
}

impl AppState {
    fn new(sort_by: SortBy) -> Self {
        Self {
            snapshot: None,
            processes: Vec::new(),
            table_state: TableState::default(),
            sort_by,
        }
    }

    fn scroll_down(&mut self) {
        let max = self.processes.len().saturating_sub(1);
        let next = (self.table_state.selected().unwrap_or(0) + 1).min(max);
        self.table_state.select(Some(next));
    }

    fn scroll_up(&mut self) {
        let prev = self.table_state.selected().unwrap_or(0).saturating_sub(1);
        self.table_state.select(Some(prev));
    }
}

pub async fn run_tui(
    mut monitor: SystemMonitor,
    filter: ProcessFilter,
    sort_by: SortBy,
    top: usize,
    refresh_secs: u64,
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut monitor, &filter, sort_by, top, refresh_secs).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    monitor: &mut SystemMonitor,
    filter: &ProcessFilter,
    sort_by: SortBy,
    top: usize,
    refresh_secs: u64,
) -> Result<()> {
    let mut state = AppState::new(sort_by);
    let refresh_interval = Duration::from_secs(refresh_secs.max(1));

    // Initial data fetch
    refresh_data(monitor, filter, &mut state, top).await;

    let mut last_refresh = Instant::now();

    loop {
        terminal.draw(|f| render(f, &mut state))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Down | KeyCode::Char('j') => state.scroll_down(),
                        KeyCode::Up | KeyCode::Char('k') => state.scroll_up(),
                        KeyCode::Char('c') => state.sort_by = SortBy::Cpu,
                        KeyCode::Char('m') => state.sort_by = SortBy::Memory,
                        KeyCode::Char('p') => state.sort_by = SortBy::Pid,
                        KeyCode::Char('n') => state.sort_by = SortBy::Name,
                        _ => {}
                    }
                }
            }
        }

        if last_refresh.elapsed() >= refresh_interval {
            refresh_data(monitor, filter, &mut state, top).await;
            last_refresh = Instant::now();
        }
    }

    Ok(())
}

async fn refresh_data(
    monitor: &mut SystemMonitor,
    filter: &ProcessFilter,
    state: &mut AppState,
    top: usize,
) {
    let snapshot = monitor.snapshot().await;
    let mut processes = monitor.processes(filter, state.sort_by);
    if top > 0 {
        processes.truncate(top);
    }
    state.snapshot = Some(snapshot);
    state.processes = processes;

    if state.table_state.selected().is_none() && !state.processes.is_empty() {
        state.table_state.select(Some(0));
    }
}

fn render(f: &mut Frame, state: &mut AppState) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, state, chunks[0]);
    render_stats(f, state, chunks[1]);
    render_processes(f, state, chunks[2]);
    render_footer(f, chunks[3]);
}

fn render_header(f: &mut Frame, state: &AppState, area: Rect) {
    let (hostname, timestamp) = if let Some(snap) = &state.snapshot {
        (
            snap.hostname.clone(),
            snap.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        )
    } else {
        ("loading...".to_string(), "—".to_string())
    };

    let line = Line::from(vec![
        Span::styled(
            " AMANDA-WATCH",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(hostname, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(paragraph, area);
}

fn render_stats(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let (cpu_pct, cpu_label, mem_pct, mem_label) = if let Some(snap) = &state.snapshot {
        let cpu = snap.cpu.usage_percent;
        let mem = snap.memory.used_percent();
        let cpu_label = format!(
            " CPU  {:.1}%  {} cores @ {} MHz ",
            cpu, snap.cpu.core_count, snap.cpu.frequency_mhz
        );
        let mem_label = format!(
            " MEM  {:.1}%  {} / {}  load: {:.2} ",
            mem,
            format_bytes(snap.memory.used_bytes),
            format_bytes(snap.memory.total_bytes),
            snap.load_average.one,
        );
        (cpu as f64, cpu_label, mem as f64, mem_label)
    } else {
        (0.0, " CPU ".to_string(), 0.0, " MEM ".to_string())
    };

    let cpu_color = gauge_color(cpu_pct);
    let mem_color = gauge_color(mem_pct);

    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title(cpu_label)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(cpu_color).bg(Color::Black))
        .ratio((cpu_pct / 100.0).clamp(0.0, 1.0));

    let mem_gauge = Gauge::default()
        .block(
            Block::default()
                .title(mem_label)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(Style::default().fg(mem_color).bg(Color::Black))
        .ratio((mem_pct / 100.0).clamp(0.0, 1.0));

    f.render_widget(cpu_gauge, chunks[0]);
    f.render_widget(mem_gauge, chunks[1]);
}

fn render_processes(f: &mut Frame, state: &mut AppState, area: Rect) {
    let sort_label = match state.sort_by {
        SortBy::Cpu => "cpu",
        SortBy::Memory => "mem",
        SortBy::Pid => "pid",
        SortBy::Name => "name",
    };

    let header = Row::new([
        Cell::from("PID").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("NAME").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("CPU%").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("MEMORY").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("MEM%").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("USER").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])
    .bottom_margin(0);

    let rows: Vec<Row> = state
        .processes
        .iter()
        .map(|p| {
            let cpu_color = if p.cpu_percent > 60.0 {
                Color::Red
            } else if p.cpu_percent > 25.0 {
                Color::Yellow
            } else {
                Color::White
            };

            let name = if p.name.len() > 20 {
                format!("{}...", &p.name[..17])
            } else {
                p.name.clone()
            };

            Row::new(vec![
                Cell::from(p.pid.to_string())
                    .style(Style::default().fg(Color::DarkGray)),
                Cell::from(name),
                Cell::from(format!("{:.1}", p.cpu_percent))
                    .style(Style::default().fg(cpu_color)),
                Cell::from(format_bytes(p.memory_bytes))
                    .style(Style::default().fg(Color::Blue)),
                Cell::from(format!("{:.1}%", p.memory_percent)),
                Cell::from(p.user.as_deref().unwrap_or("-"))
                    .style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(7),
        Constraint::Length(21),
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Min(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(
                    " Processes ({})  sort: {} ",
                    state.processes.len(),
                    sort_label
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(table, area, &mut state.table_state);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":salir  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(":scroll  "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(":CPU  "),
        Span::styled("m", Style::default().fg(Color::Yellow)),
        Span::raw(":MEM  "),
        Span::styled("p", Style::default().fg(Color::Yellow)),
        Span::raw(":PID  "),
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(":nombre"),
    ]);
    f.render_widget(Paragraph::new(line).style(Style::default().fg(Color::DarkGray)), area);
}

fn gauge_color(pct: f64) -> Color {
    if pct > 80.0 {
        Color::Red
    } else if pct > 50.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
