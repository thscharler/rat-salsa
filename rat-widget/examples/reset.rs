//!
//! Reset the terminal after a crash.
//! Just some small utility ...
//!

use ratatui_core::terminal::Terminal;
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::ExecutableCommand;
use ratatui_crossterm::crossterm::cursor::{DisableBlinking, SetCursorStyle};
use ratatui_crossterm::crossterm::event::DisableBracketedPaste;
use ratatui_crossterm::crossterm::terminal::{
    LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
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
