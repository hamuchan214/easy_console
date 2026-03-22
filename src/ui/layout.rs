use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub status_bar: Rect,
    pub log_view: Rect,
    pub stats_panel: Option<Rect>,
    pub search_bar: Option<Rect>,
    pub input_bar: Rect,
    pub hint_bar: Rect,
}

pub fn compute_layout(area: Rect, show_stats: bool, show_search: bool) -> AppLayout {
    let stats_height = if show_stats { 5 } else { 0 };
    let search_height = if show_search { 1 } else { 0 };

    let constraints = if show_stats && show_search {
        vec![
            Constraint::Length(1),           // status bar
            Constraint::Min(5),              // log view
            Constraint::Length(stats_height),
            Constraint::Length(search_height),
            Constraint::Length(1),           // input bar
            Constraint::Length(1),           // hint bar
        ]
    } else if show_stats {
        vec![
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(stats_height),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else if show_search {
        vec![
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(search_height),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else {
        vec![
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    if show_stats && show_search {
        AppLayout {
            status_bar: chunks[0],
            log_view: chunks[1],
            stats_panel: Some(chunks[2]),
            search_bar: Some(chunks[3]),
            input_bar: chunks[4],
            hint_bar: chunks[5],
        }
    } else if show_stats {
        AppLayout {
            status_bar: chunks[0],
            log_view: chunks[1],
            stats_panel: Some(chunks[2]),
            search_bar: None,
            input_bar: chunks[3],
            hint_bar: chunks[4],
        }
    } else if show_search {
        AppLayout {
            status_bar: chunks[0],
            log_view: chunks[1],
            stats_panel: None,
            search_bar: Some(chunks[2]),
            input_bar: chunks[3],
            hint_bar: chunks[4],
        }
    } else {
        AppLayout {
            status_bar: chunks[0],
            log_view: chunks[1],
            stats_panel: None,
            search_bar: None,
            input_bar: chunks[2],
            hint_bar: chunks[3],
        }
    }
}
