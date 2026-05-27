use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use ratatui::Frame;

use crate::app::mode::TAB_TITLES;
use crate::ui::theme;

const TAB_DIVIDER: &str = " ▏ ";

pub fn render(frame: &mut Frame, area: Rect, selected_tab: usize) {
    let inactive = Style::default().fg(theme::muted());

    let titles: Vec<Line> = TAB_TITLES
        .iter()
        .map(|t| Line::from(Span::styled(format!(" {} ", t), inactive)))
        .collect();

    let tabs = Tabs::new(titles)
        .block(theme::panel_block(Span::styled(
            " Oh My Docker ",
            Style::default().fg(theme::muted()),
        )))
        .select(selected_tab)
        .padding_left("")
        .padding_right("")
        .divider(Span::styled(TAB_DIVIDER, inactive))
        .highlight_style(theme::primary().add_modifier(Modifier::REVERSED));

    frame.render_widget(tabs, area);
}
