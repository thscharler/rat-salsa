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
    KeyboardEnhancementFlags,
};
#[cfg(not(windows))]
use crossterm::event::{PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
#[cfg(not(windows))]
use crossterm::terminal::supports_keyboard_enhancement;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use log::debug;
use rat_event::util::set_have_keyboard_enhancement;
#[cfg(feature = "scrolling-regions")]
use ratatui::backend::Backend;
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::{Frame, TerminalOptions, Viewport};
use std::io;
use std::io::{stdout, Stdout};
#[cfg(feature = "scrolling-regions")]
use std::ops::Range;

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
    fn init(&mut self) -> Result<(), io::Error>;

    /// See [ratatui::Terminal](ratatui::Terminal::get_frame)
    fn get_frame(&mut self) -> Frame<'_>;
    /// See [ratatui::Terminal](ratatui::Terminal::current_buffer_mut)
    fn current_buffer_mut(&mut self) -> &mut Buffer;
    /// See [ratatui::Terminal](ratatui::Terminal::flush)
    fn flush(&mut self) -> Result<(), io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::resize)
    fn resize(&mut self, area: Rect) -> Result<(), io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::hide_cursor)
    fn hide_cursor(&mut self) -> Result<(), io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::show_cursor)
    fn show_cursor(&mut self) -> Result<(), io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::get_cursor_position)
    fn get_cursor_position(&mut self) -> Result<Position, io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::set_cursor_position)
    fn set_cursor_position(&mut self, position: Position) -> Result<(), io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::clear)
    fn clear(&mut self) -> Result<(), io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::swap_buffers)
    fn swap_buffers(&mut self);
    /// See [ratatui::Terminal](ratatui::Terminal::size)
    fn size(&self) -> Result<Size, io::Error>;
    /// See [ratatui::Terminal](ratatui::Terminal::insert_before)
    fn insert_before(
        &mut self,
        height: u16,
        draw_fn: Box<dyn FnOnce(&mut Buffer)>,
    ) -> Result<(), io::Error>;
    /// See [ratatui::Backend](ratatui::backend::Backend::scroll_region_up)
    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(&mut self, region: Range<u16>, line_count: u16) -> Result<(), io::Error>;
    /// See [ratatui::Backend](ratatui::backend::Backend::scroll_region_down)
    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_down(&mut self, region: Range<u16>, line_count: u16) -> Result<(), io::Error>;

    /// Terminal shutdown.
    fn shutdown(&mut self) -> Result<(), io::Error>;

    /// Render the app widget.
    /// Creates the render-context, fetches the frame and calls render.
    fn render(
        &mut self,
        f: &mut dyn FnMut(&mut Frame<'_>) -> Result<(), Error>,
    ) -> Result<(), Error>;
}

/// Terminal options.
pub struct SalsaOptions {
    /// Switch to alternate screen.
    pub alternate_screen: bool,
    /// Enable mouse.
    pub mouse_capture: bool,
    /// See [wikipedia](https://en.wikipedia.org/wiki/Bracketed-paste)
    pub bracketed_paste: bool,
    /// Enable blinking cursor.
    pub cursor_blinking: bool,
    /// Set the cursor-style.
    pub cursor: SetCursorStyle,
    /// ...
    pub keyboard_enhancements: KeyboardEnhancementFlags,
    /// ratatui-options.
    pub ratatui_options: TerminalOptions,
    /// call [Terminal::clear](ratatui::Terminal::clear) on shutdown.
    ///
    /// This might be useful if you set [SalsaOptions::ratatui_options]
    /// to something else then FullScreen.
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
            keyboard_enhancements: KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
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

    // Create a fixed terminal.
    pub fn fixed(area: Rect, clear_on_shutdown: bool) -> Result<Self, io::Error> {
        let options = SalsaOptions {
            alternate_screen: false,
            shutdown_clear: clear_on_shutdown,
            ratatui_options: TerminalOptions {
                viewport: Viewport::Fixed(area),
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
    fn init(&mut self) -> Result<(), io::Error> {
        init(&self.cfg)?;
        Ok(())
    }

    fn get_frame(&mut self) -> Frame<'_> {
        self.term.get_frame()
    }

    fn current_buffer_mut(&mut self) -> &mut Buffer {
        self.term.current_buffer_mut()
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.term.flush()
    }

    fn resize(&mut self, area: Rect) -> Result<(), io::Error> {
        self.term.resize(area)
    }

    fn hide_cursor(&mut self) -> Result<(), io::Error> {
        self.term.hide_cursor()
    }

    fn show_cursor(&mut self) -> Result<(), io::Error> {
        self.term.show_cursor()
    }

    fn get_cursor_position(&mut self) -> Result<Position, io::Error> {
        self.term.get_cursor_position()
    }

    fn set_cursor_position(&mut self, position: Position) -> Result<(), io::Error> {
        self.term.set_cursor_position(position)
    }

    fn clear(&mut self) -> Result<(), io::Error> {
        self.term.clear()
    }

    fn swap_buffers(&mut self) {
        self.term.swap_buffers()
    }

    fn size(&self) -> Result<Size, io::Error> {
        self.term.size()
    }

    fn insert_before(
        &mut self,
        height: u16,
        draw_fn: Box<dyn FnOnce(&mut Buffer)>,
    ) -> Result<(), io::Error> {
        self.term.insert_before(height, draw_fn)
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_up(&mut self, region: Range<u16>, line_count: u16) -> Result<(), io::Error> {
        self.term.backend_mut().scroll_region_up(region, line_count)
    }

    #[cfg(feature = "scrolling-regions")]
    fn scroll_region_down(&mut self, region: Range<u16>, line_count: u16) -> Result<(), io::Error> {
        self.term
            .backend_mut()
            .scroll_region_down(region, line_count)
    }

    fn shutdown(&mut self) -> Result<(), io::Error> {
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
        debug!("enter alternate screen");
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
        stdout().execute(PushKeyboardEnhancementFlags(cfg.keyboard_enhancements))?;
        let enhanced = supports_keyboard_enhancement().unwrap_or_default();
        set_have_keyboard_enhancement(enhanced);
    }
    #[cfg(windows)]
    {
        set_have_keyboard_enhancement(true);
    }

    enable_raw_mode()?;

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
