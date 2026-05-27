use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, BorderType, Clear, Paragraph};
use crate::ui::theme;

pub fn render_column_picker(frame: &mut Frame, area: Rect, items: &[(&str, bool)], selection: usize) {
    let picker_area = Rect {
        x: area.width / 2 - 15,
        y: area.height / 2 - 4,
        width: 30,
        height: 8,
    };
    let mut lines = vec![
        Line::from(Span::styled(" COLUMNS (Space to toggle) ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];
    for (i, (label, active)) in items.iter().enumerate() {
        let check = if *active { "[x]" } else { "[ ]" };
        let style = if i == selection {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(
            format!("  {} {}", check, label),
            style,
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(" Esc: close", Style::default().fg(Color::DarkGray))));
    frame.render_widget(Clear, picker_area);
    frame.render_widget(
        Paragraph::new(ratatui::text::Text::from(lines))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(theme::view_border())))
            .style(Style::default().fg(Color::White)),
        picker_area,
    );
}
