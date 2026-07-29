mod app;
mod assets;
mod config;
mod db_manager;
mod elevation_stats;
mod events;
mod file_manager;
mod miles_stats;
mod models;
mod period;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    handle_cli_args();

    let data_dir = config::data_dir()?;

    // One-time migration from .env to config.toml
    config::migrate_from_env(&data_dir).ok();

    let app_config = config::AppConfig::load()?;

    setup_terminal()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Separate scope ensures app is dropped before terminal cleanup
    let result = {
        let mut app = App::new(app_config).await?;
        app.run(&mut terminal).await
    };

    cleanup_terminal(&mut terminal)?;
    result
}

const HELP_TEXT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "A terminal-based trail running and nutrition tracking application.\n",
    "\n",
    "USAGE:\n",
    "    ",
    env!("CARGO_PKG_NAME"),
    " [OPTIONS]\n",
    "\n",
    "OPTIONS:\n",
    "    -h, --help       Print this help message\n",
    "    -V, --version    Print version information\n",
    "\n",
    "Run with no arguments to launch the interactive TUI.\n",
    "Data is stored in ~/.mountains/ (database, config, markdown backups).\n",
    "\n",
    "Repository: https://github.com/papadavis47/mountains",
);

/// Handles `--version`/`--help` flags before the TUI starts. Exits the process
/// after printing; returns only when no recognized flag is present so the app
/// can launch normally.
fn handle_cli_args() {
    // The app accepts no combinable flags, so only the first argument matters.
    // `nth(1)` skips argv[0] (the binary name); `let ... else` returns early when
    // no argument was given, keeping the happy path unindented.
    let Some(arg) = std::env::args().nth(1) else {
        return;
    };

    match arg.as_str() {
        "-V" | "--version" => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        "-h" | "--help" => {
            println!("{}", HELP_TEXT);
            std::process::exit(0);
        }
        other => {
            eprintln!("error: unrecognized argument '{}'\n", other);
            eprintln!("{}", HELP_TEXT);
            std::process::exit(2);
        }
    }
}

/// Enables raw mode and alternate screen for TUI
fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

/// Restores terminal to normal mode and ensures cursor is visible
fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
