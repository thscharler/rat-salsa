use crate::mdedit_parts::dump::{md_dump, md_dump_styles};
use crate::mdedit_parts::format::md_format;
use crate::mdedit_parts::operations::{md_backtab, md_line_break, md_make_header, md_tab};
use rat_salsa::event::ct_event;
use rat_widget::event::{flow, HandleEvent, Regular, TextOutcome};
use rat_widget::focus::HasFocusFlag;
use rat_widget::text::upos_type;
use rat_widget::textarea::TextAreaState;
use unicode_segmentation::UnicodeSegmentation;

pub mod dump;
pub mod format;
pub mod operations;
pub mod parser;
pub mod styles;
pub mod test_markdown;

// qualifier for markdown-editing.
#[derive(Debug)]
pub struct MarkDown;

impl HandleEvent<crossterm::event::Event, MarkDown, TextOutcome> for TextAreaState {
    fn handle(&mut self, event: &crossterm::event::Event, qualifier: MarkDown) -> TextOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(key press ALT-'f') => md_format(self, false),
                ct_event!(key press ALT_SHIFT-'F') => md_format(self, true),
                ct_event!(key press ALT-'d') => md_dump(self),
                ct_event!(key press ALT-'s') => md_dump_styles(self),

                ct_event!(key press ALT-'1') => md_make_header(self, 1),
                ct_event!(key press ALT-'2') => md_make_header(self, 2),
                ct_event!(key press ALT-'3') => md_make_header(self, 3),
                ct_event!(key press ALT-'4') => md_make_header(self, 4),
                ct_event!(key press ALT-'5') => md_make_header(self, 5),
                ct_event!(key press ALT-'6') => md_make_header(self, 6),

                ct_event!(keycode press Enter) => md_line_break(self),
                ct_event!(keycode press Tab) => md_tab(self),
                ct_event!(keycode press SHIFT-BackTab) => md_backtab(self),
                _ => TextOutcome::Continue,
            });
        }

        self.handle(event, Regular)
    }
}

/// Length as grapheme count, excluding line breaks.
fn str_line_len(s: &str) -> upos_type {
    let it = s.graphemes(true);
    it.filter(|c| *c != "\n" && *c != "\r\n").count() as upos_type
}

/// Length as grapheme count.
fn str_len(s: &str) -> upos_type {
    let it = s.graphemes(true);
    it.count() as upos_type
}
