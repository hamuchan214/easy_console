use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStepConfig {
    pub send: String,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroConfig {
    pub name: String,
    pub steps: Vec<MacroStepConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub port: Option<String>,
    pub baud_rate: Option<u32>,
    pub data_bits: Option<u8>,
    pub parity: Option<String>,
    pub stop_bits: Option<u8>,
    pub flow_control: Option<String>,
    pub tx_newline: Option<String>,
    pub rx_newline: Option<String>,
    pub timestamp: Option<bool>,
    pub timestamp_precision: Option<String>,
    pub local_echo: Option<bool>,
    pub scroll_buffer: Option<usize>,
    pub log_format: Option<String>,
    pub view_mode: Option<String>,
    pub dtr_init: Option<bool>,
    pub rts_init: Option<bool>,
    pub macros: Option<Vec<MacroConfig>>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            port: None,
            baud_rate: Some(115200),
            data_bits: Some(8),
            parity: Some("none".to_string()),
            stop_bits: Some(1),
            flow_control: Some("none".to_string()),
            tx_newline: Some("crlf".to_string()),
            rx_newline: Some("auto".to_string()),
            timestamp: Some(true),
            timestamp_precision: Some("ms".to_string()),
            local_echo: Some(true),
            scroll_buffer: Some(10000),
            log_format: Some("text".to_string()),
            view_mode: Some("ascii".to_string()),
            dtr_init: Some(false),
            rts_init: Some(false),
            macros: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profiles {
    pub profiles: HashMap<String, Profile>,
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("easy_console")
}

pub fn profiles_path() -> PathBuf {
    config_dir().join("profiles.toml")
}

pub fn load_profiles() -> Result<Profiles> {
    let path = profiles_path();
    if !path.exists() {
        return Ok(Profiles::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read profiles from {:?}", path))?;
    let profiles: Profiles =
        toml::from_str(&content).with_context(|| "Failed to parse profiles.toml")?;
    Ok(profiles)
}

#[allow(dead_code)]
pub fn save_profiles(profiles: &Profiles) -> Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config dir {:?}", dir))?;
    let path = profiles_path();
    let content = toml::to_string_pretty(profiles)?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write profiles to {:?}", path))?;
    Ok(())
}
