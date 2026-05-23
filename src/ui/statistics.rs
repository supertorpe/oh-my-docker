use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table};

use crate::app::state::StatisticsState;

pub fn render(frame: &mut Frame, state: &StatisticsState) {
    let area = frame.area();

    let title = if state.loading {
        " STATISTICS (loading...) ".to_string()
    } else {
        format!(" STATISTICS ({}) ", state.items.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    if state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No running containers", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let widths = [
        Constraint::Min(16),
        Constraint::Length(8),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(6),
    ];

    let header_cells = ["NAME", "CPU %", "MEM USAGE/LIMIT", "NET I/O", "BLOCK I/O", "PIDS"]
        .iter()
        .map(|h| Cell::from(*h).style(header_style));
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .items
        .iter()
        .map(|s| {
            let cpu_color = if s.cpu_percent > 90.0 {
                Color::Red
            } else if s.cpu_percent > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let mem_pct = s.memory_percent;
            let mem_color = if mem_pct > 90.0 {
                Color::Red
            } else if mem_pct > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let cpu_str = format!("{:.1}%", s.cpu_percent);
            let mem_str = format!(
                "{:.1}MiB / {:.1}MiB",
                s.memory_usage as f64 / 1_048_576.0,
                s.memory_limit as f64 / 1_048_576.0
            );
            let net_str = format!(
                "{:.1}MB / {:.1}MB",
                s.net_rx as f64 / 1_000_000.0,
                s.net_tx as f64 / 1_000_000.0
            );
            let block_str = if s.block_read > 0 || s.block_write > 0 {
                format!(
                    "{:.1}MB / {:.1}MB",
                    s.block_read as f64 / 1_000_000.0,
                    s.block_write as f64 / 1_000_000.0
                )
            } else {
                "0B / 0B".to_string()
            };

            Row::new(vec![
                Cell::from(format!(" {}", &s.name)),
                Cell::from(cpu_str).style(Style::default().fg(cpu_color)),
                Cell::from(mem_str).style(Style::default().fg(mem_color)),
                Cell::from(net_str),
                Cell::from(block_str),
                Cell::from(s.pids.to_string()),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(rows, widths).header(header_row).block(block);
    frame.render_widget(table, area);
}
