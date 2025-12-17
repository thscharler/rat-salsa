use crate::terminal::{SalsaOptions, Terminal};
use crossterm::ExecutableCommand;
use crossterm::cursor::{DisableBlinking, EnableBlinking, SetCursorStyle};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
#[cfg(not(windows))]
use crossterm::event::{PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags};
#[cfg(not(windows))]
use crossterm::terminal::supports_keyboard_enhancement;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use log::debug;
use rat_event::util::set_have_keyboard_enhancement;
#[cfg(feature = "scrolling-regions")]
use ratatui::backend::Backend;
use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::{Frame, TerminalOptions, Viewport};
use std::io;
use std::io::{Stdout, stdout};
#[cfg(feature = "scrolling-regions")]
use std::ops::Range;

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
