//!
//! rat-salsa's own Terminal trait to hide some details.
//!
//! This hides the actual implementation for init/shutdown
//! and can be used as dyn Terminal to avoid adding more T's.
//!

use crate::_private::NonExhaustive;
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    KeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
#[cfg(not(windows))]
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
#[cfg(not(windows))]
use crossterm::terminal::supports_keyboard_enhancement;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use rat_event::util::set_have_keyboard_enhancement;
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, TerminalOptions, Viewport};
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
    Error: 'static + From<io::Error>,
{
    /// Terminal init.
    fn init(&mut self) -> Result<(), Error>;

    /// Terminal shutdown.
    fn shutdown(&mut self) -> Result<(), Error>;

    /// Render the app widget.
    /// Creates the render-context, fetches the frame and calls render.
    fn render(
        &mut self,
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<(), Error>,
    ) -> Result<(), Error>;
}

/// Terminal options.
pub struct SalsaOptions {
    pub alternate_screen: bool,
    pub mouse_capture: bool,
    pub bracketed_paste: bool,
    pub cursor_blinking: bool,
    pub cursor: SetCursorStyle,
    pub keyboard_enhancements: PushKeyboardEnhancementFlags,
    pub ratatui_options: TerminalOptions,
    pub shutdown_clear: bool,

    pub non_exhaustive: NonExhaustive,
}

impl Default for SalsaOptions {
    fn default() -> Self {
        Self {
            alternate_screen: true,
            mouse_capture: true,
            bracketed_paste: true,
            cursor_blinking: true,
            cursor: SetCursorStyle::DefaultUserShape,
            keyboard_enhancements: PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
            ),
            ratatui_options: TerminalOptions {
                viewport: Viewport::Fullscreen,
            },
            shutdown_clear: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Default RenderUI for crossterm.
pub struct CrosstermTerminal {
    term: ratatui::Terminal<CrosstermBackend<Stdout>>,
    cfg: SalsaOptions,
}

impl CrosstermTerminal {
    /// Create a full-screen, alternate screen terminal.
    pub fn new() -> Result<Self, io::Error> {
        set_panic_hook();
        Ok(Self {
            term: ratatui::Terminal::new(CrosstermBackend::new(stdout()))?,
            cfg: Default::default(),
        })
    }

    /// Create an inline terminal.
    pub fn inline(lines: u16, clear_on_shutdown: bool) -> Result<Self, io::Error> {
        let options = SalsaOptions {
            alternate_screen: false,
            shutdown_clear: clear_on_shutdown,
            ratatui_options: TerminalOptions {
                viewport: Viewport::Inline(lines),
            },
            ..Default::default()
        };

        set_panic_hook();
        Ok(Self {
            term: ratatui::Terminal::with_options(
                CrosstermBackend::new(stdout()),
                options.ratatui_options.clone(),
            )?,
            cfg: options,
        })
    }

    pub fn with_options(options: SalsaOptions) -> Result<Self, io::Error> {
        set_panic_hook();
        Ok(Self {
            term: ratatui::Terminal::with_options(
                CrosstermBackend::new(stdout()),
                options.ratatui_options.clone(),
            )?,
            cfg: options,
        })
    }
}

impl<Error> Terminal<Error> for CrosstermTerminal
where
    Error: 'static + From<io::Error>,
{
    fn init(&mut self) -> Result<(), Error> {
        init(&self.cfg)?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Error> {
        shutdown(&self.cfg)?;
        Ok(())
    }

    fn render(
        &mut self,
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut res = Ok(());
        _ = self.term.hide_cursor();
        self.term.draw(|frame| res = f(frame))?;
        res
    }
}

/// Sets a panic hook that restores the terminal before panicking.
///
/// Replaces the panic hook with a one that will restore the terminal state before calling the
/// original panic hook. This ensures that the terminal is left in a good state when a panic occurs.
fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        _ = shutdown(&SalsaOptions {
            alternate_screen: true,
            ..Default::default()
        });
        hook(info);
    }));
}

fn init(cfg: &SalsaOptions) -> io::Result<()> {
    if cfg.alternate_screen {
        stdout().execute(EnterAlternateScreen)?;
    }
    if cfg.mouse_capture {
        stdout().execute(EnableMouseCapture)?;
    }
    if cfg.bracketed_paste {
        stdout().execute(EnableBracketedPaste)?;
    }
    if cfg.cursor_blinking {
        stdout().execute(EnableBlinking)?;
    }
    stdout().execute(cfg.cursor)?;
    #[cfg(not(windows))]
    {
        stdout().execute(cfg.keyboard_enhancements)?;

        let enhanced = supports_keyboard_enhancement().unwrap_or_default();
        set_have_keyboard_enhancement(enhanced);
    }
    #[cfg(windows)]
    {
        set_have_keyboard_enhancement(true);
    }
    enable_raw_mode()?;
    // self.term.clear()?;

    Ok(())
}

fn shutdown(cfg: &SalsaOptions) -> io::Result<()> {
    disable_raw_mode()?;

    #[cfg(not(windows))]
    stdout().execute(PopKeyboardEnhancementFlags)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    if cfg.cursor_blinking {
        stdout().execute(DisableBlinking)?;
    }
    if cfg.bracketed_paste {
        stdout().execute(DisableBracketedPaste)?;
    }
    if cfg.mouse_capture {
        stdout().execute(DisableMouseCapture)?;
    }
    if cfg.alternate_screen {
        stdout().execute(LeaveAlternateScreen)?;
    }
    if cfg.shutdown_clear {
        stdout().execute(Clear(ClearType::CurrentLine))?;
        stdout().execute(Clear(ClearType::FromCursorDown))?;
    }

    Ok(())
}
