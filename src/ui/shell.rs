use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::ShellState;

pub fn render(frame: &mut Frame, state: &ShellState) {
    let block = Block::default()
        .title(format!(" SHELL — {} ", state.container_id))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let text = Text::from(vec![
        Line::from(Span::styled("  Opening shell...", Style::default().fg(Color::Yellow))),
        Line::from(""),
        if state.stop_on_exit {
            Line::from(Span::styled("  WARNING: Container will STOP when you exit (exit/Ctrl+D)", Style::default().fg(Color::Red)))
        } else {
            Line::from(Span::styled("  Container will keep running after you exit", Style::default().fg(Color::DarkGray)))
        },
        Line::from(""),
        Line::from(Span::styled("  Esc to cancel", Style::default().fg(Color::DarkGray))),
    ]);

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(paragraph, frame.area());
}
