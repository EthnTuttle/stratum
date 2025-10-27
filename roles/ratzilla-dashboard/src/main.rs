//! Ratzilla Dashboard - Terminal UI for SV2 Metrics

use ratzilla_dashboard::{App, ui};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ratzilla-dashboard")]
#[command(about = "Terminal UI dashboard for SV2 mining metrics", long_about = None)]
struct Args {
    /// Prometheus metrics URL
    #[arg(short, long, default_value = "http://localhost:9090/metrics")]
    metrics_url: String,

    /// Refresh interval in seconds
    #[arg(short, long, default_value = "5")]
    refresh_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(args.metrics_url);

    // Initial fetch
    let _ = app.fetch_metrics().await;

    // Run the app
    let res = run_app(&mut terminal, &mut app, Duration::from_secs(args.refresh_interval)).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    refresh_interval: Duration,
) -> Result<()> {
    let mut last_update = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Check for key events with 250ms timeout
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => {
                        app.fetch_metrics().await?;
                        last_update = Instant::now();
                    }
                    _ => {}
                }
            }
        }

        // Auto-refresh based on interval
        if last_update.elapsed() >= refresh_interval {
            app.fetch_metrics().await?;
            last_update = Instant::now();
        }
    }
}
