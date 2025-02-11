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
#[cfg(not(windows))]
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
#[cfg(not(windows))]
use crossterm::terminal::supports_keyboard_enhancement;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use rat_widget::event::util::set_have_keyboard_enhancement;
use ratatui_core::terminal::Frame;
use ratatui_crossterm::CrosstermBackend;
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
    Error: 'static + Send,
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
    /// Returns the frame number of the rendered frame or any error.
    #[allow(clippy::needless_lifetimes)]
    #[allow(clippy::type_complexity)]
    fn render(
        &mut self,
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<usize, Error>,
    ) -> Result<usize, Error>
    where
        Error: From<io::Error>;
}

/// Default RenderUI for crossterm.
#[derive(Debug)]
pub struct CrosstermTerminal {
    term: ratatui_core::terminal::Terminal<CrosstermBackend<Stdout>>,
}

impl CrosstermTerminal {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            term: ratatui_core::terminal::Terminal::new(CrosstermBackend::new(stdout()))?,
        })
    }
}

impl<Error> Terminal<Error> for CrosstermTerminal
where
    Error: 'static + Send,
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
        #[cfg(not(windows))]
        {
            stdout().execute(PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
            ))?;

            let enhanced = supports_keyboard_enhancement().unwrap_or_default();
            set_have_keyboard_enhancement(enhanced);
        }
        #[cfg(windows)]
        {
            set_have_keyboard_enhancement(true);
        }

        self.term.clear()?;

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Error>
    where
        Error: From<io::Error>,
    {
        #[cfg(not(windows))]
        stdout().execute(PopKeyboardEnhancementFlags)?;
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
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<usize, Error>,
    ) -> Result<usize, Error>
    where
        Error: From<io::Error>,
    {
        let mut res = Ok(0);
        _ = self.term.hide_cursor();
        self.term.draw(|frame| res = f(frame))?;
        res
    }
}
