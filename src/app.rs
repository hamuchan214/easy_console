use chrono::{DateTime, Local};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use crate::serial::{SerialConfig, TxCommand, ControlSignals};
use crate::macros::Macro;
use crate::logger::{FileLogger, LogFormat};
use crate::input::InputHistory;
use crate::search::SearchState;

#[derive(Debug, Clone, PartialEq)]
pub enum NewlineMode {
    CrLf,
    Lf,
    Cr,
    None_,
    Auto,
}

impl NewlineMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            NewlineMode::CrLf => "CRLF",
            NewlineMode::Lf => "LF",
            NewlineMode::Cr => "CR",
            NewlineMode::None_ => "NONE",
            NewlineMode::Auto => "AUTO",
        }
    }

    pub fn next_send(&self) -> Self {
        match self {
            NewlineMode::CrLf => NewlineMode::Lf,
            NewlineMode::Lf => NewlineMode::Cr,
            NewlineMode::Cr => NewlineMode::None_,
            NewlineMode::None_ => NewlineMode::CrLf,
            NewlineMode::Auto => NewlineMode::CrLf,
        }
    }

    pub fn suffix(&self) -> Vec<u8> {
        match self {
            NewlineMode::CrLf => vec![b'\r', b'\n'],
            NewlineMode::Lf => vec![b'\n'],
            NewlineMode::Cr => vec![b'\r'],
            NewlineMode::None_ => vec![],
            NewlineMode::Auto => vec![b'\r', b'\n'],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Ascii,
    Hex,
    Split,
}

impl ViewMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::Ascii => "ASCII",
            ViewMode::Hex => "HEX",
            ViewMode::Split => "SPLIT",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ViewMode::Ascii => ViewMode::Hex,
            ViewMode::Hex => ViewMode::Split,
            ViewMode::Split => ViewMode::Ascii,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Rx,
    Tx,
    System,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub direction: Direction,
    pub timestamp: DateTime<Local>,
    pub raw: Vec<u8>,
    pub text: String,
}

impl LogLine {
    pub fn new_rx(raw: Vec<u8>) -> Self {
        let text = String::from_utf8_lossy(&raw)
            .chars()
            .map(|c| {
                if c == '\r' || c == '\n' {
                    '↵'
                } else if (c as u32) < 0x20 && c != '\t' {
                    '·'
                } else {
                    c
                }
            })
            .collect();
        Self {
            direction: Direction::Rx,
            timestamp: Local::now(),
            raw,
            text,
        }
    }

    pub fn new_tx(raw: Vec<u8>) -> Self {
        let text = String::from_utf8_lossy(&raw).to_string();
        Self {
            direction: Direction::Tx,
            timestamp: Local::now(),
            raw,
            text,
        }
    }

    pub fn new_system(msg: String) -> Self {
        Self {
            direction: Direction::System,
            timestamp: Local::now(),
            raw: msg.as_bytes().to_vec(),
            text: msg,
        }
    }

    #[allow(dead_code)]
    pub fn hex_dump(&self) -> String {
        let ts = self.timestamp.format("%H:%M:%S%.3f");
        let dir = match self.direction {
            Direction::Rx => "RX",
            Direction::Tx => "TX",
            Direction::System => "SY",
        };
        let chunks: Vec<String> = self.raw.chunks(16).map(|chunk| {
            let hex: String = chunk.iter().map(|b| format!("{:02X} ", b)).collect();
            let ascii: String = chunk.iter().map(|&b| {
                if b >= 0x20 && b < 0x7f { b as char } else { '.' }
            }).collect();
            format!("[{}] {}  {:<48} {}", ts, dir, hex.trim_end(), ascii)
        }).collect();
        chunks.join("\n")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub error_count: u64,
    pub connected_at: Option<DateTime<Local>>,
    pub last_rx_at: Option<DateTime<Local>>,
    pub rx_rate_bps: f64,
    pub buffer_lines: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupKind {
    Help,
    PortSelect,
    Settings,
    Macros,
}

pub struct AppState {
    pub port_path: Option<String>,
    pub serial_config: SerialConfig,
    pub log_lines: Vec<LogLine>,
    pub input: String,
    pub cursor_pos: usize,
    pub tx_newline: NewlineMode,
    pub rx_newline: NewlineMode,
    pub view_mode: ViewMode,
    pub show_stats: bool,
    pub logging: bool,
    pub local_echo: bool,
    pub raw_mode_input: bool,
    pub scroll_offset: usize,
    pub scroll_locked: bool,
    pub active_popup: Option<PopupKind>,
    pub search_query: String,
    pub search_active: bool,
    pub filter_mode: bool,
    pub send_history: InputHistory,
    pub stats: Stats,
    pub signals: ControlSignals,
    pub macros: Vec<Macro>,
    pub scroll_buffer_size: usize,
    pub show_timestamp: bool,

    // Search state
    pub search_state: SearchState,

    // Port select popup state
    pub available_ports: Vec<String>,
    pub port_select_index: usize,

    // Settings popup state
    pub settings_field_index: usize,

    // Macro popup state
    pub macro_select_index: usize,

    // Serial channels
    pub tx_sender: Option<mpsc::Sender<TxCommand>>,
    pub reader_cancel: Option<Arc<AtomicBool>>,
    pub connected: bool,
    pub status_message: String,
    pub status_is_error: bool,

    // File logger
    pub file_logger: Option<FileLogger>,
    pub log_format: LogFormat,

    // Stats tracking
    pub rx_bytes_last_sec: u64,
    pub rx_bytes_sec_start: std::time::Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            port_path: None,
            serial_config: SerialConfig::default(),
            log_lines: Vec::new(),
            input: String::new(),
            cursor_pos: 0,
            tx_newline: NewlineMode::CrLf,
            rx_newline: NewlineMode::Auto,
            view_mode: ViewMode::Ascii,
            show_stats: false,
            logging: false,
            local_echo: true,
            raw_mode_input: false,
            scroll_offset: 0,
            scroll_locked: false,
            active_popup: None,
            search_query: String::new(),
            search_active: false,
            filter_mode: false,
            send_history: InputHistory::new(200),
            stats: Stats::default(),
            signals: ControlSignals::default(),
            macros: Vec::new(),
            scroll_buffer_size: 10000,
            show_timestamp: true,
            search_state: SearchState::default(),
            available_ports: Vec::new(),
            port_select_index: 0,
            settings_field_index: 0,
            macro_select_index: 0,
            tx_sender: None,
            reader_cancel: None,
            connected: false,
            status_message: "Ready. Press F2 to select port.".to_string(),
            status_is_error: false,
            file_logger: None,
            log_format: LogFormat::Text,
            rx_bytes_last_sec: 0,
            rx_bytes_sec_start: std::time::Instant::now(),
        }
    }

    pub fn add_log_line(&mut self, line: LogLine) {
        // Update stats
        match line.direction {
            Direction::Rx => {
                self.stats.rx_bytes += line.raw.len() as u64;
                self.stats.last_rx_at = Some(Local::now());
                self.rx_bytes_last_sec += line.raw.len() as u64;
                let elapsed = self.rx_bytes_sec_start.elapsed().as_secs_f64();
                if elapsed >= 1.0 {
                    self.stats.rx_rate_bps = self.rx_bytes_last_sec as f64 / elapsed;
                    self.rx_bytes_last_sec = 0;
                    self.rx_bytes_sec_start = std::time::Instant::now();
                }
            }
            Direction::Tx => {
                self.stats.tx_bytes += line.raw.len() as u64;
            }
            Direction::System => {}
        }

        self.log_lines.push(line);

        // Trim buffer
        while self.log_lines.len() > self.scroll_buffer_size {
            self.log_lines.remove(0);
            if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
            }
        }

        self.stats.buffer_lines = self.log_lines.len();

        // Auto-scroll to bottom if not locked
        if !self.scroll_locked {
            self.scroll_to_bottom();
        }

        // Update search
        self.update_search();
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.log_lines.is_empty() {
            self.scroll_offset = 0;
        } else {
            self.scroll_offset = self.log_lines.len().saturating_sub(1);
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        self.scroll_locked = true;
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.log_lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + n).min(max);
        if self.scroll_offset >= max {
            self.scroll_locked = false;
        }
    }

    pub fn clear_log(&mut self) {
        self.log_lines.clear();
        self.scroll_offset = 0;
        self.scroll_locked = false;
        self.stats.buffer_lines = 0;
        if let Some(logger) = &mut self.file_logger {
            let _ = logger.log_cleared();
        }
    }

    pub fn update_search(&mut self) {
        if self.search_active {
            let texts: Vec<String> = self.log_lines.iter().map(|l| l.text.clone()).collect();
            self.search_state.update_matches(&texts);
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = msg.into();
        self.status_is_error = is_error;
    }

    pub fn scan_ports(&mut self) {
        self.available_ports = serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect();
        self.available_ports.sort();
    }

    pub fn disconnect(&mut self) {
        if let Some(sender) = self.tx_sender.take() {
            let _ = sender.try_send(TxCommand::Close);
        }
        self.connected = false;
        self.stats.connected_at = None;
        self.add_log_line(LogLine::new_system("Disconnected.".to_string()));
        self.set_status("Disconnected.", false);
    }

    pub fn uptime_str(&self) -> String {
        match self.stats.connected_at {
            None => "N/A".to_string(),
            Some(t) => {
                let elapsed = (Local::now() - t).num_seconds();
                let h = elapsed / 3600;
                let m = (elapsed % 3600) / 60;
                let s = elapsed % 60;
                format!("{:02}:{:02}:{:02}", h, m, s)
            }
        }
    }
}
