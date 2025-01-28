#![doc = include_str!("../readme.md")]

use crate::op::{md_backtab, md_format, md_line_break, md_make_header, md_tab};
use rat_event::{ct_event, flow, HandleEvent, Regular};
use rat_focus::HasFocus;
use rat_text::event::TextOutcome;
use rat_text::text_area::TextAreaState;

mod dump;
mod format;
mod operations;
mod parser;
mod styles;
mod util;

pub use styles::{parse_md_styles, MDStyle};
pub mod op {
    pub use crate::format::{md_format, reformat};
    pub use crate::operations::{md_backtab, md_line_break, md_make_header, md_tab};
}
use crate::operations::md_insert_quotes;
pub use dump::{md_dump, md_dump_styles};

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

impl HandleEvent<crossterm::event::Event, MarkDown, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: MarkDown) -> TextOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(key press CONTROL-'f') =>
                    md_format(self, qualifier.text_width as usize, false),
                ct_event!(key press CONTROL-'g') =>
                    md_format(self, qualifier.text_width as usize, true),
                ct_event!(key press CONTROL-'p') => md_dump(self),

                ct_event!(key press ALT-'1') => md_make_header(self, 1),
                ct_event!(key press ALT-'2') => md_make_header(self, 2),
                ct_event!(key press ALT-'3') => md_make_header(self, 3),
                ct_event!(key press ALT-'4') => md_make_header(self, 4),
                ct_event!(key press ALT-'5') => md_make_header(self, 5),
                ct_event!(key press ALT-'6') => md_make_header(self, 6),

                ct_event!(key press ANY-'*') => md_insert_quotes(self, '*'),
                ct_event!(key press ANY-'_') => md_insert_quotes(self, '_'),
                ct_event!(key press ANY-'~') => md_insert_quotes(self, '~'),

                // todo: more
                ct_event!(keycode press Enter) => md_line_break(self),
                ct_event!(keycode press Tab) => md_tab(self),
                ct_event!(keycode press SHIFT-BackTab) => md_backtab(self),
                _ => TextOutcome::Continue,
            });
        }

        self.handle(event, Regular)
    }
}
