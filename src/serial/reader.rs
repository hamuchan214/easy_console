use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use super::SerialEvent;
use anyhow::Result;

pub fn start_reader(
    port: Box<dyn serialport::SerialPort>,
    tx: mpsc::Sender<SerialEvent>,
    cancel: Arc<AtomicBool>,
) -> Result<()> {
    let mut port = port;
    let mut buf = [0u8; 4096];

    loop {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
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
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let _ = tx.blocking_send(SerialEvent::Error(e.to_string()));
                let _ = tx.blocking_send(SerialEvent::Disconnected);
                break;
            }
        }
    }

    Ok(())
}
