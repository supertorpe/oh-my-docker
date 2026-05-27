use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::widgets::{Block, Borders};

pub fn border() -> Color {
    Color::Magenta
}

pub fn view_border() -> Color {
    Color::Green
}

pub fn accent() -> Color {
    Color::Cyan
}

pub fn muted() -> Color {
    Color::Gray
}

pub fn info() -> Color {
    Color::Blue
}

pub fn primary() -> Style {
    Style::default()
        .fg(accent())
        .add_modifier(Modifier::BOLD)
}

pub fn status_bar_default() -> Style {
    Style::default()
        .fg(info())
        .add_modifier(Modifier::REVERSED)
}

pub fn panel_block<'a, T: Into<ratatui::text::Line<'a>>>(title: T) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(border()))
        .title(title)
}
