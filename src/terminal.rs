//!
//! rat-salsa's own Terminal trait to hide some details.
//!
//! This hides the actual implementation for init/shutdown
//! and can be used as dyn Terminal to avoid adding more T's.
//!

use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Frame;
use std::fmt::Debug;
use std::io;
use std::io::{stdout, Stdout};

/// Encapsulates Terminal and Backend.
///
/// This is used as dyn Trait to hide the Background type parameter.
///
/// If you want to send other than the default Commands to the backend,
/// implement this trait.
pub trait Terminal<Error>
where
    Error: 'static + Send + Debug,
{
    /// Terminal init.
    fn init(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>;

    /// Terminal shutdown.
    fn shutdown(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>;

    /// Render the app widget.
    ///
    /// Creates the render-context, fetches the frame and calls render.
    #[allow(clippy::needless_lifetimes)]
    #[allow(clippy::type_complexity)]
    fn render(
        &mut self,
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Error: From<io::Error>;
}

/// Default RenderUI for crossterm.
#[derive(Debug)]
pub struct CrosstermTerminal {
    term: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl CrosstermTerminal {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            term: ratatui::Terminal::new(CrosstermBackend::new(stdout()))?,
        })
    }
}

impl<Error> Terminal<Error> for CrosstermTerminal
where
    Error: 'static + Send + Debug,
{
    fn init(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>,
    {
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(EnableMouseCapture)?;
        stdout().execute(EnableBracketedPaste)?;
        stdout().execute(EnableBlinking)?;
        stdout().execute(SetCursorStyle::BlinkingBar)?;
        enable_raw_mode()?;

        self.term.clear()?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>,
    {
        disable_raw_mode()?;
        stdout().execute(SetCursorStyle::DefaultUserShape)?;
        stdout().execute(DisableBlinking)?;
        stdout().execute(DisableBracketedPaste)?;
        stdout().execute(DisableMouseCapture)?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    fn render(
        &mut self,
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        Error: From<io::Error>,
    {
        let mut res = Ok(());
        _ = self.term.hide_cursor();
        self.term.draw(|frame| res = f(frame))?;
        res
    }
}
