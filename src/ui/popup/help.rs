use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help - Key Bindings ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let help_text = vec![
        Line::from(Span::styled("Global", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("  F1          Help")),
        Line::from(Span::raw("  F2          Port select")),
        Line::from(Span::raw("  F3          Settings")),
        Line::from(Span::raw("  F4          View mode (ASCII/HEX/SPLIT)")),
        Line::from(Span::raw("  F5          Log file toggle")),
        Line::from(Span::raw("  F6          Macros")),
        Line::from(Span::raw("  F7 / /      Search")),
        Line::from(Span::raw("  Ctrl+C/Q    Quit")),
        Line::from(Span::raw("  Esc         Close popup / search")),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Log View", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("  Up/Down     Scroll 1 line")),
        Line::from(Span::raw("  PgUp/PgDn   Scroll 1 page")),
        Line::from(Span::raw("  Home/End    Jump to start/end")),
        Line::from(Span::raw("  Ctrl+L      Clear log buffer")),
        Line::from(Span::raw("  Ctrl+K      Clear display (keep buffer)")),
        Line::from(Span::raw("  Ctrl+X      Disconnect + clear")),
        Line::from(Span::raw("  Ctrl+Y      Copy log to clipboard")),
        Line::from(Span::raw("  Ctrl+I      Toggle stats panel")),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Input", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("  Enter       Send")),
        Line::from(Span::raw("  Up/Down     History navigation")),
        Line::from(Span::raw("  Ctrl+E      Cycle TX newline (CRLF/LF/CR/None)")),
        Line::from(Span::raw("  Ctrl+R      Toggle local echo")),
        Line::from(Span::raw("  Ctrl+W      Toggle raw mode")),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Search", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(Span::raw("  n           Next match")),
        Line::from(Span::raw("  N           Previous match")),
        Line::from(Span::raw("  Ctrl+F      Toggle filter mode")),
        Line::from(Span::raw("")),
        Line::from(Span::styled("Press Esc to close", Style::default().fg(Color::DarkGray))),
    ];

    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(Paragraph::new(help_text), inner);
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
