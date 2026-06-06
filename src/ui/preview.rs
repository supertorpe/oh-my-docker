use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, BorderType, Clear, Paragraph, Wrap};

use crate::app::state::FilePreviewState;

pub fn render(frame: &mut Frame, area: Rect, preview: &FilePreviewState) {
    let dialog_width = area.width.saturating_sub(8).min(120);
    let dialog_height = area.height.saturating_sub(4).min(40);
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect { x, y, width: dialog_width, height: dialog_height };

    frame.render_widget(Clear, dialog_area);

    let title = format!(" PREVIEW: {} ", preview.path);

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dialog_area);

    let text: Text = if preview.loading {
        Text::from(Line::from(Span::styled(
            "  Loading...",
            Style::default().fg(Color::Yellow),
        )))
    } else if let Some(ref err) = preview.error {
        Text::from(Line::from(Span::styled(
            format!("  {}", err),
            Style::default().fg(Color::Red),
        )))
    } else if preview.content.is_empty() {
        Text::from(Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        )))
    } else {
        let visible_lines: Vec<Line> = preview.content
            .iter()
            .skip(preview.scroll_offset)
            .take(inner.height as usize - 1)
            .map(|l| {
                if l.starts_with("-- truncated") {
                    Line::from(Span::styled(l.clone(), Style::default().fg(Color::Yellow)))
                } else {
                    Line::from(Span::raw(l.clone()))
                }
            })
            .collect();
        let mut lines = visible_lines;
        if preview.scroll_offset > 0 {
            lines.insert(0, Line::from(Span::styled(
                format!("  ↑ {} more lines", preview.scroll_offset),
                Style::default().fg(Color::DarkGray),
            )));
        }
        let remaining = preview.content.len().saturating_sub(preview.scroll_offset + inner.height as usize);
        if remaining > 0 {
            lines.push(Line::from(Span::styled(
                format!("  ↓ {} more lines", remaining),
                Style::default().fg(Color::DarkGray),
            )));
        }
        Text::from(lines)
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, dialog_area);

    // Footer hint
    let footer = format!("  Esc: close  ↑/↓: scroll ({}/{})",
        preview.scroll_offset + 1,
        preview.content.len());
    let footer_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(footer, Style::default().fg(Color::DarkGray))),
        footer_area,
    );
}
