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
    let launch_mode = handle_cli_args();

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
        app.apply_launch_mode(launch_mode);
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
    "    -t, --today      Open today's log directly, skipping the startup screen\n",
    "    -s, --simple     Open today's log in a compact quick-entry view\n",
    "                     (miles, elevation, food, notes only)\n",
    "    -h, --help       Print this help message\n",
    "    -V, --version    Print version information\n",
    "\n",
    "Run with no arguments to launch the interactive TUI.\n",
    "Data is stored in ~/.mountains/ (database, config, markdown backups).\n",
    "\n",
    "Repository: https://github.com/papadavis47/mountains",
);

/// What a CLI invocation should do, decided before the TUI starts.
#[derive(Debug, PartialEq)]
enum CliAction {
    Launch(models::LaunchMode),
    PrintVersion,
    PrintHelp,
    Unrecognized(String),
}

/// Maps the (single) CLI argument to an action. The app accepts no combinable
/// flags, so only the first argument matters.
fn parse_cli_arg(arg: Option<&str>) -> CliAction {
    match arg {
        None => CliAction::Launch(models::LaunchMode::Startup),
        Some("-t") | Some("--today") => CliAction::Launch(models::LaunchMode::Today),
        Some("-s") | Some("--simple") => CliAction::Launch(models::LaunchMode::Simple),
        Some("-V") | Some("--version") => CliAction::PrintVersion,
        Some("-h") | Some("--help") => CliAction::PrintHelp,
        Some(other) => CliAction::Unrecognized(other.to_string()),
    }
}

/// Handles CLI flags before the TUI starts. Exits the process after printing
/// version/help/errors; otherwise returns the launch mode for the TUI.
fn handle_cli_args() -> models::LaunchMode {
    // `nth(1)` skips argv[0] (the binary name).
    let arg = std::env::args().nth(1);

    match parse_cli_arg(arg.as_deref()) {
        CliAction::Launch(mode) => mode,
        CliAction::PrintVersion => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        CliAction::PrintHelp => {
            println!("{}", HELP_TEXT);
            std::process::exit(0);
        }
        CliAction::Unrecognized(other) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LaunchMode;

    #[test]
    fn no_arg_launches_startup() {
        assert_eq!(parse_cli_arg(None), CliAction::Launch(LaunchMode::Startup));
    }

    #[test]
    fn today_flags_launch_todays_log() {
        assert_eq!(
            parse_cli_arg(Some("-t")),
            CliAction::Launch(LaunchMode::Today)
        );
        assert_eq!(
            parse_cli_arg(Some("--today")),
            CliAction::Launch(LaunchMode::Today)
        );
    }

    #[test]
    fn simple_flags_launch_simple_mode() {
        assert_eq!(
            parse_cli_arg(Some("-s")),
            CliAction::Launch(LaunchMode::Simple)
        );
        assert_eq!(
            parse_cli_arg(Some("--simple")),
            CliAction::Launch(LaunchMode::Simple)
        );
    }

    #[test]
    fn version_and_help_flags_print() {
        assert_eq!(parse_cli_arg(Some("-V")), CliAction::PrintVersion);
        assert_eq!(parse_cli_arg(Some("--version")), CliAction::PrintVersion);
        assert_eq!(parse_cli_arg(Some("-h")), CliAction::PrintHelp);
        assert_eq!(parse_cli_arg(Some("--help")), CliAction::PrintHelp);
    }

    #[test]
    fn unknown_arg_is_rejected() {
        assert_eq!(
            parse_cli_arg(Some("--bogus")),
            CliAction::Unrecognized("--bogus".to_string())
        );
    }
}
