use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::LogState;
use crate::ui::theme;

fn highlight_text(text: &str, search: &str) -> Line<'static> {
    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut spans: Vec<Span> = Vec::new();
    let mut last_end: usize = 0;

    for (i, _) in text_lower.char_indices() {
        if text_lower[i..].starts_with(&search_lower) {
            if last_end < i {
                spans.push(Span::styled(
                    text[last_end..i].to_string(),
                    Style::default().fg(Color::Yellow),
                ));
            }

            spans.push(Span::styled(
                text[i..i + search.len()].to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));

            last_end = i + search.len();
        }
    }

    if last_end < text.len() {
        spans.push(Span::styled(
            text[last_end..].to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

fn ansi_color(n: u8) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        _ => Color::Reset,
    }
}

fn ansi_bright_color(n: u8) -> Color {
    match n {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        7 => Color::White,
        _ => Color::Reset,
    }
}

fn parse_ansi(message: &str) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = Vec::new();
    let mut style = Style::default().fg(Color::White);
    let mut buf = String::new();
    let mut chars = message.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            if !buf.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buf), style));
            }
            chars.next();

            let mut code_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_digit() || ch == ';' {
                    code_str.push(ch);
                    chars.next();
                } else {
                    break;
                }
            }
            chars.next();

            for part in code_str.split(';') {
                if let Ok(n) = part.parse::<u8>() {
                    match n {
                        0 => style = Style::default().fg(Color::White),
                        1 => style = style.add_modifier(Modifier::BOLD),
                        3 => style = style.add_modifier(Modifier::ITALIC),
                        4 => style = style.add_modifier(Modifier::UNDERLINED),
                        7 => style = style.add_modifier(Modifier::REVERSED),
                        30..=37 => style = style.fg(ansi_color(n - 30)),
                        40..=47 => style = style.bg(ansi_color(n - 40)),
                        90..=97 => style = style.fg(ansi_bright_color(n - 90)),
                        _ => {}
                    }
                }
            }
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        spans.push(Span::styled(buf, style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            message.to_string(),
            Style::default().fg(Color::White),
        ));
    }

    spans
}

pub fn render(frame: &mut Frame, area: Rect, state: &mut LogState) {
    let search_label = if !state.search.is_empty() {
        format!(" SEARCH '{}'", state.search)
    } else {
        String::new()
    };

    let title = if state.paused {
        format!(" LOGS — {} (PAUSED{}) ", state.container_id, search_label)
    } else {
        let mode = if state.tail { "LIVE · FOLLOW" } else { "SCROLL" };
        format!(" LOGS — {} ({}{}) ", state.container_id, mode, search_label)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::view_border()));

    let inner = block.inner(area);

    if state.buffer.is_empty() {
        let text = Text::from(vec![
            Line::from(Span::styled("  Waiting for logs...", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let has_filter = !state.search.is_empty();
    let height = inner.height as usize;

    state.viewport_height = height;
    let max_offset = state.buffer.len().saturating_sub(height);
    state.scroll_offset = state.scroll_offset.min(max_offset);
    let start = state.buffer.len().saturating_sub(height + state.scroll_offset);
    let show_ts = state.show_timestamps;
    let lines: Vec<Line> = state
        .buffer
        .iter()
        .skip(start)
        .take(height)
        .map(|entry| {
            let message = entry.message.trim_end().to_string();

            if show_ts && !entry.timestamp.is_empty() {
                let display_text = format!("{} {}", entry.timestamp, message);
                if has_filter && display_text.to_lowercase().contains(&state.search.to_lowercase()) {
                    highlight_text(&display_text, &state.search)
                } else {
                    let ts = entry.timestamp.clone();
                    let mut spans = vec![
                        Span::styled(ts, Style::default().fg(Color::DarkGray)),
                    ];
                    spans.extend(parse_ansi(&message));
                    Line::from(spans)
                }
            } else if has_filter && message.to_lowercase().contains(&state.search.to_lowercase()) {
                highlight_text(&message, &state.search)
            } else {
                Line::from(parse_ansi(&message))
            }
        })
        .collect();

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);

    if state.search_active {
        render_search_bar(frame, inner, &state.search);
    }
}

fn render_search_bar(frame: &mut Frame, area: Rect, search: &str) {
    let search_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2).min(40),
        height: 1,
    };
    let display = if search.is_empty() {
        "/  search..."
    } else {
        search
    };
    frame.render_widget(
        Paragraph::new(format!("/{}", display))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        search_area,
    );
}
