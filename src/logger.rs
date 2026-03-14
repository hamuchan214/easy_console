use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    Text,
    Hex,
    Raw,
}

pub struct FileLogger {
    file: File,
    pub format: LogFormat,
    #[allow(dead_code)]
    pub path: PathBuf,
}

impl FileLogger {
    pub fn new(path: PathBuf, format: LogFormat) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open log file {:?}", path))?;
        Ok(Self { file, format, path })
    }

    pub fn log_rx(&mut self, data: &[u8]) -> Result<()> {
        match self.format {
            LogFormat::Text => {
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let text = String::from_utf8_lossy(data);
                writeln!(self.file, "[{}] RX: {}", ts, text)?;
            }
            LogFormat::Hex => {
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let hex: String = data.iter().map(|b| format!("{:02X} ", b)).collect();
                writeln!(self.file, "[{}] RX: {}", ts, hex.trim())?;
            }
            LogFormat::Raw => {
                self.file.write_all(data)?;
            }
        }
        self.file.flush()?;
        Ok(())
    }

    pub fn log_tx(&mut self, data: &[u8]) -> Result<()> {
        match self.format {
            LogFormat::Text => {
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let text = String::from_utf8_lossy(data);
                writeln!(self.file, "[{}] TX: {}", ts, text)?;
            }
            LogFormat::Hex => {
                let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let hex: String = data.iter().map(|b| format!("{:02X} ", b)).collect();
                writeln!(self.file, "[{}] TX: {}", ts, hex.trim())?;
            }
            LogFormat::Raw => {
                self.file.write_all(data)?;
            }
        }
        self.file.flush()?;
        Ok(())
    }

    pub fn log_cleared(&mut self) -> Result<()> {
        if self.format != LogFormat::Raw {
            let ts = Local::now().format("%H:%M:%S");
            writeln!(self.file, "--- [CLEARED at {}] ---", ts)?;
            self.file.flush()?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn log_system(&mut self, msg: &str) -> Result<()> {
        if self.format != LogFormat::Raw {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(self.file, "[{}] SYS: {}", ts, msg)?;
            self.file.flush()?;
        }
        Ok(())
    }
}

pub fn default_log_path() -> PathBuf {
    let ts = Local::now().format("%Y%m%d_%H%M%S");
    PathBuf::from(format!("easy_console_{}.log", ts))
}
