use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::HelpState;

pub fn render(frame: &mut Frame, help: &mut HelpState) {
    let area = frame.area();
    let block = Block::default()
        .title(" HELP ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let text = Text::from(vec![
        format!("  omdocker v{}", env!("CARGO_PKG_VERSION")).into(),
        "".into(),
        "  GLOBAL".into(),
        "    q     Quit".into(),
        "    ?     Toggle help".into(),
        "    Esc   Go back".into(),
        "".into(),
        "  CONTAINERS".into(),
        "    j/↓   Navigate down".into(),
        "    k/↑   Navigate up".into(),
        "    Enter Open details".into(),
        "    l     Open logs".into(),
        "    s     Open shell".into(),
        "    t     Start/Stop container".into(),
        "    r     Restart container".into(),
        "    d     Delete container".into(),
        "    /     Search".into(),
        "    i     Images view".into(),
        "    e     Events view".into(),
        "    %     Statistics view".into(),
        "    n     Networks view".into(),
        "    v     Volumes view".into(),
        "".into(),
        "  IMAGES".into(),
        "    j/↓   Navigate down".into(),
        "    k/↑   Navigate up".into(),
        "    r     Run image".into(),
        "    d     Remove image".into(),
        "    /     Search".into(),
        "".into(),
        "  LOGS".into(),
        "    j/↓   Scroll down".into(),
        "    k/↑   Scroll up".into(),
        "    PgDn  Page down".into(),
        "    PgUp  Page up".into(),
        "    g     Jump to bottom".into(),
        "    G     Jump to top".into(),
        "    /     Search logs".into(),
        "    Space Pause/unpause".into(),
       "".into(),
        "  EVENTS".into(),
        "    /     Filter events".into(),
        "".into(),
        "  STATISTICS".into(),
        "".into(),
        "  NETWORKS / VOLUMES".into(),
        "    j/↓   Navigate down".into(),
        "    k/↑   Navigate up".into(),
        "    d     Delete selected".into(),
        "".into(),
        "  SCROLLING".into(),
        "    j/↓   Scroll down".into(),
        "    k/↑   Scroll up".into(),
        "    PgDn  Page down".into(),
        "    PgUp  Page up".into(),
        "    g     Jump to bottom".into(),
        "    G     Jump to top".into(),
        "".into(),
        "  Press Esc or ? to close".into(),
    ]);

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
