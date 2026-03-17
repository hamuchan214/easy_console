pub mod reader;
pub mod writer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
    pub flow_control: String,
    pub dtr: bool,
    pub rts: bool,
    pub timeout_ms: u64,
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            data_bits: 8,
            parity: "none".to_string(),
            stop_bits: 1,
            flow_control: "none".to_string(),
            dtr: false,
            rts: false,
            timeout_ms: 100,
        }
    }
}

impl SerialConfig {
    pub fn to_serialport_data_bits(&self) -> serialport::DataBits {
        match self.data_bits {
            5 => serialport::DataBits::Five,
            6 => serialport::DataBits::Six,
            7 => serialport::DataBits::Seven,
            _ => serialport::DataBits::Eight,
        }
    }

    pub fn to_serialport_parity(&self) -> serialport::Parity {
        match self.parity.to_lowercase().as_str() {
            "odd" => serialport::Parity::Odd,
            "even" => serialport::Parity::Even,
            _ => serialport::Parity::None,
        }
    }

    pub fn to_serialport_stop_bits(&self) -> serialport::StopBits {
        match self.stop_bits {
            2 => serialport::StopBits::Two,
            _ => serialport::StopBits::One,
        }
    }

    pub fn to_serialport_flow_control(&self) -> serialport::FlowControl {
        match self.flow_control.to_lowercase().as_str() {
            "hardware" => serialport::FlowControl::Hardware,
            "software" => serialport::FlowControl::Software,
            _ => serialport::FlowControl::None,
        }
    }

    pub fn description(&self) -> String {
        let parity = match self.parity.to_lowercase().as_str() {
            "odd" => "O",
            "even" => "E",
            _ => "N",
        };
        format!("{} {}{}{}",
            self.baud_rate,
            self.data_bits,
            parity,
            self.stop_bits
        )
    }
}

#[derive(Debug, Clone)]
pub struct ControlSignals {
    pub dtr: bool,
    pub rts: bool,
    pub cts: bool,
    pub dsr: bool,
    pub dcd: bool,
    pub ri: bool,
}

impl Default for ControlSignals {
    fn default() -> Self {
        Self {
            dtr: false,
            rts: false,
            cts: false,
            dsr: false,
            dcd: false,
            ri: false,
        }
    }
}

#[derive(Debug)]
pub enum SerialEvent {
    Data(Vec<u8>),
    Error(String),
    Disconnected,
}

#[derive(Debug)]
pub enum TxCommand {
    Send(Vec<u8>),
    SetDtr(bool),
    SetRts(bool),
    Close,
}
