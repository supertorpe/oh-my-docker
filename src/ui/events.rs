use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::EventsState;

pub fn render(frame: &mut Frame, state: &EventsState) {
    let area = frame.area();

    let title = if state.paused {
        if !state.filter.is_empty() {
            format!(" DOCKER EVENTS (PAUSED) FILTER '{}' ", state.filter)
        } else {
            " DOCKER EVENTS (PAUSED) ".to_string()
        }
    } else if !state.filter.is_empty() {
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
            Line::from(Span::styled("  /  filter  Space:pause  e:export  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), frame.area());
        return;
    }

    let filter_active = !state.filter.is_empty();
    let height = inner.height as usize;

    let start = if state.paused {
        state.buffer.len().saturating_sub(height + state.scroll_offset)
    } else {
        0
    };

    let filtered: Vec<&crate::app::event::DockerEvent> = state
        .buffer
        .iter()
        .skip(start)
        .take(height)
        .filter(|e| {
            !filter_active
                || e.kind.contains(&state.filter)
                || e.action.contains(&state.filter)
                || e.actor.contains(&state.filter)
        })
        .collect();

    let lines: Vec<Line> = filtered.iter().map(|e| {
        let icon = match e.kind.as_str() {
            "container" => "\u{1f4e6}",
            "image" => "\u{1f5bc}",
            "network" => "\u{1f310}",
            "volume" => "\u{1f4be}",
            _ => "\u{2699}",
        };

        let color = match e.kind.as_str() {
            "container" => Color::Cyan,
            "image" => Color::Yellow,
            "network" => Color::Magenta,
            "volume" => Color::Green,
            _ => Color::White,
        };

        Line::from(Span::styled(
            format!(" {} {}  {}  {}", icon, e.timestamp, e.kind, e.action),
            Style::default().fg(color),
        ))
    }).collect();

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, frame.area());

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "filter");
    }

    render_footer(frame, area, state.paused, state.buffer.len());
}

fn render_footer(frame: &mut Frame, area: Rect, paused: bool, total: usize) {
    let footer = Rect {
        x: area.x,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let text = if paused {
        format!("  r resume   / filter   e:export   j/k scroll   Esc back  ({} events)", total)
    } else {
        format!("  p pause   / filter   e:export   j/k scroll   Esc back  ({} events)", total)
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
