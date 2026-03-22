use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};
use crate::app::AppState;

pub fn render(f: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Macros (Enter=Run, Esc=Cancel) ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let items: Vec<ListItem> = if state.macros.is_empty() {
        vec![ListItem::new(Span::styled(
            "No macros defined",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state.macros.iter().map(|m| {
            ListItem::new(Line::from(vec![
                Span::styled(&m.name, Style::default().fg(Color::White)),
                Span::styled(
                    format!(" ({} steps)", m.steps.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        }).collect()
    };

    let mut list_state = ListState::default();
    if !state.macros.is_empty() {
        list_state.select(Some(state.macro_select_index));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut list_state);
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
