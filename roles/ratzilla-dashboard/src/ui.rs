//! Terminal UI rendering using ratatui

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Share accounting table
            Constraint::Length(5),  // Connection stats
            Constraint::Length(3),  // Footer/error
        ])
        .split(f.area());

    // Header
    render_header(f, chunks[0], app);

    // Share accounting table
    render_share_table(f, chunks[1], app);

    // Connection stats
    render_connection_stats(f, chunks[2], app);

    // Footer
    render_footer(f, chunks[3], app);
}

fn render_header(f: &mut Frame, area: Rect, _app: &App) {
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("╔═══════════════════════════════════════════════════╗", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("║", Style::default().fg(Color::Cyan)),
            Span::styled("   SV2 Mining Dashboard - Share Accounting", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("    ║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("╚═══════════════════════════════════════════════════╝", Style::default().fg(Color::Cyan)),
        ]),
    ]);
    f.render_widget(header, area);
}

fn render_share_table(f: &mut Frame, area: Rect, app: &App) {
    let mut rows = Vec::new();

    // Sort by downstream_id, then channel_id
    let mut metrics = app.metrics.share_metrics.clone();
    metrics.sort_by(|a, b| {
        a.downstream_id.cmp(&b.downstream_id)
            .then(a.channel_id.cmp(&b.channel_id))
    });

    for metric in metrics {
        rows.push(Row::new(vec![
            Cell::from(metric.downstream_id.to_string()),
            Cell::from(metric.channel_id.to_string()),
            Cell::from(metric.channel_type.clone()),
            Cell::from(metric.shares_accepted.to_string()),
            Cell::from(format!("{:.2}", metric.share_work_sum)),
            Cell::from(metric.sequence_number.to_string()),
            Cell::from(metric.blocks_found.to_string()),
        ]));
    }

    let header = Row::new(vec![
        Cell::from("DS ID").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("CH ID").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Shares").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Work Sum").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Seq #").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Blocks").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("📊 Share Accounting")
            .border_style(Style::default().fg(Color::Cyan))
    )
    .style(Style::default().fg(Color::White));

    f.render_widget(table, area);
}

fn render_connection_stats(f: &mut Frame, area: Rect, app: &App) {
    let stats = &app.metrics.connection_metrics;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Downstream Connections: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                stats.downstream_connections_active.to_string(),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("Template Provider: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                if stats.template_provider_connected { "✓ Connected" } else { "✗ Disconnected" },
                if stats.template_provider_connected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                }
            ),
        ]),
    ];

    // Add channel type counts
    for (channel_type, count) in &stats.channels_by_type {
        lines.push(Line::from(vec![
            Span::styled(format!("  {} channels: ", channel_type), Style::default().fg(Color::Cyan)),
            Span::styled(count.to_string(), Style::default().fg(Color::White)),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔌 Connections")
                .border_style(Style::default().fg(Color::Cyan))
        );

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(error) = &app.error {
        vec![
            Line::from(vec![
                Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ])
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::DarkGray)),
                Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to quit  •  ", Style::default().fg(Color::DarkGray)),
                Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to refresh  •  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Updates every 5s", Style::default().fg(Color::Green)),
            ])
        ]
    };

    let footer = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}
