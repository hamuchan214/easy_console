pub mod layout;
pub mod log_view;
pub mod input_bar;
pub mod status_bar;
pub mod stats_panel;
pub mod popup;

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{AppState, PopupKind};
use layout::compute_layout;

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let layout = compute_layout(area, state.show_stats, state.search_active);

    // Status bar
    status_bar::render(f, layout.status_bar, state);

    // Log view
    log_view::render(f, layout.log_view, state);

    // Stats panel
    if let Some(stats_area) = layout.stats_panel {
        stats_panel::render(f, stats_area, state);
    }

    // Search bar
    if let Some(search_area) = layout.search_bar {
        render_search_bar(f, search_area, state);
    }

    // Input bar
    input_bar::render(f, layout.input_bar, state);

    // Hint bar
    render_hint_bar(f, layout.hint_bar, state);

    // Popup overlays
    match &state.active_popup {
        Some(PopupKind::Help) => popup::help::render(f),
        Some(PopupKind::PortSelect) => popup::port_select::render(f, state),
        Some(PopupKind::Settings) => popup::settings::render(f, state),
        Some(PopupKind::Macros) => popup::macros::render(f, state),
        None => {}
    }
}

fn render_search_bar(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let match_count = state.search_state.matches.len();
    let current = state.search_state.current_match
        .map(|i| (i + 1).to_string())
        .unwrap_or_else(|| "0".to_string());

    let text = format!(
        " Search: {} [{}/{}] {}",
        state.search_query,
        current,
        match_count,
        if state.filter_mode { "[FILTER]" } else { "" }
    );

    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Yellow)),
        Span::raw(text),
    ]))
    .style(Style::default().bg(Color::DarkGray));
    f.render_widget(paragraph, area);
}

fn render_hint_bar(f: &mut Frame, area: ratatui::layout::Rect, _state: &AppState) {
    let hints = "[F1]Help [F2]Port [F3]Config [F4]View [F5]Log [F6]Macros [F7]Search [Ctrl+C]Quit";
    let paragraph = Paragraph::new(Line::from(Span::styled(
        hints,
        Style::default().fg(Color::DarkGray),
    )))
    .style(Style::default().bg(Color::Black));
    f.render_widget(paragraph, area);
}
