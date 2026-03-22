use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let s = &state.stats;
    let sig = &state.signals;

    let last_rx = match s.last_rx_at {
        None => "N/A".to_string(),
        Some(t) => {
            let d = (chrono::Local::now() - t).num_milliseconds();
            format!("{:.1}s ago", d as f64 / 1000.0)
        }
    };

    let line1 = format!(
        " RX: {:>10} bytes  TX: {:>8} bytes  Errors: {:>4}  Uptime: {}",
        s.rx_bytes,
        s.tx_bytes,
        s.error_count,
        state.uptime_str()
    );

    let line2 = format!(
        " RX Rate: {:>8.1} B/s  Last RX: {:>12}  Buffer: {:>6} lines",
        s.rx_rate_bps,
        last_rx,
        s.buffer_lines
    );

    let sig_str = |v: bool| if v { "ON " } else { "OFF" };
    let line3 = format!(
        " DTR: {}  RTS: {}  CTS: {}  DSR: {}  DCD: {}  RI: {}",
        sig_str(sig.dtr),
        sig_str(sig.rts),
        sig_str(sig.cts),
        sig_str(sig.dsr),
        sig_str(sig.dcd),
        sig_str(sig.ri),
    );

    let text = vec![
        Line::from(Span::styled(line1, Style::default().fg(Color::White))),
        Line::from(Span::styled(line2, Style::default().fg(Color::White))),
        Line::from(Span::styled(line3, Style::default().fg(Color::Cyan))),
    ];

    let block = Block::default()
        .title(" Statistics ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}
