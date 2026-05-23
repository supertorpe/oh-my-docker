use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::EventsState;

pub fn render(frame: &mut Frame, state: &EventsState) {
    let area = frame.area();

    let title = if !state.filter.is_empty() {
        format!(" DOCKER EVENTS FILTER '{}' ", state.filter)
    } else {
        " DOCKER EVENTS ".to_string()
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);

    if state.buffer.is_empty() {
        let text = Text::from(vec![
            Line::from(Span::styled("  Waiting for Docker events...", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  /  filter  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), frame.area());
        return;
    }

    let filter_active = !state.filter.is_empty();
    let lines: Vec<Line> = state
        .buffer
        .iter()
        .rev()
        .take(inner.height as usize)
        .filter(|e| {
            !filter_active
                || e.kind.contains(&state.filter)
                || e.action.contains(&state.filter)
                || e.actor.contains(&state.filter)
        })
        .map(|e| {
            let color = match e.kind.as_str() {
                "container" => Color::Cyan,
                "image" => Color::Yellow,
                "network" => Color::Magenta,
                _ => Color::White,
            };

            Line::from(Span::styled(
                format!(" {}  {}  {}  {}", e.timestamp, e.kind, e.action, e.actor),
                Style::default().fg(color),
            ))
        })
        .collect();

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, frame.area());

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "filter");
    }
}
