use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::HelpState;
use crate::config::OmdockerConfig;

pub fn render(frame: &mut Frame, help: &mut HelpState, config: &OmdockerConfig) {
    let area = frame.area();
    let block = Block::default()
        .title(" HELP ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let help_lines: Vec<Line> = config.keybindings.to_help_text().into_iter().map(|s| Line::from(s)).collect();
    let text = Text::from(help_lines);

    let max_offset = text.height().saturating_sub(area.height as usize);
    let scroll_offset = help.scroll_offset.min(max_offset);
    let scroll_offset = scroll_offset.min(10000);
    help.scroll_offset = scroll_offset;

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .scroll((scroll_offset as u16, 0))
        .block(block);

    frame.render_widget(paragraph, area);
}
