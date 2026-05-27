use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::VolumesState;

pub fn render(frame: &mut Frame, state: &VolumesState, columns: &crate::config::VolumeColumns) {
    let area = frame.area();

    if state.show_column_picker {
        render_column_picker(frame, area, columns, state.column_picker_selection);
        return;
    }

    let (indicator_char, indicator_color) = if state.loading {
        ('⠋', Color::Yellow)
    } else {
        let (ch, color) = if let Some(instant) = state.last_updated {
            let elapsed = instant.elapsed();
            let fresh = std::time::Duration::from_secs(20);
            let stale = std::time::Duration::from_secs(50);
            if elapsed < fresh { ('●', Color::Green) }
            else if elapsed < stale { ('○', Color::Yellow) }
            else { ('◌', Color::Red) }
        } else { ('?', Color::DarkGray) };
        (ch, color)
    };

    let title = if state.loading {
        format!(" VOLUMES {} (loading...) ", indicator_char)
    } else {
        format!(" VOLUMES {} ({}) ", indicator_char, state.items.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    let inner = block.inner(area);

    if state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No volumes found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let mut widths = Vec::new();
    let mut header_cells = Vec::new();

    if columns.show_name {
        widths.push(Constraint::Length(22));
        header_cells.push("NAME");
    }
    if columns.show_driver {
        widths.push(Constraint::Length(10));
        header_cells.push("DRIVER");
    }
    if columns.show_mountpoint {
        widths.push(Constraint::Fill(1));
        header_cells.push("MOUNTPOINT");
    }

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

    let rows: Vec<Row> = state
        .items
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let is_selected = state.selected == idx;
            let indicator = if is_selected { "▶" } else { " " };
            let row_style = if is_selected { selected_bg } else { Style::default() };

            let mut cells: Vec<Cell> = Vec::new();
            if columns.show_name {
                cells.push(Cell::from(format!("{} {}", indicator, &v.name)));
            }
            if columns.show_driver {
                cells.push(Cell::from(v.driver.clone()));
            }
            if columns.show_mountpoint {
                cells.push(Cell::from(v.mountpoint.clone()));
            }

            Row::new(cells)
                .style(row_style)
                .height(1)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new().with_selected(state.selected);
    frame.render_stateful_widget(table, area, &mut table_state);

    render_footer(frame, inner);
}

fn render_column_picker(frame: &mut Frame, area: Rect, columns: &crate::config::VolumeColumns, selection: usize) {
    use ratatui::widgets::Clear;
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
    for (i, (label, active)) in [
        ("Name", columns.show_name),
        ("Driver", columns.show_driver),
        ("Mountpoint", columns.show_mountpoint),
    ].iter().enumerate() {
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
        Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(Color::Cyan)))
            .style(Style::default().fg(Color::White)),
        picker_area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(" d  delete  j/k  navigate  Esc  back  ^O:columns ")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
