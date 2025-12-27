use crate::_private::NonExhaustive;
use crate::poll::PollEvents;
use crossbeam::channel::TryRecvError;
use ratatui_core::layout::Rect;
use ratatui_core::terminal::{Terminal, TerminalOptions, Viewport};
use ratatui_crossterm::CrosstermBackend;
use ratatui_crossterm::crossterm::cursor::SetCursorStyle;
use ratatui_crossterm::crossterm::event::KeyboardEnhancementFlags;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{Stdout, stdout};
use std::rc::Rc;

/// Captures some parameters for [crate::run_tui()].
pub struct RunConfig<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    /// This is the renderer that connects to the backend, and calls out
    /// for rendering the application.
    ///
    /// Defaults to RenderCrossterm.
    pub(crate) term: Rc<RefCell<Terminal<CrosstermBackend<Stdout>>>>,
    /// List of all event-handlers for the application.
    ///
    /// Defaults to PollTimers, PollCrossterm, PollTasks. Add yours here.
    pub(crate) poll: Vec<Box<dyn PollEvents<Event, Error>>>,
    /// Terminal init flags.
    pub(crate) term_init: TermInit,
}

impl<Event, Error> Debug for RunConfig<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunConfig")
            .field("term", &self.term)
            .field("poll", &"...")
            .field("term_init", &self.term_init)
            .finish()
    }
}

impl<Event, Error> RunConfig<Event, Error>
where
    Event: 'static,
    Error: 'static + From<io::Error> + From<TryRecvError>,
{
    /// Defaults.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self, Error> {
        Ok(Self {
            term: Rc::new(RefCell::new(Terminal::new(
                CrosstermBackend::new(stdout()),
            )?)),
            poll: Default::default(),
            term_init: Default::default(),
        })
    }

    /// New with terminal.
    pub fn new(term: Terminal<CrosstermBackend<Stdout>>) -> Self {
        Self {
            term: Rc::new(RefCell::new(term)),
            poll: Default::default(),
            term_init: Default::default(),
        }
    }

    /// Initialize as an inline terminal
    pub fn inline(lines: u16, clear_on_shutdown: bool) -> Result<Self, io::Error> {
        Ok(Self {
            term: Rc::new(RefCell::new(Terminal::with_options(
                CrosstermBackend::new(stdout()),
                TerminalOptions {
                    viewport: Viewport::Inline(lines),
                },
            )?)),
            poll: Default::default(),
            term_init: TermInit {
                alternate_screen: false,
                clear_area: clear_on_shutdown,
                ..Default::default()
            },
        })
    }

    /// Initialize for a fixed portion of the actual terminal.
    pub fn fixed(area: Rect, clear_on_shutdown: bool) -> Result<Self, io::Error> {
        Ok(Self {
            term: Rc::new(RefCell::new(Terminal::with_options(
                CrosstermBackend::new(stdout()),
                TerminalOptions {
                    viewport: Viewport::Fixed(area),
                },
            )?)),
            poll: Default::default(),
            term_init: TermInit {
                alternate_screen: false,
                clear_area: clear_on_shutdown,
                ..Default::default()
            },
        })
    }

    /// Add one more poll impl.
    pub fn poll(mut self, poll: impl PollEvents<Event, Error> + 'static) -> Self {
        self.poll.push(Box::new(poll));
        self
    }

    /// Set flags for terminal init/shutdown.
    pub fn term_init(mut self, term_init: TermInit) -> Self {
        self.term_init = term_init;
        self
    }
}

#[derive(Clone, Copy)]
pub struct TermInit {
    /// Don't do any init/shutdown.
    /// Will be done by main().
    pub manual: bool,
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
    /// Key encoding options.
    pub keyboard_enhancements: KeyboardEnhancementFlags,
    /// Clear the configured terminal at shutdown.
    pub clear_area: bool,

    pub non_exhaustive: NonExhaustive,
}

impl Debug for TermInit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TermInit")
            .field("manual", &self.manual)
            .field("alternate_screen", &self.alternate_screen)
            .field("mouse_capture", &self.mouse_capture)
            .field("bracketed_paste", &self.bracketed_paste)
            .field("cursor_blinking", &self.cursor_blinking)
            .field(
                "cursor",
                &match self.cursor {
                    SetCursorStyle::DefaultUserShape => "DefaultUserShape",
                    SetCursorStyle::BlinkingBlock => "BlinkingBlock",
                    SetCursorStyle::SteadyBlock => "SteadyBlock",
                    SetCursorStyle::BlinkingUnderScore => "BlinkingUnderScore",
                    SetCursorStyle::SteadyUnderScore => "SteadyUnderScore",
                    SetCursorStyle::BlinkingBar => "BlinkingBar",
                    SetCursorStyle::SteadyBar => "SteadyBar",
                },
            )
            .field("keyboard_enhancements", &self.keyboard_enhancements)
            .finish()
    }
}

impl Default for TermInit {
    fn default() -> Self {
        Self {
            manual: false,
            alternate_screen: true,
            mouse_capture: true,
            bracketed_paste: true,
            cursor_blinking: true,
            cursor: SetCursorStyle::DefaultUserShape,
            keyboard_enhancements: KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES,
            clear_area: false,
            non_exhaustive: NonExhaustive,
        }
    }
}
