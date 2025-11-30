#![doc = include_str!("../readme.md")]

use ratatui_crossterm::crossterm::event::Event;
use crate::op::{md_backtab, md_line_break, md_make_header, md_strong, md_surround, md_tab};
use rat_event::{ct_event, flow, HandleEvent, Regular};
use rat_focus::HasFocus;
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;

pub mod dump;
mod format;
mod operations;
pub mod parser;
pub mod styles;
mod util;

pub mod op {
    pub use crate::format::{md_format, reformat};
    pub use crate::operations::{
        md_backtab, md_line_break, md_make_header, md_strong, md_surround, md_tab,
    };
}

/// Event qualifier.
#[derive(Debug)]
pub struct MarkDown {
    text_width: u16,
}

impl Default for MarkDown {
    fn default() -> Self {
        Self { text_width: 65 }
    }
}

impl MarkDown {
    pub fn new(text_width: u16) -> Self {
        Self { text_width }
    }

    pub fn text_width(mut self, width: u16) -> Self {
        self.text_width = width;
        self
    }
}

impl HandleEvent<Event, MarkDown, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &Event, _qualifier: MarkDown) -> TextOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(key press ALT-'1') => md_make_header(self, 1),
                ct_event!(key press ALT-'2') => md_make_header(self, 2),
                ct_event!(key press ALT-'3') => md_make_header(self, 3),
                ct_event!(key press ALT-'4') => md_make_header(self, 4),
                ct_event!(key press ALT-'5') => md_make_header(self, 5),
                ct_event!(key press ALT-'6') => md_make_header(self, 6),

                ct_event!(key press ANY-'*') => md_strong(self, '*'),
                ct_event!(key press ANY-'_') => md_strong(self, '_'),
                ct_event!(key press ANY-'~') => md_strong(self, '~'),
                ct_event!(key press ALT-'c') => md_surround(self, "```\n", None, "\n```", Some(0)),
                ct_event!(key press ALT-'i') => md_surround(self, "![", None, "]()", Some(2)),
                ct_event!(key press ALT-'l') => md_surround(self, "[", None, "]()", Some(2)),
                ct_event!(key press ALT-'k') => md_surround(self, "[", None, "][]", Some(2)),
                ct_event!(key press ALT-'r') => md_surround(self, "[", Some(1), "]: ", None),
                ct_event!(key press ALT-'f') => md_surround(self, "[^1]", Some(4), "", None),

                ct_event!(keycode press Enter) => md_line_break(self),
                ct_event!(keycode press Tab) => md_tab(self),
                ct_event!(keycode press SHIFT-BackTab) => md_backtab(self),
                _ => TextOutcome::Continue,
            });
        }

        self.handle(event, Regular)
    }
}
