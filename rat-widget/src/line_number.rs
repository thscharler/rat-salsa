//!
//! Line numbers widget.
//!
//!
//! Render line numbers in sync with a text area.
//! ```
//! # use ratatui_core::buffer::Buffer;
//! # use ratatui_core::layout::Rect;
//! # use ratatui_core::symbols::border::EMPTY;
//! # use ratatui_core::widgets::{StatefulWidget};
//! # use ratatui_widgets::block::{Block};
//! # use ratatui_widgets::borders::{Borders};
//! use rat_text::line_number::{LineNumberState, LineNumbers};
//! # use rat_text::text_area::TextAreaState;
//!
//! # struct State {textarea: TextAreaState, line_numbers: LineNumberState}
//! # let mut state = State {textarea: Default::default(),line_numbers: Default::default()};
//! # let mut buf = Buffer::default();
//! # let buf = &mut buf;
//! # let area = Rect::default();
//!
//! LineNumbers::new()
//!     .block(
//!         Block::new()
//!             .borders(Borders::TOP | Borders::BOTTOM)
//!             .border_set(EMPTY),
//!     )
//! .with_textarea(&state.textarea)
//! .render(area, buf, &mut state.line_numbers);
//! ```

pub use rat_text::line_number::{LineNumberState, LineNumberStyle, LineNumbers};
