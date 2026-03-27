mod app;
mod config;
mod input;
mod logger;
mod macros;
mod search;
mod serial;
mod ui;

use anyhow::{Context, Result};
use app::{AppState, Direction, LogLine, NewlineMode, PopupKind, ViewMode};
use chrono::Local;
use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use logger::{default_log_path, FileLogger, LogFormat};
use ratatui::{backend::CrosstermBackend, Terminal};
use serial::{SerialConfig, SerialEvent, TxCommand};
use std::io;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

#[derive(Parser, Debug)]
#[command(name = "easy_console", version = "0.1.0", about = "Serial console TUI")]
struct Cli {
    /// Serial port device path
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate
    #[arg(short, long, default_value = "115200")]
    baud: u32,

    /// Data bits
    #[arg(long, default_value = "8")]
    data_bits: u8,

    /// Parity (none/odd/even)
    #[arg(long, default_value = "none")]
    parity: String,

    /// Stop bits
    #[arg(long, default_value = "1")]
    stop_bits: u8,

    /// Flow control (none/hardware/software)
    #[arg(long, default_value = "none")]
    flow: String,

    /// TX newline (crlf/lf/cr/none)
    #[arg(long, default_value = "crlf")]
    tx_nl: String,

    /// RX newline (auto/crlf/lf/cr/none)
    #[arg(long, default_value = "auto")]
    rx_nl: String,

    /// View mode (ascii/hex/split)
    #[arg(long, default_value = "ascii")]
    view: String,

    /// Enable timestamp display
    #[arg(long)]
    timestamp: bool,

    /// Disable local echo
    #[arg(long)]
    no_echo: bool,

    /// Log file path
    #[arg(short, long)]
    log: Option<PathBuf>,

    /// Log format (text/hex/raw)
    #[arg(long, default_value = "text")]
    log_format: String,

    /// Profile name
    #[arg(long)]
    profile: Option<String>,

    /// List available ports and exit
    #[arg(long)]
    list_ports: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // List ports mode
    if cli.list_ports {
        let ports = serialport::available_ports().unwrap_or_default();
        if ports.is_empty() {
            println!("No serial ports found.");
        } else {
            println!("Available serial ports:");
            for p in ports {
                println!("  {}", p.port_name);
            }
        }
        return Ok(());
    }

    // Setup tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    // Build initial app state
    let mut state = AppState::new();

    // Apply CLI args
    state.serial_config = SerialConfig {
        baud_rate: cli.baud,
        data_bits: cli.data_bits,
        parity: cli.parity,
        stop_bits: cli.stop_bits,
        flow_control: cli.flow,
        dtr: false,
        rts: false,
        timeout_ms: 100,
    };

    state.tx_newline = match cli.tx_nl.to_lowercase().as_str() {
        "lf" => NewlineMode::Lf,
        "cr" => NewlineMode::Cr,
        "none" => NewlineMode::None_,
        _ => NewlineMode::CrLf,
    };
    state.rx_newline = match cli.rx_nl.to_lowercase().as_str() {
        "crlf" => NewlineMode::CrLf,
        "lf" => NewlineMode::Lf,
        "cr" => NewlineMode::Cr,
        "none" => NewlineMode::None_,
        _ => NewlineMode::Auto,
    };
    state.view_mode = match cli.view.to_lowercase().as_str() {
        "hex" => ViewMode::Hex,
        "split" => ViewMode::Split,
        _ => ViewMode::Ascii,
    };
    state.show_timestamp = cli.timestamp;
    state.local_echo = !cli.no_echo;
    state.port_path = cli.port;

    // Set log format
    state.log_format = match cli.log_format.to_lowercase().as_str() {
        "hex" => LogFormat::Hex,
        "raw" => LogFormat::Raw,
        _ => LogFormat::Text,
    };

    // Load profile if specified
    if let Some(profile_name) = &cli.profile {
        if let Ok(profiles) = config::load_profiles() {
            if let Some(profile) = profiles.profiles.get(profile_name) {
                apply_profile(&mut state, profile);
            }
        }
    }

    // Load macros from default profile
    if let Ok(profiles) = config::load_profiles() {
        if let Some(profile) = profiles.profiles.get("default") {
            if let Some(ms) = &profile.macros {
                state.macros = ms.iter().map(|m| crate::macros::Macro {
                    name: m.name.clone(),
                    steps: m.steps.iter().map(|s| crate::macros::MacroStep {
                        send: s.send.clone(),
                        delay_ms: s.delay_ms,
                    }).collect(),
                }).collect();
            }
        }
    }

    // Start file logging if requested
    if let Some(log_path) = cli.log {
        match FileLogger::new(log_path, state.log_format.clone()) {
            Ok(logger) => {
                state.file_logger = Some(logger);
                state.logging = true;
            }
            Err(e) => {
                eprintln!("Failed to open log file: {}", e);
            }
        }
    }

    // Channel for serial events
    let (serial_tx, mut serial_rx) = mpsc::channel::<SerialEvent>(1024);

    // Setup TUI
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initial port scan
    state.scan_ports();

    // Auto-connect if port specified
    if state.port_path.is_some() {
        connect_port(&mut state, serial_tx.clone());
    }

    let result = run_app(&mut terminal, &mut state, &mut serial_rx, serial_tx).await;

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn apply_profile(state: &mut AppState, profile: &config::Profile) {
    if let Some(port) = &profile.port {
        state.port_path = Some(port.clone());
    }
    if let Some(baud) = profile.baud_rate {
        state.serial_config.baud_rate = baud;
    }
    if let Some(data_bits) = profile.data_bits {
        state.serial_config.data_bits = data_bits;
    }
    if let Some(parity) = &profile.parity {
        state.serial_config.parity = parity.clone();
    }
    if let Some(stop_bits) = profile.stop_bits {
        state.serial_config.stop_bits = stop_bits;
    }
    if let Some(flow) = &profile.flow_control {
        state.serial_config.flow_control = flow.clone();
    }
    if let Some(tx_nl) = &profile.tx_newline {
        state.tx_newline = match tx_nl.to_lowercase().as_str() {
            "lf" => NewlineMode::Lf,
            "cr" => NewlineMode::Cr,
            "none" => NewlineMode::None_,
            _ => NewlineMode::CrLf,
        };
    }
    if let Some(rx_nl) = &profile.rx_newline {
        state.rx_newline = match rx_nl.to_lowercase().as_str() {
            "crlf" => NewlineMode::CrLf,
            "lf" => NewlineMode::Lf,
            "cr" => NewlineMode::Cr,
            "none" => NewlineMode::None_,
            _ => NewlineMode::Auto,
        };
    }
    if let Some(echo) = profile.local_echo {
        state.local_echo = echo;
    }
    if let Some(buf) = profile.scroll_buffer {
        state.scroll_buffer_size = buf;
    }
    if let Some(ts) = profile.timestamp {
        state.show_timestamp = ts;
    }
    if let Some(dtr) = profile.dtr_init {
        state.serial_config.dtr = dtr;
    }
    if let Some(rts) = profile.rts_init {
        state.serial_config.rts = rts;
    }
    if let Some(view) = &profile.view_mode {
        state.view_mode = match view.to_lowercase().as_str() {
            "hex" => ViewMode::Hex,
            "split" => ViewMode::Split,
            _ => ViewMode::Ascii,
        };
    }
    if let Some(ms) = &profile.macros {
        state.macros = ms.iter().map(|m| crate::macros::Macro {
            name: m.name.clone(),
            steps: m.steps.iter().map(|s| crate::macros::MacroStep {
                send: s.send.clone(),
                delay_ms: s.delay_ms,
            }).collect(),
        }).collect();
    }
}

fn connect_port(state: &mut AppState, serial_tx: mpsc::Sender<SerialEvent>) {
    let port_path = match &state.port_path {
        Some(p) => p.clone(),
        None => {
            state.set_status("No port selected.", true);
            return;
        }
    };

    // Disconnect existing
    if let Some(sender) = state.tx_sender.take() {
        let _ = sender.try_send(TxCommand::Close);
    }

    let config = state.serial_config.clone();

    // Open port once, then clone for writer
    let port = match serialport::new(&port_path, config.baud_rate)
        .data_bits(config.to_serialport_data_bits())
        .parity(config.to_serialport_parity())
        .stop_bits(config.to_serialport_stop_bits())
        .flow_control(config.to_serialport_flow_control())
        .timeout(std::time::Duration::from_millis(config.timeout_ms))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            state.set_status(format!("Failed to open port: {}", e), true);
            state.connected = false;
            return;
        }
    };
    let writer_port = match port.try_clone() {
        Ok(p) => p,
        Err(e) => {
            state.set_status(format!("Failed to clone port: {}", e), true);
            state.connected = false;
            return;
        }
    };

    // Start reader
    let reader_tx = serial_tx.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = serial::reader::start_reader(port, reader_tx.clone()) {
            let _ = reader_tx.blocking_send(SerialEvent::Error(e.to_string()));
            let _ = reader_tx.blocking_send(SerialEvent::Disconnected);
        }
    });

    // Start writer
    let (tx_cmd_tx, tx_cmd_rx) = mpsc::channel::<TxCommand>(256);
    tokio::task::spawn_blocking(move || {
        let _ = serial::writer::start_writer(writer_port, tx_cmd_rx);
    });

    // Set DTR/RTS if configured
    if config.dtr {
        let _ = tx_cmd_tx.try_send(TxCommand::SetDtr(true));
    }
    if config.rts {
        let _ = tx_cmd_tx.try_send(TxCommand::SetRts(true));
    }

    state.tx_sender = Some(tx_cmd_tx);
    state.connected = true;
    state.signals.dtr = config.dtr;
    state.signals.rts = config.rts;
    state.stats.connected_at = Some(Local::now());

    let msg = format!("Connected to {} at {} baud.", port_path, config.baud_rate);
    state.add_log_line(LogLine::new_system(msg.clone()));
    state.set_status(msg, false);
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
    serial_rx: &mut mpsc::Receiver<SerialEvent>,
    serial_tx: mpsc::Sender<SerialEvent>,
) -> Result<()> {
    let mut tick = interval(Duration::from_millis(16));
    let mut rx_buffer: Vec<u8> = Vec::new();

    loop {
        // Drain serial events
        loop {
            match serial_rx.try_recv() {
                Ok(event) => {
                    handle_serial_event(state, event, &mut rx_buffer);
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }

        // Process buffered RX data into lines
        process_rx_buffer(state, &mut rx_buffer);

        // Render
        terminal.draw(|f| ui::render(f, state))?;

        // Poll for keyboard events (non-blocking)
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(state, key, serial_tx.clone()) {
                    break;
                }
            }
        }

        tick.tick().await;
    }

    Ok(())
}

fn process_rx_buffer(state: &mut AppState, buffer: &mut Vec<u8>) {
    if buffer.is_empty() {
        return;
    }

    let data = buffer.clone();
    buffer.clear();

    // Split into lines based on rx_newline mode
    let lines = split_rx_data(&data, &state.rx_newline.clone());
    for line_bytes in lines {
        if line_bytes.is_empty() {
            continue;
        }
        let line = LogLine::new_rx(line_bytes.clone());
        // Log to file
        if state.logging {
            if let Some(logger) = &mut state.file_logger {
                let _ = logger.log_rx(&line_bytes);
            }
        }
        state.add_log_line(line);
    }
}

fn split_rx_data(data: &[u8], mode: &NewlineMode) -> Vec<Vec<u8>> {
    match mode {
        NewlineMode::Auto => split_auto(data),
        NewlineMode::CrLf => split_by(data, b"\r\n"),
        NewlineMode::Lf => split_by_byte(data, b'\n'),
        NewlineMode::Cr => split_by_byte(data, b'\r'),
        NewlineMode::None_ => vec![data.to_vec()],
    }
}

fn split_auto(data: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\r' {
            if i + 1 < data.len() && data[i + 1] == b'\n' {
                lines.push(current.clone());
                current.clear();
                i += 2;
            } else {
                lines.push(current.clone());
                current.clear();
                i += 1;
            }
        } else if data[i] == b'\n' {
            lines.push(current.clone());
            current.clear();
            i += 1;
        } else {
            current.push(data[i]);
            i += 1;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn split_by(data: &[u8], sep: &[u8]) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if i + sep.len() <= data.len() && &data[i..i + sep.len()] == sep {
            lines.push(current.clone());
            current.clear();
            i += sep.len();
        } else {
            current.push(data[i]);
            i += 1;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn split_by_byte(data: &[u8], sep: u8) -> Vec<Vec<u8>> {
    let mut lines = Vec::new();
    let mut current = Vec::new();
    for &b in data {
        if b == sep {
            lines.push(current.clone());
            current.clear();
        } else {
            current.push(b);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn handle_serial_event(state: &mut AppState, event: SerialEvent, rx_buffer: &mut Vec<u8>) {
    match event {
        SerialEvent::Data(data) => {
            rx_buffer.extend_from_slice(&data);
        }
        SerialEvent::Error(msg) => {
            state.stats.error_count += 1;
            let line = LogLine::new_system(format!("Error: {}", msg));
            state.add_log_line(line);
            state.set_status(format!("Error: {}", msg), true);
        }
        SerialEvent::Disconnected => {
            state.connected = false;
            let line = LogLine::new_system("Port disconnected.".to_string());
            state.add_log_line(line);
            state.set_status("Port disconnected.", true);
        }
    }
}

/// Returns true if the app should quit
fn handle_key_event(
    state: &mut AppState,
    key: KeyEvent,
    serial_tx: mpsc::Sender<SerialEvent>,
) -> bool {
    // If popup is open, handle popup keys first
    if let Some(popup) = state.active_popup.clone() {
        return handle_popup_key(state, key, popup, serial_tx);
    }

    // Search mode
    if state.search_active {
        return handle_search_key(state, key);
    }

    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        // Quit
        KeyCode::Char('c') if ctrl => return true,
        KeyCode::Char('q') if ctrl => return true,

        // Function keys
        KeyCode::F(1) => {
            state.active_popup = Some(PopupKind::Help);
        }
        KeyCode::F(2) => {
            state.scan_ports();
            state.active_popup = Some(PopupKind::PortSelect);
        }
        KeyCode::F(3) => {
            state.active_popup = Some(PopupKind::Settings);
        }
        KeyCode::F(4) => {
            state.view_mode = state.view_mode.next();
        }
        KeyCode::F(5) => {
            toggle_logging(state);
        }
        KeyCode::F(6) => {
            state.active_popup = Some(PopupKind::Macros);
        }
        KeyCode::F(7) => {
            state.search_active = true;
            state.search_query.clear();
        }

        // Ctrl combinations
        KeyCode::Char('l') if ctrl => {
            state.clear_log();
            state.add_log_line(LogLine::new_system("[Log cleared]".to_string()));
        }
        KeyCode::Char('k') if ctrl => {
            // Clear display only - scroll to current bottom
            state.scroll_to_bottom();
        }
        KeyCode::Char('x') if ctrl => {
            state.disconnect();
            state.clear_log();
            state.port_path = None;
        }
        KeyCode::Char('e') if ctrl => {
            state.tx_newline = state.tx_newline.next_send();
        }
        KeyCode::Char('r') if ctrl => {
            state.local_echo = !state.local_echo;
            let msg = if state.local_echo { "Local echo ON" } else { "Local echo OFF" };
            state.set_status(msg, false);
        }
        KeyCode::Char('w') if ctrl => {
            state.raw_mode_input = !state.raw_mode_input;
            let msg = if state.raw_mode_input { "Raw mode ON" } else { "Raw mode OFF" };
            state.set_status(msg, false);
        }
        KeyCode::Char('i') if ctrl => {
            state.show_stats = !state.show_stats;
        }
        KeyCode::Char('y') if ctrl => {
            copy_log_to_clipboard(state);
        }

        // Search shortcut
        KeyCode::Char('/') if !ctrl => {
            state.search_active = true;
            state.search_query.clear();
        }

        // Scroll (when input is empty)
        KeyCode::Up if state.input.is_empty() => {
            state.scroll_up(1);
        }
        KeyCode::Down if state.input.is_empty() => {
            state.scroll_down(1);
        }
        KeyCode::PageUp => {
            state.scroll_up(20);
        }
        KeyCode::PageDown => {
            state.scroll_down(20);
        }
        KeyCode::Home => {
            state.scroll_offset = 0;
            state.scroll_locked = true;
        }
        KeyCode::End => {
            state.scroll_to_bottom();
            state.scroll_locked = false;
        }

        // History navigation (when input has content or arrow keys)
        KeyCode::Up => {
            let current = state.input.clone();
            if let Some(entry) = state.send_history.navigate_up(&current) {
                state.input = entry.to_string();
                state.cursor_pos = state.input.len();
            }
        }
        KeyCode::Down => {
            if let Some(entry) = state.send_history.navigate_down() {
                state.input = entry.to_string();
                state.cursor_pos = state.input.len();
            }
        }

        // Input handling
        KeyCode::Enter => {
            send_input(state);
        }
        KeyCode::Backspace => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
                state.input.remove(state.cursor_pos);
            }
        }
        KeyCode::Delete => {
            if state.cursor_pos < state.input.len() {
                state.input.remove(state.cursor_pos);
            }
        }
        KeyCode::Left => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
            }
        }
        KeyCode::Right => {
            if state.cursor_pos < state.input.len() {
                state.cursor_pos += 1;
            }
        }
        KeyCode::Char(c) if !ctrl => {
            if state.raw_mode_input {
                // Send immediately
                let data = vec![c as u8];
                if let Some(sender) = &state.tx_sender {
                    let _ = sender.try_send(TxCommand::Send(data.clone()));
                }
                if state.local_echo {
                    state.add_log_line(LogLine::new_tx(data));
                }
            } else {
                state.input.insert(state.cursor_pos, c);
                state.cursor_pos += 1;
            }
        }
        KeyCode::Esc => {
            state.search_active = false;
            state.active_popup = None;
        }
        _ => {}
    }

    false
}

fn handle_search_key(state: &mut AppState, key: KeyEvent) -> bool {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Esc => {
            state.search_active = false;
            state.search_query.clear();
            state.search_state = search::SearchState::default();
        }
        KeyCode::Enter => {
            state.search_state.next_match();
            if let Some(idx) = state.search_state.current_line_index() {
                state.scroll_offset = idx;
                state.scroll_locked = true;
            }
        }
        KeyCode::Char('n') if !ctrl => {
            state.search_state.next_match();
            if let Some(idx) = state.search_state.current_line_index() {
                state.scroll_offset = idx;
                state.scroll_locked = true;
            }
        }
        KeyCode::Char('N') => {
            state.search_state.prev_match();
            if let Some(idx) = state.search_state.current_line_index() {
                state.scroll_offset = idx;
                state.scroll_locked = true;
            }
        }
        KeyCode::Char('f') if ctrl => {
            state.filter_mode = !state.filter_mode;
        }
        KeyCode::Backspace => {
            state.search_query.pop();
            let query = state.search_query.clone();
            state.search_state.set_query(&query);
            let texts: Vec<String> = state.log_lines.iter().map(|l| l.text.clone()).collect();
            state.search_state.update_matches(&texts);
        }
        KeyCode::Char(c) if !ctrl => {
            state.search_query.push(c);
            let query = state.search_query.clone();
            state.search_state.set_query(&query);
            let texts: Vec<String> = state.log_lines.iter().map(|l| l.text.clone()).collect();
            state.search_state.update_matches(&texts);
        }
        _ => {}
    }

    false
}

fn handle_popup_key(
    state: &mut AppState,
    key: KeyEvent,
    popup: PopupKind,
    serial_tx: mpsc::Sender<SerialEvent>,
) -> bool {
    match popup {
        PopupKind::Help => {
            state.active_popup = None;
        }
        PopupKind::PortSelect => match key.code {
            KeyCode::Esc => {
                state.active_popup = None;
            }
            KeyCode::Up => {
                if state.port_select_index > 0 {
                    state.port_select_index -= 1;
                }
            }
            KeyCode::Down => {
                if !state.available_ports.is_empty()
                    && state.port_select_index < state.available_ports.len() - 1
                {
                    state.port_select_index += 1;
                }
            }
            KeyCode::Char('r') => {
                state.scan_ports();
                state.port_select_index = 0;
            }
            KeyCode::Enter => {
                if !state.available_ports.is_empty() {
                    let port = state.available_ports[state.port_select_index].clone();
                    state.port_path = Some(port);
                    state.active_popup = None;
                    connect_port(state, serial_tx);
                }
            }
            _ => {}
        },
        PopupKind::Settings => match key.code {
            KeyCode::Esc => {
                state.active_popup = None;
            }
            KeyCode::Up => {
                if state.settings_field_index > 0 {
                    state.settings_field_index -= 1;
                }
            }
            KeyCode::Down => {
                if state.settings_field_index < 13 {
                    state.settings_field_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                cycle_settings_field(state, false);
            }
            KeyCode::Left => {
                cycle_settings_field(state, true);
            }
            _ => {}
        },
        PopupKind::Macros => match key.code {
            KeyCode::Esc => {
                state.active_popup = None;
            }
            KeyCode::Up => {
                if state.macro_select_index > 0 {
                    state.macro_select_index -= 1;
                }
            }
            KeyCode::Down => {
                if !state.macros.is_empty()
                    && state.macro_select_index < state.macros.len() - 1
                {
                    state.macro_select_index += 1;
                }
            }
            KeyCode::Enter => {
                if !state.macros.is_empty() {
                    let macro_idx = state.macro_select_index;
                    state.active_popup = None;
                    execute_macro(state, macro_idx);
                }
            }
            _ => {}
        },
    }

    false
}

fn cycle_settings_field(state: &mut AppState, reverse: bool) {
    let baud_rates = [
        300u32, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
    ];
    match state.settings_field_index {
        0 => {
            let current = state.serial_config.baud_rate;
            let pos = baud_rates.iter().position(|&b| b == current).unwrap_or(8);
            let next_pos = if reverse {
                if pos == 0 { baud_rates.len() - 1 } else { pos - 1 }
            } else {
                (pos + 1) % baud_rates.len()
            };
            state.serial_config.baud_rate = baud_rates[next_pos];
        }
        1 => {
            let bits = [5u8, 6, 7, 8];
            let pos = bits
                .iter()
                .position(|&b| b == state.serial_config.data_bits)
                .unwrap_or(3);
            let next = if reverse {
                if pos == 0 { 3 } else { pos - 1 }
            } else {
                (pos + 1) % 4
            };
            state.serial_config.data_bits = bits[next];
        }
        2 => {
            let parities = ["none", "odd", "even"];
            let pos = parities
                .iter()
                .position(|&p| p == state.serial_config.parity.to_lowercase())
                .unwrap_or(0);
            let next = if reverse {
                if pos == 0 { 2 } else { pos - 1 }
            } else {
                (pos + 1) % 3
            };
            state.serial_config.parity = parities[next].to_string();
        }
        3 => {
            state.serial_config.stop_bits = if state.serial_config.stop_bits == 1 { 2 } else { 1 };
        }
        4 => {
            let flows = ["none", "hardware", "software"];
            let pos = flows
                .iter()
                .position(|&f| f == state.serial_config.flow_control.to_lowercase())
                .unwrap_or(0);
            let next = if reverse {
                if pos == 0 { 2 } else { pos - 1 }
            } else {
                (pos + 1) % 3
            };
            state.serial_config.flow_control = flows[next].to_string();
        }
        5 => {
            state.serial_config.dtr = !state.serial_config.dtr;
            state.signals.dtr = state.serial_config.dtr;
            if let Some(sender) = &state.tx_sender {
                let _ = sender.try_send(TxCommand::SetDtr(state.serial_config.dtr));
            }
        }
        6 => {
            state.serial_config.rts = !state.serial_config.rts;
            state.signals.rts = state.serial_config.rts;
            if let Some(sender) = &state.tx_sender {
                let _ = sender.try_send(TxCommand::SetRts(state.serial_config.rts));
            }
        }
        8 => {
            state.tx_newline = state.tx_newline.next_send();
        }
        11 => {
            state.local_echo = !state.local_echo;
        }
        13 => {
            state.show_timestamp = !state.show_timestamp;
        }
        _ => {}
    }
}

fn send_input(state: &mut AppState) {
    if state.input.is_empty() {
        return;
    }

    let raw_input = state.input.clone();
    let mut data = crate::macros::expand_escapes(&raw_input);
    data.extend_from_slice(&state.tx_newline.suffix());

    if let Some(sender) = &state.tx_sender {
        let _ = sender.try_send(TxCommand::Send(data.clone()));
    } else {
        state.set_status("Not connected. Press F2 to select port.", true);
    }

    if state.local_echo {
        let tx_line = LogLine::new_tx(data.clone());
        if state.logging {
            if let Some(logger) = &mut state.file_logger {
                let _ = logger.log_tx(&data);
            }
        }
        state.add_log_line(tx_line);
    }

    state.send_history.push(raw_input);
    state.input.clear();
    state.cursor_pos = 0;
}

fn toggle_logging(state: &mut AppState) {
    if state.logging {
        state.logging = false;
        state.file_logger = None;
        state.set_status("Logging stopped.", false);
    } else {
        let path = default_log_path();
        let format = state.log_format.clone();
        match FileLogger::new(path.clone(), format) {
            Ok(logger) => {
                state.file_logger = Some(logger);
                state.logging = true;
                state.set_status(format!("Logging to {:?}", path), false);
            }
            Err(e) => {
                state.set_status(format!("Failed to open log: {}", e), true);
            }
        }
    }
}

fn copy_log_to_clipboard(state: &mut AppState) {
    let text: String = state
        .log_lines
        .iter()
        .map(|l| {
            let dir = match l.direction {
                Direction::Rx => ">",
                Direction::Tx => "<",
                Direction::System => "*",
            };
            let ts = l.timestamp.format("%H:%M:%S%.3f");
            format!("[{}] {} {}\n", ts, dir, l.text)
        })
        .collect();

    match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.set_text(text) {
            Ok(_) => state.set_status("Log copied to clipboard.", false),
            Err(e) => state.set_status(format!("Clipboard error: {}", e), true),
        },
        Err(e) => state.set_status(format!("Clipboard error: {}", e), true),
    }
}

fn execute_macro(state: &mut AppState, macro_idx: usize) {
    if macro_idx >= state.macros.len() {
        return;
    }

    let m = state.macros[macro_idx].clone();
    let tx_sender = state.tx_sender.clone();

    if tx_sender.is_none() {
        state.set_status("Not connected.", true);
        return;
    }

    let sender = tx_sender.unwrap();

    tokio::spawn(async move {
        for step in &m.steps {
            let data = crate::macros::expand_escapes(&step.send);
            let _ = sender.send(TxCommand::Send(data)).await;
            if step.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(step.delay_ms)).await;
            }
        }
    });

    let msg = format!("Macro '{}' started.", m.name);
    state.add_log_line(LogLine::new_system(msg.clone()));
    state.set_status(msg, false);
}
