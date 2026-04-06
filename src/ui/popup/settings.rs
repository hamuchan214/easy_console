use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Settings (↑↓=Navigate, ←/Enter/→=Change, Esc=Close) ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let cfg = &state.serial_config;
    let fields = vec![
        ("Baud Rate", cfg.baud_rate.to_string()),
        ("Data Bits", cfg.data_bits.to_string()),
        ("Parity", cfg.parity.clone()),
        ("Stop Bits", cfg.stop_bits.to_string()),
        ("Flow Control", cfg.flow_control.clone()),
        ("DTR", if cfg.dtr { "ON".to_string() } else { "OFF".to_string() }),
        ("RTS", if cfg.rts { "ON".to_string() } else { "OFF".to_string() }),
        ("Timeout (ms)", cfg.timeout_ms.to_string()),
        ("TX Newline", state.tx_newline.as_str().to_string()),
        ("RX Newline", state.rx_newline.as_str().to_string()),
        ("View Mode", state.view_mode.as_str().to_string()),
        ("Local Echo", if state.local_echo { "ON".to_string() } else { "OFF".to_string() }),
        ("Scroll Buffer", state.scroll_buffer_size.to_string()),
        ("Show Timestamp", if state.show_timestamp { "ON".to_string() } else { "OFF".to_string() }),
    ];

    let lines: Vec<Line> = fields.iter().enumerate().map(|(i, (name, val))| {
        let style = if i == state.settings_field_index {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        Line::from(Span::styled(format!("  {:<20} {}", name, val), style))
    }).collect();

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(lines), inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
