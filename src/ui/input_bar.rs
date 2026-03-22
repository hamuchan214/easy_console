use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let mode_str = if state.raw_mode_input { "RAW" } else { "TX" };
    let nl_str = state.tx_newline.as_str();
    let echo_str = if state.local_echo { "" } else { " [NO ECHO]" };

    let prefix = format!("{} {} > {}", mode_str, nl_str, echo_str);
    let input_text = &state.input;

    let spans = vec![
        Span::styled(
            format!("{} {} >", mode_str, nl_str),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(input_text.clone()),
    ];

    let _ = prefix;
    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black));
    f.render_widget(paragraph, area);

    // Show cursor
    let cursor_x = area.x + "TX CRLF > ".len() as u16 + state.cursor_pos as u16;
    let prefix_len = format!("{} {} > ", mode_str, nl_str).len() as u16;
    let cx = area.x + prefix_len + state.cursor_pos as u16;
    let cy = area.y;
    f.set_cursor_position((cx.min(area.x + area.width - 1), cy));
    let _ = cursor_x;
}
