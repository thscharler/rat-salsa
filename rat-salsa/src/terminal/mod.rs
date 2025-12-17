//!
//! rat-salsa's own Terminal trait to hide some details.
//!
//! This hides the actual implementation for init/shutdown
//! and can be used as dyn Terminal to avoid adding more T's.
//!

#[cfg(feature = "crossterm")]
mod crossterm_terminal;

#[cfg(feature = "crossterm")]
pub use crossterm_terminal::*;

use crate::_private::NonExhaustive;
#[cfg(feature = "crossterm")]
use crossterm::cursor::SetCursorStyle;
#[cfg(feature = "crossterm")]
use crossterm::event::KeyboardEnhancementFlags;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect, Size};
use ratatui::{Frame, TerminalOptions, Viewport};
use std::io;
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
    #[cfg(feature = "crossterm")]
    pub cursor: SetCursorStyle,
    /// ...
    #[cfg(feature = "crossterm")]
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
            #[cfg(feature = "crossterm")]
            cursor: SetCursorStyle::DefaultUserShape,
            #[cfg(feature = "crossterm")]
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
