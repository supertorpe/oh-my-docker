use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, BorderType};

use crate::app::mode::Mode;
use crate::app::state::{AppState, ExplorerPanel, ExplorerFocus};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let is_volume = matches!(state.navigation.mode_stack.current(), Mode::ExplorerVolume(_, _));
    let container_id = &state.explorer.container_id.clone();

    if !is_volume && container_id.is_empty() {
        render_prompt(frame, area);
        return;
    }

    let show_toast = state.explorer.transfer_message.is_some() || state.explorer.transfer_error.is_some();
    let toast_height: u16 = if show_toast { 2 } else { 0 };
    let main_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(toast_height),
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Percentage(48),
        ])
        .split(main_area);

    let current_focus = state.explorer.focus;
    let focus_left = current_focus == ExplorerFocus::Left;

    if is_volume {
        let host_title = format!("Host ({})", state.explorer.host.path);
        let volume_title = format!("Volume ({})", state.explorer.container.path);
        render_panel(frame, chunks[0], &mut state.explorer.host, focus_left, &host_title, state.tick_count);
        render_panel(frame, chunks[1], &mut state.explorer.container, !focus_left, &volume_title, state.tick_count);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(48),
                Constraint::Percentage(48),
            ])
            .split(main_area);

        let container_name = state.containers.items
            .iter()
            .find(|c| c.id == state.explorer.container_id)
            .map(|c| c.name.as_str())
            .unwrap_or(container_id);

        let host_title = format!("Host ({})", state.explorer.host.path);
        let container_title = format!("{} ({})", container_name, state.explorer.container.path);
        render_panel(frame, chunks[0], &mut state.explorer.host, focus_left, &host_title, state.tick_count);
        render_panel(frame, chunks[1], &mut state.explorer.container, !focus_left, &container_title, state.tick_count);
    }

    let toast_y = area.y + main_area.height;
    let toast_area = Rect {
        x: area.x,
        y: toast_y,
        width: area.width,
        height: 1,
    };
    if let Some(ref msg) = state.explorer.transfer_message {
        let spinner = if state.explorer.transfer_in_progress {
            let phase = state.tick_count % 4;
            ["  \u{258f}", "  \u{258e}", "  \u{258d}", "  \u{258b}"][phase as usize]
        } else {
            "  "
        };
        let fg = if state.explorer.transfer_in_progress { Color::Yellow } else { Color::Green };
        let text = format!("{}{}", spinner, msg);
        let paragraph = Paragraph::new(Line::from(Span::styled(text, Style::default().fg(fg))))
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, toast_area);
    }

    if let Some(ref err) = state.explorer.transfer_error {
        frame.render_widget(
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Black).bg(Color::Red)),
            toast_area,
        );
    }

    if let Some(ref menu) = state.explorer.context_menu {
        let menu_w = menu.items.iter().map(|i| i.label.len()).max().unwrap_or(20) as u16 + 4;
        let menu_h = menu.items.len() as u16 + 2;
        let mx = menu.x.min(area.width.saturating_sub(menu_w));
        let my = menu.y.min(area.height.saturating_sub(menu_h));
        let menu_area = Rect { x: mx, y: my, width: menu_w, height: menu_h };

        frame.render_widget(Clear, menu_area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::Black));
        frame.render_widget(block.clone(), menu_area);

        let inner = Rect { x: menu_area.x + 1, y: menu_area.y + 1, width: menu_area.width.saturating_sub(2), height: menu_area.height.saturating_sub(2) };
        for (i, item) in menu.items.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height { break; }
            let selected = i == menu.selected;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };
            frame.render_widget(
                Paragraph::new(Span::styled(format!(" {}", item.label), style)),
                Rect { x: inner.x, y, width: inner.width, height: 1 },
            );
        }
    }
}

fn render_prompt(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" EXPLORER ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let text = ratatui::text::Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Enter a container ID to start",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Navigate to a container and press 'x'",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::White)).block(block);
    frame.render_widget(paragraph, area);
}

fn render_breadcrumb_bar(frame: &mut Frame, area: Rect, path: &str, is_focused: bool) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();
    if path == "/" {
        segments.push("/".to_string());
    } else {
        segments.push("/".to_string());
        for part in path.trim_start_matches('/').split('/') {
            if !part.is_empty() {
                segments.push(part.to_string());
            }
        }
    }
    let mut display = String::new();
    let mut seg_paths = Vec::new();
    let mut cumulative = String::new();
    for seg in &segments {
        if seg == "/" {
            display.push_str(" / ");
            cumulative = "/".to_string();
        } else {
            display.push_str(&format!(" {} ", seg));
            if cumulative != "/" {
                cumulative = format!("{}/{}", cumulative, seg);
            } else {
                cumulative = format!("/{}", seg);
            }
        }
        seg_paths.push(cumulative.clone());
    }
    let fg = if is_focused { Color::Cyan } else { Color::DarkGray };
    frame.render_widget(
        Paragraph::new(Span::styled(display, Style::default().fg(fg)))
            .style(Style::default().bg(Color::Black)),
        area,
    );
    seg_paths
}

fn render_panel(
    frame: &mut Frame,
    area: Rect,
    panel: &mut ExplorerPanel,
    is_focused: bool,
    panel_title: &str,
    tick_count: u64,
) {
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let title = if !panel.filter.is_empty() {
        format!(" {} FILTER: {} ", panel_title, panel.filter)
    } else {
        format!(" {} ", panel_title)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner_area = block.inner(area);
    let (breadcrumb_area, list_area) = {
        let inner = inner_area;
        let bc_h = 1u16.min(inner.height.saturating_sub(2));
        let bc_area = Rect { x: inner.x, y: inner.y, width: inner.width, height: bc_h };
        let list_h = inner.height.saturating_sub(bc_h);
        if list_h == 0 {
            (bc_area, Rect { x: inner.x, y: inner.y + bc_h, width: inner.width, height: 0 })
        } else {
            let list_area = Rect { x: inner.x, y: inner.y + bc_h, width: inner.width, height: list_h };
            (bc_area, list_area)
        }
    };

    // Draw border block first
    frame.render_widget(block.clone(), area);
    render_breadcrumb_bar(frame, breadcrumb_area, &panel.path, is_focused);

    if panel.loading && panel.items.is_empty() {
        let spinner = crate::ui::spinner_char(tick_count);
        let text = ratatui::text::Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  Loading {}...", spinner),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text), list_area);
        render_rename_bar(frame, inner_area, panel);
        render_goto_bar(frame, inner_area, panel);
        render_create_bar(frame, inner_area, panel);
        if panel.filter_active {
            crate::ui::render_filter_bar(frame, inner_area, &panel.filter, "filter");
        }
        return;
    }

    let show_parent = panel.path != "/";
    let wide = list_area.width >= 55;

    if panel.items.is_empty() && !panel.loading {
        if show_parent {
            let indicator = if is_focused { "\u{25B6}" } else { " " };
            let style = if is_focused {
                Style::default().fg(Color::White).bg(Color::Blue)
            } else {
                Style::default()
            };
            let mut rows = Vec::new();
            let cell = if wide {
                Cell::from(format!("  {} ..  ", indicator))
            } else {
                Cell::from(format!("  {} ..", indicator))
            };
            rows.push(Row::new(vec![cell])
                .style(style)
                .height(1));
            let table = Table::new(rows, vec![Constraint::Fill(1)]);
            let mut table_state = if is_focused {
                TableState::new().with_selected(Some(0)).with_offset(0)
            } else {
                TableState::new().with_offset(0)
            };
            frame.render_stateful_widget(table, list_area, &mut table_state);
        } else {
            let text = ratatui::text::Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  (empty)",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ]);
            frame.render_widget(Paragraph::new(text), list_area);
        }
        render_rename_bar(frame, inner_area, panel);
        render_goto_bar(frame, inner_area, panel);
        render_create_bar(frame, inner_area, panel);
        if panel.filter_active {
            crate::ui::render_filter_bar(frame, inner_area, &panel.filter, "filter");
        }
        return;
    }

    let mut rows = Vec::new();

    if show_parent {
        let show_cursor = is_focused && panel.selected == 0;
        let indicator = if show_cursor { "\u{25B6}" } else { " " };
        let style = if show_cursor {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default()
        };
        let cell = if wide {
            Cell::from(format!("  {} ..  ", indicator))
        } else {
            Cell::from(format!("  {} ..", indicator))
        };
        rows.push(Row::new(vec![cell]).style(style).height(1));
    }

    for (i, entry) in panel.items.iter().enumerate() {
        let item_idx = if show_parent { i + 1 } else { i };
        let show_cursor = is_focused && panel.selected == item_idx;
        let indicator = if show_cursor { "\u{25B6}" } else { " " };
        let icon = if entry.is_dir { "\u{1F4C1}" } else { "\u{1F4C4}" };
        let sel_mark = if panel.selected_names.contains(&entry.name) { "\u{2713}" } else { " " };

        let style = if show_cursor {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else if panel.selected_names.contains(&entry.name) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let cell = if wide {
            Cell::from(format!(
                " {} {} {:>8} {} {} {}",
                indicator, entry.permissions, entry.size_str(), entry.modified, sel_mark, entry.name
            ))
        } else {
            Cell::from(format!(
                " {} {}{} {}",
                indicator, sel_mark, icon, entry.name
            ))
        };

        rows.push(Row::new(vec![cell]).style(style).height(1));
    }

    let widths = vec![Constraint::Fill(1)];
    let table = Table::new(rows, widths);

    let mut table_state = if is_focused {
        let selected_row = panel.selected;
        TableState::new()
            .with_selected(Some(selected_row))
            .with_offset(panel.scroll_offset)
    } else {
        TableState::new().with_offset(panel.scroll_offset)
    };

    frame.render_stateful_widget(table, list_area, &mut table_state);
    panel.scroll_offset = table_state.offset();
    render_rename_bar(frame, inner_area, panel);
    render_goto_bar(frame, inner_area, panel);
    render_create_bar(frame, inner_area, panel);
    if panel.filter_active {
        crate::ui::render_filter_bar(frame, inner_area, &panel.filter, "filter");
    }
}
fn render_rename_bar(frame: &mut Frame, inner: Rect, panel: &ExplorerPanel) {
    if !panel.rename_active {
        return;
    }
    let bar_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width.min(50),
        height: 1,
    };
    let display = if panel.rename_buffer.is_empty() {
        "  rename...".to_string()
    } else {
        format!(" {}", panel.rename_buffer)
    };
    frame.render_widget(Clear, bar_area);
    frame.render_widget(
        Paragraph::new(format!("/{}", display))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        bar_area,
    );
}

fn render_create_bar(frame: &mut Frame, inner: Rect, panel: &ExplorerPanel) {
    if !panel.create_active {
        return;
    }
    let bar_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width.min(60),
        height: 1,
    };
    let display = if panel.create_buffer.is_empty() {
        "  name (trailing / for dir)...".to_string()
    } else {
        format!(" {}", panel.create_buffer)
    };
    frame.render_widget(Clear, bar_area);
    frame.render_widget(
        Paragraph::new(format!(">{}", display))
            .style(Style::default().fg(Color::White).bg(Color::Green)),
        bar_area,
    );
}

fn render_goto_bar(frame: &mut Frame, inner: Rect, panel: &ExplorerPanel) {
    if !panel.goto_active {
        return;
    }
    let bar_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width.min(60),
        height: 1,
    };
    let display = if panel.goto_buffer.is_empty() {
        "  path...".to_string()
    } else {
        format!(" {}", panel.goto_buffer)
    };
    frame.render_widget(Clear, bar_area);
    frame.render_widget(
        Paragraph::new(format!(":{}", display))
            .style(Style::default().fg(Color::White).bg(Color::Blue)),
        bar_area,
    );
}
