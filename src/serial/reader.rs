use std::io::Read;
use tokio::sync::mpsc;
use super::{SerialEvent, SerialConfig};
use anyhow::Result;

pub fn start_reader(
    port_path: String,
    config: SerialConfig,
    tx: mpsc::Sender<SerialEvent>,
) -> Result<()> {
    let port = serialport::new(&port_path, config.baud_rate)
        .data_bits(config.to_serialport_data_bits())
        .parity(config.to_serialport_parity())
        .stop_bits(config.to_serialport_stop_bits())
        .flow_control(config.to_serialport_flow_control())
        .timeout(std::time::Duration::from_millis(config.timeout_ms))
        .open()?;

    let mut port = port;
    let mut buf = [0u8; 4096];

    loop {
        match port.read(&mut buf) {
            Ok(0) => {}
            Ok(n) => {
                let data = buf[..n].to_vec();
                if tx.blocking_send(SerialEvent::Data(data)).is_err() {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => {
                let _ = tx.blocking_send(SerialEvent::Error(e.to_string()));
                let _ = tx.blocking_send(SerialEvent::Disconnected);
                break;
            }
        }
    }

    Ok(())
}
