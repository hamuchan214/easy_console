use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let port_str = state.port_path.as_deref().unwrap_or("No Port");
    let conn_str = if state.connected { "Connected" } else { "Disconnected" };
    let conn_color = if state.connected { Color::Green } else { Color::Red };

    let rx_kb = state.stats.rx_bytes as f64 / 1024.0;
    let tx_b = state.stats.tx_bytes;
    let lock_str = if state.scroll_locked { " [LOCK]" } else { "" };
    let log_str = if state.logging { " [LOG]" } else { "" };

    let spans = vec![
        Span::styled("easy_console", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(port_str, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::raw(state.serial_config.description()),
        Span::raw("  "),
        Span::styled(state.tx_newline.as_str(), Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::raw(format!("[RX:{:.1}KB TX:{}B]", rx_kb, tx_b)),
        Span::styled(lock_str, Style::default().fg(Color::Magenta)),
        Span::styled(log_str, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", conn_str),
            Style::default().fg(conn_color).add_modifier(Modifier::BOLD),
        ),
    ];

    // Add status message
    let status_color = if state.status_is_error { Color::Red } else { Color::DarkGray };
    let mut all_spans = spans;
    all_spans.push(Span::raw("  "));
    all_spans.push(Span::styled(
        state.status_message.clone(),
        Style::default().fg(status_color),
    ));

    let paragraph = Paragraph::new(Line::from(all_spans))
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, area);
}
