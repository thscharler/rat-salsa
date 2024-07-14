//!
//! Reset the terminal after a crash.
//! Just some small utility ...
//!

use crossterm::cursor::{DisableBlinking, SetCursorStyle};
use crossterm::event::{DisableBracketedPaste, DisableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;

fn main() -> Result<(), anyhow::Error> {
    enable_raw_mode().expect("");
    disable_raw_mode().expect("");
    stdout().execute(DisableBracketedPaste).expect("");
    stdout()
        .execute(SetCursorStyle::DefaultUserShape)
        .expect("");
    stdout().execute(DisableBlinking).expect("");
    // stdout().execute(DisableMouseCapture).expect("");
    stdout().execute(LeaveAlternateScreen).expect("");

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).expect("");
    terminal.clear().expect("");

    Ok(())
}
