use std::io::Write;
use tokio::sync::mpsc;
use super::{TxCommand, SerialConfig};
use anyhow::Result;

pub fn start_writer(
    port_path: String,
    config: SerialConfig,
    mut rx: mpsc::Receiver<TxCommand>,
) -> Result<()> {
    let port = serialport::new(&port_path, config.baud_rate)
        .data_bits(config.to_serialport_data_bits())
        .parity(config.to_serialport_parity())
        .stop_bits(config.to_serialport_stop_bits())
        .flow_control(config.to_serialport_flow_control())
        .timeout(std::time::Duration::from_millis(config.timeout_ms))
        .open()?;

    let mut port = port;

    loop {
        match rx.blocking_recv() {
            Some(TxCommand::Send(data)) => {
                let _ = port.write_all(&data);
                let _ = port.flush();
            }
            Some(TxCommand::SetDtr(v)) => {
                let _ = port.write_data_terminal_ready(v);
            }
            Some(TxCommand::SetRts(v)) => {
                let _ = port.write_request_to_send(v);
            }
            Some(TxCommand::Close) | None => {
                break;
            }
        }
    }

    Ok(())
}
