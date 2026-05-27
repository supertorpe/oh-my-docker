use std::collections::HashMap;
use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::ContainersState;
use crate::config::ContainerColumns;

fn staleness_indicator(last_updated: Option<std::time::Instant>, interval_ms: u64) -> (char, Color) {
    let threshold_fresh = Duration::from_millis(interval_ms * 2);
    let threshold_stale = Duration::from_millis(interval_ms * 5);

    match last_updated {
        Some(instant) => {
            let elapsed = instant.elapsed();
            if elapsed < threshold_fresh {
                ('●', Color::Green)
            } else if elapsed < threshold_stale {
                ('○', Color::Yellow)
            } else {
                ('◌', Color::Red)
            }
        }
        None => ('?', Color::DarkGray),
    }
}

fn project_group_header(group_name: &str, count: usize) -> Row<'static> {
    let header = format!(" {} ({}) ", group_name, count);
    Row::new(vec![Cell::from(header).style(Style::default().fg(Color::Yellow))])
}

pub fn render(frame: &mut Frame, state: &ContainersState, tick_count: u64, columns: &ContainerColumns) {
    let area = frame.area();

    let (indicator_char, indicator_color) = if state.loading {
        ('⠋', Color::Yellow)
    } else if !state.docker_connected {
        ('?', Color::Red)
    } else {
        staleness_indicator(state.last_updated, 2000)
    };

    let title = if state.loading {
        format!(" CONTAINERS {} (loading...) ", indicator_char)
    } else if !state.filter.is_empty() {
        format!(" CONTAINERS {} ({}/{}) FILTER '{}' ", indicator_char, state.filtered.len(), state.items.len(), state.filter)
    } else if state.selection_mode && !state.selected_ids.is_empty() {
        format!(" CONTAINERS {} ({}) [{}] ", indicator_char, state.filtered.len(), state.selected_ids.len())
    } else {
        format!(" CONTAINERS {} ({}) ", indicator_char, state.filtered.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    let inner = block.inner(area);

    if state.loading && !state.docker_connected {
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠏'];
        let spinner = spinner_chars[(tick_count as usize / 2) % spinner_chars.len()];
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} Connecting to Docker...", spinner),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if state.docker_connected && state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No containers found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  j/k ↓↑  Enter:details  /:search  l:logs  s:shell  ?:help", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if !state.docker_connected && !state.loading {
        let (msg, hint) = if state.docker_reconnecting {
            ("  Docker daemon not available — reconnecting...", "  Waiting for Docker to come back online")
        } else {
            ("  Docker daemon not available", "  Start Docker and restart the app")
        };
        let color = if state.docker_reconnecting { Color::Yellow } else { Color::Red };
        let spinner = if state.docker_reconnecting { " ⠋" } else { "" };
        let text = Text::from(vec![
            Line::from(Span::styled(format!("{}{}", spinner, msg), Style::default().fg(color))),
            Line::from(""),
            Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if state.show_column_picker {
        render_column_picker(frame, area, columns, state.column_picker_selection);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let mut widths = Vec::new();
    let mut header_cells = Vec::new();

    if state.selection_mode {
        widths.push(Constraint::Length(3));
        header_cells.push("");
    }

    if columns.show_name {
        widths.push(Constraint::Min(15));
        header_cells.push("NAME");
    }
    if columns.show_image {
        widths.push(Constraint::Min(14));
        header_cells.push("IMAGE");
    }
    if columns.show_state {
        widths.push(Constraint::Min(16));
        header_cells.push("STATE");
    }
    if columns.show_ports {
        widths.push(Constraint::Fill(1));
        header_cells.push("PORTS");
    }

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

    // Always group by project
    let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();
    for &idx in &state.filtered {
        let project = &state.items[idx].project;
        grouped.entry(if project.is_empty() { "Ungrouped".to_string() } else { project.clone() }).or_default().push(idx);
    }
    let all_rows: Vec<Row> = {
        let mut rows = Vec::new();
        let mut group_names: Vec<String> = grouped.keys().cloned().collect();
        group_names.sort();
        for group_name in group_names {
            let indices = &grouped[&group_name];
            rows.push(project_group_header(&group_name, indices.len()));
            for &idx in indices {
                let c = &state.items[idx];
                let is_selected = state.filtered.get(state.selected) == Some(&idx);
                let is_stopping = state.stopping_containers.contains(&c.id);
                let is_starting = state.starting_containers.contains(&c.id);
                let is_deleting = state.deleting_containers.contains(&c.id);
                let is_id_selected = state.selected_ids.contains(&c.id);

                let state_color = if is_stopping || is_starting || is_deleting {
                    Color::Yellow
                } else {
                    match c.state.as_str() {
                        "running" => Color::Green,
                        "exited" | "dead" => Color::Red,
                        _ => Color::Yellow,
                    }
                };

                let health_indicator = if !c.health.is_empty() {
                    match c.health.as_str() {
                        "healthy" => ("● ", Color::Green),
                        "unhealthy" => ("✗ ", Color::Red),
                        "starting" => ("◐ ", Color::Yellow),
                        _ => ("", Color::DarkGray),
                    }
                } else {
                    ("", Color::DarkGray)
                };

                let state_text = if is_stopping {
                    format!("{}stopping...", health_indicator.0)
                } else if is_starting {
                    format!("{}starting...", health_indicator.0)
                } else if is_deleting {
                    format!("{}deleting...", health_indicator.0)
                } else {
                    format!("{}{}", health_indicator.0, c.status)
                };

                let indicator = if is_selected { "▶" } else { " " };

                let mut cells: Vec<Cell> = Vec::new();
                if state.selection_mode {
                    let check = if is_id_selected { "[x]" } else { "[ ]" };
                    cells.push(Cell::from(check));
                }
                if columns.show_name {
                    let name_display = if !c.project.is_empty() {
                        format!("🐳 {} {}", indicator, &c.name)
                    } else {
                        format!("{} {}", indicator, &c.name)
                    };
                    cells.push(Cell::from(name_display));
                }
                if columns.show_image {
                    cells.push(Cell::from(c.image.clone()));
                }
                if columns.show_state {
                    let indicator_color = if !c.health.is_empty() {
                        match c.health.as_str() {
                            "healthy" => Color::Green,
                            "unhealthy" => Color::Red,
                            "starting" => Color::Yellow,
                            _ => Color::DarkGray,
                        }
                    } else {
                        state_color
                    };
                    cells.push(Cell::from(state_text).style(Style::default().fg(indicator_color)));
                }
                if columns.show_ports {
                    cells.push(Cell::from(c.ports.clone()));
                }

                let row_style = if is_selected { selected_bg } else { Style::default() };
                rows.push(Row::new(cells).style(row_style).height(1));
            }
        }
        rows
    };

    let table = Table::new(all_rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new().with_selected(state.selected);
    frame.render_stateful_widget(table, area, &mut table_state);

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "search");
    }

    render_footer(frame, inner, state.selection_mode);
}

fn render_column_picker(frame: &mut Frame, area: Rect, columns: &ContainerColumns, selection: usize) {
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
        ("Image", columns.show_image),
        ("State", columns.show_state),
        ("Ports", columns.show_ports),
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

fn render_footer(frame: &mut Frame, area: Rect, selection_mode: bool) {
    let footer = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let text = if selection_mode {
        " Space:toggle/select  Ctrl+a:all  t:stop  d:delete  Esc:exit mode  j/k ↓↑  /search  Enter:details  l:logs  s:shell  ?:help  ^O:columns "
    } else {
        " Space:select mode  j/k ↓↑  /search  Enter:details  l:logs  s:shell  r:restart  t:stop/start  d:delete  ?:help  ^O:columns "
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}


