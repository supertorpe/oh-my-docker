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
        format!(" CONTAINERS {} ({}) FILTER '{}' ", indicator_char, state.filtered.len(), state.filter)
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
            Line::from(Span::styled("  j/k  navigate  Enter  details  /  search  l  logs  s  shell  ?  help", Style::default().fg(Color::DarkGray))),
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

   let rows: Vec<Row> = if state.group_by_project {
        let mut grouped: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        for &idx in &state.filtered {
            let project = &state.items[idx].project;
            grouped.entry(project.clone()).or_default().push(idx);
        }
        let mut all_rows = Vec::new();
        let mut project_names: Vec<String> = grouped.keys().cloned().collect();
        project_names.sort();
        for project in project_names {
            let is_expanded = state.expanded_projects.contains(&project);
            let count = grouped[&project].len();
            let header = if is_expanded {
                format!("▾ {} ({})", project, count)
            } else {
                format!("▸ {} ({})", project, count)
            };
            let header_cells: Vec<Cell> = vec![Cell::from(header)]
                .into_iter()
                .map(|c| c.style(Style::default().fg(Color::Yellow)))
                .collect();
            all_rows.push(Row::new(header_cells));
            if is_expanded {
                for &idx in &grouped[&project] {
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

                    let state_text = if is_stopping {
                        "stopping...".to_string()
                    } else if is_starting {
                        "starting...".to_string()
                    } else if is_deleting {
                        "deleting...".to_string()
                    } else {
                        c.status.clone()
                    };

                    let indicator = if is_selected { "▶" } else { " " };

                    let mut cells: Vec<Cell> = Vec::new();
                    if state.selection_mode {
                        let check = if is_id_selected { "[x]" } else { "[ ]" };
                        cells.push(Cell::from(check));
                    }
                    if columns.show_name {
                        cells.push(Cell::from(format!("    {} {}", indicator, &c.name)));
                    }
                    if columns.show_image {
                        cells.push(Cell::from(c.image.clone()));
                    }
                    if columns.show_state {
                        cells.push(Cell::from(state_text).style(Style::default().fg(state_color)));
                    }
                    if columns.show_ports {
                        cells.push(Cell::from(c.ports.clone()));
                    }

                    let row_style = if is_selected { selected_bg } else { Style::default() };
                    all_rows.push(Row::new(cells).style(row_style).height(1));
                }
            }
        }
        all_rows
    } else {
        state
            .filtered
            .iter()
            .map(|&idx| {
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

                let state_text = if is_stopping {
                    "stopping...".to_string()
                } else if is_starting {
                    "starting...".to_string()
                } else if is_deleting {
                    "deleting...".to_string()
                } else {
                    c.status.clone()
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
                    cells.push(Cell::from(state_text).style(Style::default().fg(state_color)));
                }
                if columns.show_ports {
                    cells.push(Cell::from(c.ports.clone()));
                }

                let row_style = if is_selected { selected_bg } else { Style::default() };

                Row::new(cells)
                    .style(row_style)
                    .height(1)
            })
            .collect()
    };

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new().with_selected(state.selected);
    frame.render_stateful_widget(table, area, &mut table_state);

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "search");
    }

    render_footer(frame, inner, state.selection_mode);
}

fn render_footer(frame: &mut Frame, area: Rect, selection_mode: bool) {
    let footer = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let text = if selection_mode {
        " Space:toggle/select  Ctrl+a:all  t:stop  d:delete  Esc:exit mode  j/k ↓↑  /search  Enter:details  l:logs  s:shell  ?:help "
    } else {
        " Space:select mode  j/k ↓↑  /search  Enter:details  l:logs  s:shell  r:restart  t:stop/start  d:delete  ?:help "
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}


