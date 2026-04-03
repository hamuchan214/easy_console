use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{AppState, Direction as Dir, LogLine, ViewMode};

fn line_color(dir: &Dir) -> Color {
    match dir {
        Dir::Rx => Color::White,
        Dir::Tx => Color::Cyan,
        Dir::System => Color::Yellow,
    }
}

fn format_ascii_line(line: &LogLine, show_timestamp: bool, is_match: bool) -> Line<'static> {
    let ts = if show_timestamp {
        format!("[{}] ", line.timestamp.format("%H:%M:%S%.3f"))
    } else {
        String::new()
    };
    let dir_str = match line.direction {
        Dir::Rx => "> ",
        Dir::Tx => "< ",
        Dir::System => "* ",
    };
    let text = line.text.replace('\n', "↵").replace('\r', "↵");
    let full = format!("{}{}{}", ts, dir_str, text);

    let color = line_color(&line.direction);
    let style = if is_match {
        Style::default().fg(Color::Black).bg(Color::Magenta)
    } else {
        Style::default().fg(color)
    };

    Line::from(Span::styled(full, style))
}

fn format_hex_lines(line: &LogLine, show_timestamp: bool, is_match: bool) -> Vec<Line<'static>> {
    let ts = if show_timestamp {
        format!("[{}] ", line.timestamp.format("%H:%M:%S%.3f"))
    } else {
        String::new()
    };
    let dir_str = match line.direction {
        Dir::Rx => "RX",
        Dir::Tx => "TX",
        Dir::System => "SY",
    };
    let color = line_color(&line.direction);
    let style = if is_match {
        Style::default().fg(Color::Black).bg(Color::Magenta)
    } else {
        Style::default().fg(color)
    };

    if line.raw.is_empty() {
        let s = format!("{}{}", ts, dir_str);
        return vec![Line::from(Span::styled(s, style))];
    }

    line.raw.chunks(16).map(|chunk| {
        let hex: String = chunk.iter().map(|b| format!("{:02X} ", b)).collect();
        let ascii: String = chunk.iter().map(|&b| {
            if b >= 0x20 && b < 0x7f { b as char } else { '.' }
        }).collect();
        let s = format!("{}{}  {:<48} {}", ts, dir_str, hex.trim_end(), ascii);
        Line::from(Span::styled(s, style))
    }).collect()
}

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    match state.view_mode {
        ViewMode::Ascii => render_ascii(f, area, state),
        ViewMode::Hex => render_hex(f, area, state),
        ViewMode::Split => render_split(f, area, state),
    }
}

fn visible_lines<'a>(state: &'a AppState) -> Vec<(usize, &'a LogLine)> {
    if state.filter_mode && state.search_active {
        state.log_lines.iter().enumerate()
            .filter(|(i, _)| state.search_state.is_match(*i))
            .collect()
    } else {
        state.log_lines.iter().enumerate().collect()
    }
}

fn render_ascii(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(format!(" Log [{}] ", state.view_mode.as_str()))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines_data = visible_lines(state);
    let total = lines_data.len();
    let height = inner.height as usize;

    let start = if total > height {
        let offset = state.scroll_offset.min(total.saturating_sub(1));
        let half = height / 2;
        if offset < half {
            0
        } else if offset + half >= total {
            total.saturating_sub(height)
        } else {
            offset.saturating_sub(half)
        }
    } else {
        0
    };

    let visible: Vec<Line> = lines_data.iter().skip(start).take(height)
        .map(|(orig_i, line)| {
            let is_match = state.search_active && state.search_state.is_match(*orig_i);
            format_ascii_line(line, state.show_timestamp, is_match)
        })
        .collect();

    let paragraph = Paragraph::new(visible)
        .style(Style::default().bg(Color::Black));
    f.render_widget(paragraph, inner);
}

fn render_hex(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(format!(" Log [{}] ", state.view_mode.as_str()))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines_data = visible_lines(state);
    let height = inner.height as usize;

    // Flatten all hex lines
    let mut all_lines: Vec<Line> = Vec::new();
    for (orig_i, line) in &lines_data {
        let is_match = state.search_active && state.search_state.is_match(*orig_i);
        let hex_lines = format_hex_lines(line, state.show_timestamp, is_match);
        all_lines.extend(hex_lines);
    }

    let total = all_lines.len();
    let start = if total > height {
        total.saturating_sub(height)
    } else {
        0
    };
    let visible: Vec<Line> = all_lines.into_iter().skip(start).take(height).collect();

    let paragraph = Paragraph::new(visible)
        .style(Style::default().bg(Color::Black));
    f.render_widget(paragraph, inner);
}

fn render_split(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(format!(" Log [{}] ", state.view_mode.as_str()))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let lines_data = visible_lines(state);
    let height = chunks[0].height as usize;
    let total = lines_data.len();

    let start = if total > height {
        let offset = state.scroll_offset.min(total.saturating_sub(1));
        let half = height / 2;
        if offset < half {
            0
        } else if offset + half >= total {
            total.saturating_sub(height)
        } else {
            offset.saturating_sub(half)
        }
    } else {
        0
    };

    let slice: Vec<_> = lines_data.iter().skip(start).take(height).collect();

    let ascii_lines: Vec<Line> = slice.iter()
        .map(|(orig_i, line)| {
            let is_match = state.search_active && state.search_state.is_match(*orig_i);
            format_ascii_line(line, state.show_timestamp, is_match)
        })
        .collect();

    let hex_lines: Vec<Line> = slice.iter()
        .map(|(orig_i, line)| {
            let is_match = state.search_active && state.search_state.is_match(*orig_i);
            let hex: String = line.raw.iter().map(|b| format!("{:02X} ", b)).collect();
            let color = line_color(&line.direction);
            let style = if is_match {
                Style::default().fg(Color::Black).bg(Color::Magenta)
            } else {
                Style::default().fg(color)
            };
            Line::from(Span::styled(hex.trim().to_string(), style))
        })
        .collect();

    // Headers
    let ascii_header = Block::default()
        .title(" ASCII ")
        .borders(Borders::RIGHT)
        .style(Style::default().add_modifier(Modifier::BOLD));
    let hex_header = Block::default()
        .title(" HEX ");

    let ascii_inner = ascii_header.inner(chunks[0]);
    let hex_inner = hex_header.inner(chunks[1]);

    f.render_widget(ascii_header, chunks[0]);
    f.render_widget(hex_header, chunks[1]);

    f.render_widget(
        Paragraph::new(ascii_lines).style(Style::default().bg(Color::Black)),
        ascii_inner,
    );
    f.render_widget(
        Paragraph::new(hex_lines).style(Style::default().bg(Color::Black)),
        hex_inner,
    );
}
