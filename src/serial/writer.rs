use std::io::Write;
use tokio::sync::mpsc;
use super::TxCommand;
use anyhow::Result;

pub fn start_writer(
    port: Box<dyn serialport::SerialPort>,
    mut rx: mpsc::Receiver<TxCommand>,
) -> Result<()> {
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
