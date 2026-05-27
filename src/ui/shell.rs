use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::ShellState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &ShellState) {
    let block = Block::default()
        .title(format!(" SHELL — {} ", state.container_id))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::view_border()));

    let text = Text::from(vec![
        Line::from(Span::styled("  Opening shell... (Press 'exit' or Ctrl+D to return)", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled("  Esc to cancel", Style::default().fg(Color::DarkGray))),
    ]);

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(paragraph, area);
}
