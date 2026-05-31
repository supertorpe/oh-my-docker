use std::collections::HashSet;
use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::event::{ContainerSummary, ImageEntry, NetworkEntry, VolumeEntry};

pub trait Resource: 'static {
    type Summary: Clone + std::fmt::Debug + Send;

    fn title() -> &'static str;
    fn column_headers() -> Vec<(&'static str, Constraint)>;
    fn cell_value(item: &Self::Summary, col: usize) -> String;
    fn matches_filter(item: &Self::Summary, term: &str) -> bool;

    fn row_style(item: &Self::Summary, is_selected: bool) -> Style {
        let _ = item;
        if is_selected {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        }
    }

    fn group_by(item: &Self::Summary) -> Option<String> {
        let _ = item;
        None
    }

    fn column_picker_labels() -> Vec<&'static str> {
        vec![]
    }

    fn empty_hint() -> &'static str {
        "  Esc  back"
    }
}

#[derive(Clone, Debug)]
pub struct ResourceState<T: Resource> {
    pub items: Vec<T::Summary>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub filter_active: bool,
    pub loading: bool,
    pub last_updated: Option<Instant>,
    pub scroll_offset: usize,
    pub show_column_picker: bool,
    pub column_picker_selection: usize,
}

impl<T: Resource> Default for ResourceState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            filter: String::new(),
            filter_active: false,
            loading: true,
            last_updated: None,
            scroll_offset: 0,
            show_column_picker: false,
            column_picker_selection: 0,
        }
    }
}

impl<T: Resource> ResourceState<T> {
    pub fn apply_filter<F>(&mut self, extra_filter: F)
    where
        F: Fn(&T::Summary) -> bool,
    {
        let items = &self.items;
        let filter = &self.filter;
        if filter.is_empty() {
            self.filtered = (0..items.len()).filter(|&i| extra_filter(&items[i])).collect();
        } else {
            let term = filter.to_lowercase();
            self.filtered = (0..items.len())
                .filter(|&i| extra_filter(&items[i]) && T::matches_filter(&items[i], &term))
                .collect();
        }
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn reorder_by_group(&mut self) {
        let mut grouped: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        for &idx in &self.filtered {
            let g = T::group_by(&self.items[idx]).unwrap_or_else(|| "Ungrouped".to_string());
            grouped.entry(g).or_default().push(idx);
        }
        let mut names: Vec<String> = grouped.keys().cloned().collect();
        names.sort();
        self.filtered = names.into_iter().flat_map(|g| grouped.remove(&g).unwrap()).collect();
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn update_items<F>(&mut self, new_items: Vec<T::Summary>, extra_filter: F)
    where
        F: Fn(&T::Summary) -> bool,
    {
        self.items = new_items;
        self.loading = false;
        self.last_updated = Some(Instant::now());
        self.apply_filter(extra_filter);
    }
}

// Generic render for simple list views (images, volumes, networks)
pub fn render_simple_list<T: Resource>(
    frame: &mut Frame,
    area: Rect,
    state: &mut ResourceState<T>,
    tick_count: u64,
    polling_interval_ms: u64,
) {
    if state.show_column_picker {
        render_column_picker::<T>(frame, area, state);
        return;
    }

    let (indicator_char, indicator_color) = if state.loading {
        (crate::ui::spinner_char(tick_count), Color::Yellow)
    } else {
        crate::ui::staleness_indicator(state.last_updated, polling_interval_ms)
    };

    let title = format!(
        " {} {} ({}) ",
        T::title(),
        indicator_char,
        if state.loading { "loading...".to_string() } else if !state.filter.is_empty() {
            format!("{}/{}", state.filtered.len(), state.items.len())
        } else {
            state.filtered.len().to_string()
        }
    );
    if !state.filter.is_empty() && !state.loading {
        let _ = &format!("'{}'", state.filter);
    }

    let block = Block::default()
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    let inner = block.inner(area);

    if state.loading && state.items.is_empty() {
        let noun = T::title().to_lowercase();
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} Loading {}...", crate::ui::spinner_char(tick_count), noun),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if state.items.is_empty() && !state.loading {
        let noun = T::title().to_lowercase();
        let text = Text::from(vec![
            Line::from(Span::styled(
                format!("  No {} found", noun),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                T::empty_hint(),
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if !state.filter.is_empty() && state.filtered.is_empty() && !state.items.is_empty() {
        let text = Text::from(vec![
            Line::from(Span::styled("  Nothing matched", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  Esc  clear filter  /  change filter", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let mut widths: Vec<Constraint> = Vec::new();
    let mut header_cells: Vec<&str> = Vec::new();

    for (h, w) in T::column_headers() {
        widths.push(w);
        header_cells.push(h);
    }

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

    let rows: Vec<Row> = state.filtered.iter().map(|&idx| {
        let item = &state.items[idx];
        let is_selected = state.filtered.get(state.selected) == Some(&idx);
        let indicator = if is_selected { "▶ " } else { "  " };
        let style = T::row_style(item, is_selected);

        let mut cells: Vec<Cell> = Vec::new();
        for col in 0..T::column_headers().len() {
            let val = if col == 0 {
                format!("{}{}", indicator, T::cell_value(item, col))
            } else {
                T::cell_value(item, col)
            };
            cells.push(Cell::from(val));
        }

        Row::new(cells).style(style).height(1)
    }).collect();

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new()
        .with_selected(Some(state.selected))
        .with_offset(state.scroll_offset);
    frame.render_stateful_widget(table, area, &mut table_state);
    state.scroll_offset = table_state.offset();

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "filter");
    }
}

fn render_column_picker<T: Resource>(frame: &mut Frame, area: Rect, state: &ResourceState<T>) {
    let labels: Vec<(&str, bool)> = T::column_picker_labels().iter().map(|l| (*l, true)).collect();
    crate::ui::column_picker::render_column_picker(frame, area, &labels, state.column_picker_selection);
}

// --- Container Resource ---

#[derive(Clone, Debug)]
pub struct ContainerResource;
impl Resource for ContainerResource {
    type Summary = ContainerSummary;

    fn title() -> &'static str { "CONTAINERS" }
    fn column_headers() -> Vec<(&'static str, Constraint)> {
        vec![
            ("NAME", Constraint::Min(15)),
            ("IMAGE", Constraint::Min(14)),
            ("STATE", Constraint::Min(16)),
            ("PORTS", Constraint::Fill(1)),
        ]
    }
    fn cell_value(item: &Self::Summary, col: usize) -> String {
        match col {
            0 => item.name.clone(),
            1 => item.image.clone(),
            2 => item.status.clone(),
            3 => item.ports.clone(),
            _ => String::new(),
        }
    }
    fn matches_filter(item: &Self::Summary, term: &str) -> bool {
        item.name.to_lowercase().contains(term)
            || item.image.to_lowercase().contains(term)
            || item.state.to_lowercase().contains(term)
            || item.id.contains(term)
    }
    fn group_by(item: &Self::Summary) -> Option<String> {
        if item.project.is_empty() { Some("Ungrouped".to_string()) } else { Some(item.project.clone()) }
    }
    fn column_picker_labels() -> Vec<&'static str> {
        vec!["Name", "Image", "State", "Ports"]
    }
    fn empty_hint() -> &'static str {
        "  r  run an image  /  search containers  Space  select mode"
    }
}

#[derive(Clone, Debug)]
pub struct ContainerStateExtra {
    pub docker_connected: bool,
    pub docker_reconnecting: bool,
    pub stopping_containers: HashSet<String>,
    pub starting_containers: HashSet<String>,
    pub deleting_containers: HashSet<String>,
    pub selection_mode: bool,
    pub selected_ids: HashSet<String>,
    pub status_filter: String,
}

impl ContainerStateExtra {
    pub fn new() -> Self {
        Self {
            docker_connected: false,
            docker_reconnecting: false,
            stopping_containers: HashSet::new(),
            starting_containers: HashSet::new(),
            deleting_containers: HashSet::new(),
            selection_mode: false,
            selected_ids: HashSet::new(),
            status_filter: String::new(),
        }
    }
}

pub fn render_containers(
    frame: &mut Frame,
    area: Rect,
    state: &mut ResourceState<ContainerResource>,
    extra: &ContainerStateExtra,
    tick_count: u64,
    polling_interval_ms: u64,
) {
    if state.show_column_picker {
        let labels: Vec<(&str, bool)> = ContainerResource::column_picker_labels().iter().map(|l| (*l, true)).collect();
        crate::ui::column_picker::render_column_picker(frame, area, &labels, state.column_picker_selection);
        return;
    }

    let (indicator_char, indicator_color) = if state.loading {
        (crate::ui::spinner_char(tick_count), Color::Yellow)
    } else {
        crate::ui::staleness_indicator(state.last_updated, polling_interval_ms)
    };

    let title = if state.loading {
        format!(" CONTAINERS {} (loading...) ", indicator_char)
    } else if !state.filter.is_empty() {
        format!(" CONTAINERS {} ({}/{}) FILTER '{}' ", indicator_char, state.filtered.len(), state.items.len(), state.filter)
    } else if extra.selection_mode && !extra.selected_ids.is_empty() {
        format!(" CONTAINERS {} ({}) [{}] ", indicator_char, state.filtered.len(), extra.selected_ids.len())
    } else {
        let status_tag = if extra.status_filter.is_empty() {
            String::new()
        } else {
            format!(" [{}]", extra.status_filter)
        };
        format!(" CONTAINERS {} ({}){} ", indicator_char, state.filtered.len(), status_tag)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    let inner = block.inner(area);

    if state.loading && !extra.docker_connected {
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

    if state.loading && extra.docker_connected {
        let spinner = crate::ui::spinner_char(tick_count);
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} Loading containers...", spinner),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if extra.docker_connected && state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No containers found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  r  run an image  /  search containers  Space  select mode", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if !state.filter.is_empty() && state.filtered.is_empty() && !state.items.is_empty() {
        let text = Text::from(vec![
            Line::from(Span::styled("  Nothing matched", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  Esc  clear filter  /  change filter", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if !extra.docker_connected && !state.loading {
        let (msg, hint) = if extra.docker_reconnecting {
            ("  Docker daemon not available — reconnecting...", "  Waiting for Docker to come back online")
        } else {
            ("  Docker daemon not available", "  Start Docker and restart the app")
        };
        let color = if extra.docker_reconnecting { Color::Yellow } else { Color::Red };
        let text = Text::from(vec![
            Line::from(Span::styled(msg, Style::default().fg(color))),
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

    if extra.selection_mode {
        widths.push(Constraint::Length(3));
        header_cells.push("");
    }
    for (h, w) in ContainerResource::column_headers() {
        widths.push(w);
        header_cells.push(h);
    }

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

    let mut grouped: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for &idx in &state.filtered {
        let project = &state.items[idx].project;
        grouped.entry(if project.is_empty() { "Ungrouped".to_string() } else { project.clone() }).or_default().push(idx);
    }
    let mut selected_row = 0;
    let all_rows: Vec<Row> = {
        let mut rows = Vec::new();
        let mut group_names: Vec<String> = grouped.keys().cloned().collect();
        group_names.sort();
        for group_name in group_names {
            let indices = &grouped[&group_name];
            let mut hdr_cells: Vec<Cell> = Vec::new();
            if extra.selection_mode {
                hdr_cells.push(Cell::from(""));
            }
            hdr_cells.push(
                Cell::from(format!(" {} ({}) ", group_name, indices.len()))
                    .style(Style::default().fg(Color::Yellow)),
            );
            rows.push(Row::new(hdr_cells));

            for &idx in indices {
                let c = &state.items[idx];
                let is_selected = state.filtered.get(state.selected) == Some(&idx);
                if is_selected {
                    selected_row = rows.len();
                }
                let is_stopping = extra.stopping_containers.contains(&c.id);
                let is_starting = extra.starting_containers.contains(&c.id);
                let is_deleting = extra.deleting_containers.contains(&c.id);
                let is_id_selected = extra.selected_ids.contains(&c.id);

                let indicator = if is_selected { "▶" } else { " " };

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

                let state_color = if is_stopping || is_starting || is_deleting {
                    Color::Yellow
                } else {
                    match c.state.as_str() {
                        "running" => Color::Green,
                        "exited" | "dead" => Color::Red,
                        _ => Color::Yellow,
                    }
                };

                let mut cells: Vec<Cell> = Vec::new();
                if extra.selection_mode {
                    let check = if is_id_selected { "[x]" } else { "[ ]" };
                    cells.push(Cell::from(check));
                }
                cells.push(Cell::from(format!("{} {}", indicator, &c.name)));
                cells.push(Cell::from(c.image.clone()));
                cells.push(Cell::from(state_text).style(Style::default().fg(state_color)));
                cells.push(Cell::from(c.ports.clone()));

                let row_style = if is_selected { selected_bg } else { Style::default() };
                rows.push(Row::new(cells).style(row_style).height(1));
            }
        }
        rows
    };

    let table = Table::new(all_rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new()
        .with_selected(Some(selected_row))
        .with_offset(state.scroll_offset);
    frame.render_stateful_widget(table, area, &mut table_state);
    state.scroll_offset = table_state.offset();

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "filter");
    }
}

// --- Image Resource ---

#[derive(Clone, Debug)]
pub struct ImageResource;
impl Resource for ImageResource {
    type Summary = ImageEntry;

    fn title() -> &'static str { "IMAGES" }
    fn column_headers() -> Vec<(&'static str, Constraint)> {
        vec![
            ("REPOSITORY", Constraint::Min(18)),
            ("TAG", Constraint::Length(16)),
            ("SIZE", Constraint::Length(10)),
            ("ID", Constraint::Length(14)),
        ]
    }
    fn cell_value(item: &Self::Summary, col: usize) -> String {
        match col {
            0 => item.repository.clone(),
            1 => item.tag.clone(),
            2 => {
                if item.size > 1_000_000_000 {
                    format!("{:.1}GB", item.size as f64 / 1_000_000_000.0)
                } else if item.size > 1_000_000 {
                    format!("{:.1}MB", item.size as f64 / 1_000_000.0)
                } else if item.size > 1_000 {
                    format!("{:.1}KB", item.size as f64 / 1_000.0)
                } else {
                    format!("{}B", item.size)
                }
            }
            3 => item.id[..12.min(item.id.len())].to_string(),
            _ => String::new(),
        }
    }
    fn matches_filter(item: &Self::Summary, term: &str) -> bool {
        item.repository.to_lowercase().contains(term)
            || item.tag.to_lowercase().contains(term)
            || item.id.contains(term)
    }
    fn column_picker_labels() -> Vec<&'static str> {
        vec!["Repository", "Tag", "Size", "ID"]
    }
    fn empty_hint() -> &'static str {
        "  Esc  back"
    }
}

#[derive(Clone, Debug)]
pub struct VolumeResource;
impl Resource for VolumeResource {
    type Summary = VolumeEntry;

    fn title() -> &'static str { "VOLUMES" }
    fn column_headers() -> Vec<(&'static str, Constraint)> {
        vec![
            ("NAME", Constraint::Min(15)),
            ("DRIVER", Constraint::Length(10)),
            ("MOUNTPOINT", Constraint::Fill(1)),
        ]
    }
    fn cell_value(item: &Self::Summary, col: usize) -> String {
        match col {
            0 => item.name.clone(),
            1 => item.driver.clone(),
            2 => item.mountpoint.clone(),
            _ => String::new(),
        }
    }
    fn matches_filter(item: &Self::Summary, term: &str) -> bool {
        item.name.to_lowercase().contains(term)
            || item.driver.to_lowercase().contains(term)
    }
    fn column_picker_labels() -> Vec<&'static str> {
        vec!["Name", "Driver", "Mountpoint"]
    }
    fn empty_hint() -> &'static str {
        "  Esc  back"
    }
}

#[derive(Clone, Debug)]
pub struct NetworkResource;
impl Resource for NetworkResource {
    type Summary = NetworkEntry;

    fn title() -> &'static str { "NETWORKS" }
    fn column_headers() -> Vec<(&'static str, Constraint)> {
        vec![
            ("NAME", Constraint::Length(22)),
            ("ID", Constraint::Length(10)),
            ("DRIVER", Constraint::Length(10)),
            ("SCOPE", Constraint::Length(8)),
            ("SUBNET", Constraint::Length(18)),
            ("GATEWAY", Constraint::Length(18)),
            ("CONTAINERS", Constraint::Length(10)),
        ]
    }
    fn cell_value(item: &Self::Summary, col: usize) -> String {
        match col {
            0 => item.name.clone(),
            1 => item.id[..12.min(item.id.len())].to_string(),
            2 => item.driver.clone(),
            3 => item.scope.clone(),
            4 => item.subnet.clone(),
            5 => item.gateway.clone(),
            6 => item.containers.to_string(),
            _ => String::new(),
        }
    }
    fn matches_filter(item: &Self::Summary, term: &str) -> bool {
        item.name.to_lowercase().contains(term)
            || item.driver.to_lowercase().contains(term)
            || item.scope.to_lowercase().contains(term)
    }
    fn column_picker_labels() -> Vec<&'static str> {
        vec!["Name", "ID", "Driver", "Scope", "IPAM"]
    }
    fn empty_hint() -> &'static str {
        "  Esc  back"
    }
}
