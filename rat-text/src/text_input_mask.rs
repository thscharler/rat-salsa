//! Text input widget with an input mask.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection with keyboard + mouse
//! * Scrolls with the cursor.
//! * Modes for focus and valid.
//! * Localization with [format_num_pattern::NumberSymbols]
//!
//! * Accepts an input mask:
//!   * `0`: can enter digit, display as 0
//!   * `9`: can enter digit, display as space
//!   * `#`: digit, plus or minus sign, display as space
//!   * `-`: sign
//!   * `+`: sign, positive is '+', negative is '-', not localized.
//!   * `.` and `,`: decimal and grouping separators
//!
//!   * `H`: must enter a hex digit, display as 0
//!   * `h`: can enter a hex digit, display as space
//!   * `O`: must enter an octal digit, display as 0
//!   * `o`: can enter an octal digit, display as space
//!   * `D`: must enter a decimal digit, display as 0
//!   * `d`: can enter a decimal digit, display as space
//!
//!   * `l`: can enter letter, display as space
//!   * `a`: can enter letter or digit, display as space
//!   * `c`: can enter character or space, display as space
//!   * `_`: anything, display as space
//!
//!   * `<space>` separator character move the cursor when entered.
//!   * `\`: escapes the following character and uses it as a separator.
//!   * everything else must be escaped
//!
//! * Accepts a display overlay used instead of the default chars of the input mask.
//!
//! ```rust ignore
//! use ratatui::widgets::StatefulWidget;
//! use rat_input::masked_input::{MaskedInput, MaskedInputState};
//!
//! let date_focused = false;
//! let creditcard_focused = true;
//! let area = Rect::default();
//! let buf = Buffer::default();
//!
//! let mut date_state = MaskedInputState::new();
//! date_state.set_mask("99\\/99\\/9999")?;
//!
//! let w_date = MaskedInput::default();
//! w_date.render(area, &mut buf, &mut date_state);
//! if date_focused {
//!     frame.set_cursor(date_state.cursor.x, date_state.cursor.y);
//! }
//!
//! let mut creditcard_state = MaskedInputState::new();
//! creditcard_state.set_mask("dddd dddd dddd dddd")?;
//!
//! let w_creditcard = MaskedInput::default();
//! w_creditcard.render(area, &mut buf, &mut creditcard_state);
//! if creditcard_focused {
//!     frame.set_cursor(creditcard_state.cursor.x, creditcard_state.cursor.y);
//! }
//!
//! ```
//!
//! The visual cursor must be set separately after rendering.
//! It is accessible as [TextInputState::screen_cursor()] after rendering.
//!
//! Event handling by calling the freestanding fn [handle_events].
//! There's [handle_mouse_events] if you want to override the default key bindings but keep
//! the mouse behaviour.
//!

pub mod mask_op;
pub(crate) mod mask_token;
pub(crate) mod masked_graphemes;

use crate::_private::NonExhaustive;
use crate::clipboard::{Clipboard, global_clipboard};
use crate::core::{TextCore, TextString};
use crate::event::{ReadOnly, TextOutcome};
use crate::glyph2::{Glyph2, GlyphIter2, TextWrap2};
use crate::text_input::TextInputState;
use crate::text_input_mask::mask_token::{EditDirection, Mask, MaskToken};
use crate::text_input_mask::masked_graphemes::MaskedGraphemes;
use crate::text_store::TextStore;
use crate::undo_buffer::{UndoBuffer, UndoEntry, UndoVec};
use crate::{
    Grapheme, HasScreenCursor, TextError, TextFocusGained, TextFocusLost, TextPosition, TextRange,
    TextStyle, ipos_type, upos_type,
};
use crossterm::event::KeyModifiers;
use format_num_pattern::NumberSymbols;
use rat_event::util::MouseFlags;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{RelocatableState, relocate_area, relocate_dark_offset};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::cell::Cell;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt;
use std::iter::once;
use std::ops::Range;
use std::rc::Rc;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

/// Text input widget with input mask.
///
/// # Stateful
/// This widget implements [`StatefulWidget`], you can use it with
/// [`MaskedInputState`] to handle common actions.
#[derive(Debug, Default, Clone)]
pub struct MaskedInput<'a> {
    compact: bool,
    block: Option<Block<'a>>,
    style: Style,
    focus_style: Option<Style>,
    select_style: Option<Style>,
    invalid_style: Option<Style>,
    text_style: HashMap<usize, Style>,
    on_focus_gained: TextFocusGained,
    on_focus_lost: TextFocusLost,
}

/// State & event-handling.
#[derive(Debug)]
pub struct MaskedInputState {
    /// The whole area with block.
    /// __read only__ renewed with each render.
    pub area: Rect,
    /// Area inside a possible block.
    /// __read only__ renewed with each render.
    pub inner: Rect,
    /// Rendered dimension. This may differ from (inner.width, inner.height)
    /// if the text area has been relocated.
    pub rendered: Size,
    /// Widget has been rendered in compact mode.
    /// __read only: renewed with each render.
    pub compact: bool,

    /// Display offset
    /// __read+write__
    pub offset: upos_type,
    /// Dark offset due to clipping. Always set during rendering.
    /// __read only__ ignore this value.
    pub dark_offset: (u16, u16),
    /// __read only__ use [scroll_cursor_to_visible](MaskedInputState::scroll_cursor_to_visible)
    pub scroll_to_cursor: Rc<Cell<bool>>,

    /// Editing core
    pub value: TextCore<TextString>,
    /// Editing core
    /// __read only__
    pub sym: Option<NumberSymbols>,
    /// Editing core
    /// __read only__
    pub mask: Vec<MaskToken>,

    /// Display as invalid.
    /// __read only__ use [set_invalid](MaskedInputState::set_invalid)
    pub invalid: bool,
    /// The next user edit clears the text for doing any edit.
    /// It will reset this flag. Other interactions may reset this flag too.
    /// __read only__ use [set_overwrite](MaskedInputState::set_overwrite)
    pub overwrite: Rc<Cell<bool>>,
    /// Focus behaviour.
    /// __read only__ use [on_focus_gained](MaskedInput::on_focus_gained)
    pub on_focus_gained: Rc<Cell<TextFocusGained>>,
    /// Focus behaviour.
    /// __read only__ use [on_focus_lost](MaskedInput::on_focus_lost)
    pub on_focus_lost: Rc<Cell<TextFocusLost>>,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Mouse selection in progress.
    /// __read+write__
    pub mouse: MouseFlags,

    /// Construct with `..Default::default()`
    pub non_exhaustive: NonExhaustive,
}

impl<'a> MaskedInput<'a> {
    /// New widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a compact form of the content without unnecessary spaces,
    /// if this widget is not focused.
    #[inline]
    pub fn compact(mut self, show_compact: bool) -> Self {
        self.compact = show_compact;
        self
    }

    /// Set the combined style.
    #[inline]
    pub fn styles_opt(self, styles: Option<TextStyle>) -> Self {
        if let Some(styles) = styles {
            self.styles(styles)
        } else {
            self
        }
    }

    /// Set the combined style.
    #[inline]
    pub fn styles(mut self, styles: TextStyle) -> Self {
        self.style = styles.style;
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if styles.invalid.is_some() {
            self.invalid_style = styles.invalid;
        }
        if let Some(of) = styles.on_focus_gained {
            self.on_focus_gained = of;
        }
        if let Some(of) = styles.on_focus_lost {
            self.on_focus_lost = of;
        }
        if let Some(border_style) = styles.border_style {
            self.block = self.block.map(|v| v.border_style(border_style));
        }
        self.block = self.block.map(|v| v.style(self.style));
        if styles.block.is_some() {
            self.block = styles.block;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base text style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = Some(style.into());
        self
    }

    /// Style for selection
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = Some(style.into());
        self
    }

    /// Style for the invalid indicator.
    /// This is patched onto either base_style or focus_style
    #[inline]
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.invalid_style = Some(style.into());
        self
    }

    /// Indexed text-style.
    ///
    /// Use [TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style_idx(mut self, idx: usize, style: Style) -> Self {
        self.text_style.insert(idx, style);
        self
    }

    /// List of text-styles.
    ///
    /// Use [MaskedInputState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style<T: IntoIterator<Item = Style>>(mut self, styles: T) -> Self {
        for (i, s) in styles.into_iter().enumerate() {
            self.text_style.insert(i, s);
        }
        self
    }

    /// Map of style_id -> text_style.
    ///
    /// Use [TextAreaState::add_style()] to refer a text range to
    /// one of these styles.
    pub fn text_style_map<T: Into<Style>>(mut self, styles: HashMap<usize, T>) -> Self {
        for (i, s) in styles.into_iter() {
            self.text_style.insert(i, s.into());
        }
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_gained(mut self, of: TextFocusGained) -> Self {
        self.on_focus_gained = of;
        self
    }

    /// Focus behaviour
    #[inline]
    pub fn on_focus_lost(mut self, of: TextFocusLost) -> Self {
        self.on_focus_lost = of;
        self
    }
}

impl<'a> StatefulWidget for &MaskedInput<'a> {
    type State = MaskedInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl StatefulWidget for MaskedInput<'_> {
    type State = MaskedInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(
    widget: &MaskedInput<'_>,
    area: Rect,

    buf: &mut Buffer,
    state: &mut MaskedInputState,
) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);
    state.rendered = state.inner.as_size();
    state.compact = widget.compact;
    state.on_focus_gained.set(widget.on_focus_gained);
    state.on_focus_lost.set(widget.on_focus_lost);

    if state.scroll_to_cursor.get() {
        let c = state.cursor();
        let o = state.offset();
        let mut no = if c < o {
            c
        } else if c >= o + state.rendered.width as upos_type {
            c.saturating_sub(state.rendered.width as upos_type)
        } else {
            o
        };
        // correct by one at right margin. block cursors appear as part of the
        // right border otherwise.
        if c == no + state.rendered.width as upos_type {
            no = no.saturating_add(1);
        }
        state.set_offset(no);
    }

    let style = widget.style;
    let focus_style = if let Some(focus_style) = widget.focus_style {
        focus_style
    } else {
        style
    };
    let select_style = if let Some(select_style) = widget.select_style {
        select_style
    } else {
        Style::default().black().on_yellow()
    };
    let invalid_style = if let Some(invalid_style) = widget.invalid_style {
        invalid_style
    } else {
        Style::default().red()
    };

    let (style, select_style) = if state.focus.get() {
        if state.invalid {
            (
                style.patch(focus_style).patch(invalid_style),
                style
                    .patch(focus_style)
                    .patch(select_style)
                    .patch(invalid_style),
            )
        } else {
            (
                style.patch(focus_style),
                style.patch(focus_style).patch(select_style),
            )
        }
    } else {
        if state.invalid {
            (
                style.patch(invalid_style),
                style.patch(select_style).patch(invalid_style),
            )
        } else {
            (style, style.patch(select_style))
        }
    };

    // set base style
    if let Some(block) = &widget.block {
        block.render(area, buf);
    } else {
        buf.set_style(area, style);
    }

    if state.inner.width == 0 || state.inner.height == 0 {
        // noop
        return;
    }

    let ox = state.offset() as u16;
    // this is just a guess at the display-width
    let show_range = {
        let start = ox as upos_type;
        let end = min(start + state.inner.width as upos_type, state.len());
        state.bytes_at_range(start..end)
    };
    let selection = state.selection();
    let mut styles = Vec::new();

    for g in state.glyphs2() {
        if g.screen_width() > 0 {
            let mut style = style;
            styles.clear();
            state
                .value
                .styles_at_page(g.text_bytes().start, show_range.clone(), &mut styles);
            for style_nr in &styles {
                if let Some(s) = widget.text_style.get(style_nr) {
                    style = style.patch(*s);
                }
            }
            // selection
            if selection.contains(&g.pos().x) {
                style = style.patch(select_style);
            };

            // relative screen-pos of the glyph
            let screen_pos = g.screen_pos();

            // render glyph
            if let Some(cell) =
                buf.cell_mut((state.inner.x + screen_pos.0, state.inner.y + screen_pos.1))
            {
                cell.set_symbol(g.glyph());
                cell.set_style(style);
            }
            // clear the reset of the cells to avoid interferences.
            for d in 1..g.screen_width() {
                if let Some(cell) = buf.cell_mut((
                    state.inner.x + screen_pos.0 + d,
                    state.inner.y + screen_pos.1,
                )) {
                    cell.reset();
                    cell.set_style(style);
                }
            }
        }
    }
}

impl Clone for MaskedInputState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            rendered: self.rendered,
            compact: self.compact,
            offset: self.offset,
            dark_offset: self.dark_offset,
            scroll_to_cursor: Rc::new(Cell::new(self.scroll_to_cursor.get())),
            value: self.value.clone(),
            sym: self.sym,
            mask: self.mask.clone(),
            invalid: self.invalid,
            overwrite: Rc::new(Cell::new(self.overwrite.get())),
            on_focus_gained: Rc::new(Cell::new(self.on_focus_gained.get())),
            on_focus_lost: Rc::new(Cell::new(self.on_focus_lost.get())),
            focus: self.focus_cb(FocusFlag::named(self.focus.name())),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for MaskedInputState {
    fn default() -> Self {
        let core = TextCore::new(
            Some(Box::new(UndoVec::new(99))),
            Some(Box::new(global_clipboard())),
        );

        let mut z = Self {
            area: Default::default(),
            inner: Default::default(),
            rendered: Default::default(),
            compact: Default::default(),
            offset: Default::default(),
            dark_offset: Default::default(),
            scroll_to_cursor: Default::default(),
            value: core,
            sym: None,
            mask: Default::default(),
            invalid: Default::default(),
            overwrite: Default::default(),
            on_focus_gained: Default::default(),
            on_focus_lost: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        };
        z.focus = z.focus_cb(FocusFlag::default());
        z
    }
}

impl MaskedInputState {
    fn focus_cb(&self, flag: FocusFlag) -> FocusFlag {
        let on_focus_lost = self.on_focus_lost.clone();
        let cursor = self.value.shared_cursor();
        let scroll_cursor_to_visible = self.scroll_to_cursor.clone();
        flag.on_lost(move || match on_focus_lost.get() {
            TextFocusLost::None => {}
            TextFocusLost::Position0 => {
                scroll_cursor_to_visible.set(true);
                let mut new_cursor = cursor.get();
                new_cursor.cursor.x = 0;
                new_cursor.anchor.x = 0;
                cursor.set(new_cursor);
            }
        });
        let on_focus_gained = self.on_focus_gained.clone();
        let overwrite = self.overwrite.clone();
        let cursor = self.value.shared_cursor();
        let scroll_cursor_to_visible = self.scroll_to_cursor.clone();
        flag.on_gained(move || match on_focus_gained.get() {
            TextFocusGained::None => {}
            TextFocusGained::Overwrite => {
                overwrite.set(true);
            }
            TextFocusGained::SelectAll => {
                scroll_cursor_to_visible.set(true);
                let mut new_cursor = cursor.get();
                new_cursor.anchor = TextPosition::new(0, 0);
                new_cursor.cursor = TextPosition::new(0, 1);
                cursor.set(new_cursor);
            }
        });

        flag
    }
}

impl HasFocus for MaskedInputState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn navigable(&self) -> Navigation {
        let sel = self.selection();

        let has_next = self
            .next_section_range(sel.end)
            .map(|v| !v.is_empty())
            .is_some();
        let has_prev = self
            .prev_section_range(sel.start.saturating_sub(1))
            .map(|v| !v.is_empty())
            .is_some();

        if has_next {
            if has_prev {
                Navigation::Reach
            } else {
                Navigation::ReachLeaveFront
            }
        } else {
            if has_prev {
                Navigation::ReachLeaveBack
            } else {
                Navigation::Regular
            }
        }
    }
}

impl MaskedInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.focus = z.focus_cb(FocusFlag::named(name));
        z
    }

    /// With localized symbols for number formatting.
    #[inline]
    pub fn with_symbols(mut self, sym: NumberSymbols) -> Self {
        self.set_num_symbols(sym);
        self
    }

    /// With input mask.
    pub fn with_mask<S: AsRef<str>>(mut self, mask: S) -> Result<Self, fmt::Error> {
        self.set_mask(mask.as_ref())?;
        Ok(self)
    }

    /// Set symbols for number display.
    ///
    /// These are only used for rendering and to map user input.
    /// The value itself uses ".", "," and "-".
    #[inline]
    pub fn set_num_symbols(&mut self, sym: NumberSymbols) {
        self.sym = Some(sym);
    }

    fn dec_sep(&self) -> char {
        if let Some(sym) = &self.sym {
            sym.decimal_sep
        } else {
            '.'
        }
    }

    fn grp_sep(&self) -> char {
        if let Some(sym) = &self.sym {
            // fallback for empty grp-char.
            // it would be really ugly, if we couldn't keep
            //   mask-idx == grapheme-idx
            sym.decimal_grp.unwrap_or(' ')
        } else {
            ','
        }
    }

    fn neg_sym(&self) -> char {
        if let Some(sym) = &self.sym {
            sym.negative_sym
        } else {
            '-'
        }
    }

    fn pos_sym(&self) -> char {
        if let Some(sym) = &self.sym {
            sym.positive_sym
        } else {
            ' '
        }
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below.
    ///
    /// * `0`: can enter digit, display as 0
    /// * `9`: can enter digit, display as space
    /// * `#`: digit, plus or minus sign, display as space
    /// * `+`: sign. display '+' for positive
    /// * `-`: sign. display ' ' for positive
    /// * `.` and `,`: decimal and grouping separators
    ///
    /// * `H`: must enter a hex digit, display as 0
    /// * `h`: can enter a hex digit, display as space
    /// * `O`: must enter an octal digit, display as 0
    /// * `o`: can enter an octal digit, display as space
    /// * `D`: must enter a decimal digit, display as 0
    /// * `d`: can enter a decimal digit, display as space
    ///
    /// * `l`: can enter letter, display as space
    /// * `a`: can enter letter or digit, display as space
    /// * `c`: can enter character or space, display as space
    /// * `_`: anything, display as space
    ///
    /// * `SPACE`: separator character move the cursor when entered.
    /// * `\`: escapes the following character and uses it as a separator.
    /// * all other ascii characters a reserved.
    ///
    /// Inspired by <https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da>
    #[inline]
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) -> Result<(), fmt::Error> {
        self.mask = Self::parse_mask(s.as_ref())?;
        self.clear();
        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn parse_mask(mask_str: &str) -> Result<Vec<MaskToken>, fmt::Error> {
        let mut out = Vec::<MaskToken>::new();

        let mut start_sub = 0;
        let mut start_sec = 0;
        let mut sec_id = 0;
        let mut last_mask = Mask::None;
        let mut dec_dir = EditDirection::Rtol;
        let mut esc = false;
        let mut idx = 0;
        for m in mask_str.graphemes(true).chain(once("")) {
            let mask = if esc {
                esc = false;
                Mask::Separator(Box::from(m))
            } else {
                match m {
                    "0" => Mask::Digit0(dec_dir),
                    "9" => Mask::Digit(dec_dir),
                    "#" => Mask::Numeric(dec_dir),
                    "." => Mask::DecimalSep,
                    "," => Mask::GroupingSep,
                    "-" => Mask::Sign,
                    "+" => Mask::Plus,
                    "h" => Mask::Hex,
                    "H" => Mask::Hex0,
                    "o" => Mask::Oct,
                    "O" => Mask::Oct0,
                    "d" => Mask::Dec,
                    "D" => Mask::Dec0,
                    "l" => Mask::Letter,
                    "a" => Mask::LetterOrDigit,
                    "c" => Mask::LetterDigitSpace,
                    "_" => Mask::AnyChar,
                    "" => Mask::None,
                    " " => Mask::Separator(Box::from(m)),
                    "\\" => {
                        esc = true;
                        continue;
                    }
                    _ => return Err(fmt::Error),
                }
            };

            match mask {
                Mask::Digit0(_)
                | Mask::Digit(_)
                | Mask::Numeric(_)
                | Mask::GroupingSep
                | Mask::Sign
                | Mask::Plus => {
                    // no change
                }
                Mask::DecimalSep => {
                    dec_dir = EditDirection::Ltor;
                }
                Mask::Hex0
                | Mask::Hex
                | Mask::Oct0
                | Mask::Oct
                | Mask::Dec0
                | Mask::Dec
                | Mask::Letter
                | Mask::LetterOrDigit
                | Mask::LetterDigitSpace
                | Mask::AnyChar
                | Mask::Separator(_) => {
                    // reset to default number input direction
                    dec_dir = EditDirection::Rtol
                }
                Mask::None => {
                    // no change, doesn't matter
                }
            }

            if matches!(mask, Mask::Separator(_)) || mask.section() != last_mask.section() {
                for j in start_sec..idx {
                    out[j].sec_id = sec_id;
                    out[j].sec_start = start_sec as upos_type;
                    out[j].sec_end = idx as upos_type;
                }
                sec_id += 1;
                start_sec = idx;
            }
            if matches!(mask, Mask::Separator(_)) || mask.sub_section() != last_mask.sub_section() {
                for j in start_sub..idx {
                    out[j].sub_start = start_sub as upos_type;
                    out[j].sub_end = idx as upos_type;
                }
                start_sub = idx;
            }

            let tok = MaskToken {
                sec_id: 0,
                sec_start: 0,
                sec_end: 0,
                sub_start: 0,
                sub_end: 0,
                peek_left: last_mask,
                right: mask.clone(),
                edit: mask.edit_value().into(),
            };
            out.push(tok);

            idx += 1;
            last_mask = mask;
        }
        for j in start_sec..out.len() {
            out[j].sec_id = sec_id;
            out[j].sec_start = start_sec as upos_type;
            out[j].sec_end = mask_str.graphemes(true).count() as upos_type;
        }
        for j in start_sub..out.len() {
            out[j].sub_start = start_sub as upos_type;
            out[j].sub_end = mask_str.graphemes(true).count() as upos_type;
        }

        Ok(out)
    }

    /// Display mask.
    #[inline]
    pub fn mask(&self) -> String {
        use std::fmt::Write;

        let mut buf = String::new();
        for t in self.mask.iter() {
            _ = write!(buf, "{}", t.right);
        }
        buf
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn set_invalid(&mut self, invalid: bool) {
        self.invalid = invalid;
    }

    /// Renders the widget in invalid style.
    #[inline]
    pub fn get_invalid(&self) -> bool {
        self.invalid
    }

    /// The next edit operation will overwrite the current content
    /// instead of adding text. Any move operations will cancel
    /// this overwrite.
    #[inline]
    pub fn set_overwrite(&mut self, overwrite: bool) {
        self.overwrite.set(overwrite);
    }

    /// Will the next edit operation overwrite the content?
    #[inline]
    pub fn overwrite(&self) -> bool {
        self.overwrite.get()
    }
}

impl MaskedInputState {
    /// Clipboard used.
    /// Default is to use the global_clipboard().
    #[inline]
    pub fn set_clipboard(&mut self, clip: Option<impl Clipboard + 'static>) {
        self.value.set_clipboard(clip.map(|v| {
            let r: Box<dyn Clipboard> = Box::new(v);
            r
        }));
    }

    /// Clipboard used.
    /// Default is to use the global_clipboard().
    #[inline]
    pub fn clipboard(&self) -> Option<&dyn Clipboard> {
        self.value.clipboard()
    }

    /// Copy to internal buffer
    #[inline]
    pub fn copy_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        _ = clip.set_string(self.selected_text().as_ref());

        true
    }

    /// Cut to internal buffer
    #[inline]
    pub fn cut_to_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        match clip.set_string(self.selected_text().as_ref()) {
            Ok(_) => self.delete_range(self.selection()),
            Err(_) => true,
        }
    }

    /// Paste from internal buffer.
    #[inline]
    pub fn paste_from_clip(&mut self) -> bool {
        let Some(clip) = self.value.clipboard() else {
            return false;
        };

        if let Ok(text) = clip.get_string() {
            for c in text.chars() {
                self.insert_char(c);
            }
            true
        } else {
            false
        }
    }
}

impl MaskedInputState {
    /// Set undo buffer.
    #[inline]
    pub fn set_undo_buffer(&mut self, undo: Option<impl UndoBuffer + 'static>) {
        self.value.set_undo_buffer(undo.map(|v| {
            let r: Box<dyn UndoBuffer> = Box::new(v);
            r
        }));
    }

    /// Undo
    #[inline]
    pub fn undo_buffer(&self) -> Option<&dyn UndoBuffer> {
        self.value.undo_buffer()
    }

    /// Undo
    #[inline]
    pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn UndoBuffer> {
        self.value.undo_buffer_mut()
    }

    /// Get all recent replay recordings.
    #[inline]
    pub fn recent_replay_log(&mut self) -> Vec<UndoEntry> {
        self.value.recent_replay_log()
    }

    /// Apply the replay recording.
    #[inline]
    pub fn replay_log(&mut self, replay: &[UndoEntry]) {
        self.value.replay_log(replay)
    }

    /// Begin a sequence of changes that should be undone in one go.
    #[inline]
    pub fn begin_undo_seq(&mut self) {
        self.value.begin_undo_seq();
    }

    /// End a sequence of changes that should be undone in one go.
    #[inline]
    pub fn end_undo_seq(&mut self) {
        self.value.end_undo_seq();
    }

    /// Undo operation
    #[inline]
    pub fn undo(&mut self) -> bool {
        self.value.undo()
    }

    /// Redo operation
    #[inline]
    pub fn redo(&mut self) -> bool {
        self.value.redo()
    }
}

impl MaskedInputState {
    /// Clear all styles.
    #[inline]
    pub fn clear_styles(&mut self) {
        self.value.set_styles(Vec::default());
    }

    /// Set and replace all styles.
    #[inline]
    pub fn set_styles(&mut self, styles: Vec<(Range<usize>, usize)>) {
        self.value.set_styles(styles);
    }

    /// Add a style for a byte-range. The style-nr refers to
    /// one of the styles set with the widget.
    #[inline]
    pub fn add_style(&mut self, range: Range<usize>, style: usize) {
        self.value.add_style(range, style);
    }

    /// Add a style for a `Range<upos_type>` to denote the cells.
    /// The style-nr refers to one of the styles set with the widget.
    #[inline]
    pub fn add_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self
            .value
            .bytes_at_range(TextRange::from((range.start, 0)..(range.end, 0)))?;
        self.value.add_style(r, style);
        Ok(())
    }

    /// Remove the exact byte-range and style.
    #[inline]
    pub fn remove_style(&mut self, range: Range<usize>, style: usize) {
        self.value.remove_style(range, style);
    }

    /// Remove the exact `Range<upos_type>` and style.
    #[inline]
    pub fn remove_range_style(
        &mut self,
        range: Range<upos_type>,
        style: usize,
    ) -> Result<(), TextError> {
        let r = self
            .value
            .bytes_at_range(TextRange::from((range.start, 0)..(range.end, 0)))?;
        self.value.remove_style(r, style);
        Ok(())
    }

    /// Find all styles that touch the given range.
    pub fn styles_in(&self, range: Range<usize>, buf: &mut Vec<(Range<usize>, usize)>) {
        self.value.styles_in(range, buf)
    }

    /// All styles active at the given position.
    #[inline]
    pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(Range<usize>, usize)>) {
        self.value.styles_at(byte_pos, buf)
    }

    /// Check if the given style applies at the position and
    /// return the complete range for the style.
    #[inline]
    pub fn styles_at_match(&self, byte_pos: usize, style: usize) -> Option<Range<usize>> {
        self.value.styles_at_match(byte_pos, style)
    }

    /// List of all styles.
    #[inline]
    pub fn styles(&self) -> Option<impl Iterator<Item = (Range<usize>, usize)> + '_> {
        self.value.styles()
    }
}

impl MaskedInputState {
    /// Text-offset.
    #[inline]
    pub fn offset(&self) -> upos_type {
        self.offset
    }

    /// Set the text-offset.
    #[inline]
    pub fn set_offset(&mut self, offset: upos_type) {
        self.scroll_to_cursor.set(false);
        self.offset = offset;
    }

    /// Cursor position.
    #[inline]
    pub fn cursor(&self) -> upos_type {
        self.value.cursor().x
    }

    /// Set the cursor position.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_cursor(&mut self, cursor: upos_type, extend_selection: bool) -> bool {
        self.scroll_cursor_to_visible();
        self.value
            .set_cursor(TextPosition::new(cursor, 0), extend_selection)
    }

    // find default cursor position for a number range
    fn number_cursor(&self, range: Range<upos_type>) -> upos_type {
        for (i, t) in self.mask[range.start as usize..range.end as usize]
            .iter()
            .enumerate()
            .rev()
        {
            match t.right {
                Mask::Digit(EditDirection::Rtol)
                | Mask::Digit0(EditDirection::Rtol)
                | Mask::Numeric(EditDirection::Rtol) => {
                    return range.start + i as upos_type + 1;
                }
                _ => {}
            }
        }
        range.start
    }

    /// Get the default cursor for the section at the given cursor position,
    /// if it is an editable section.
    pub fn section_cursor(&self, cursor: upos_type) -> Option<upos_type> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mask = &self.mask[cursor as usize];

        if mask.right.is_number() {
            Some(self.number_cursor(mask.sec_start..mask.sec_end))
        } else if mask.right.is_separator() {
            None
        } else if mask.right.is_none() {
            None
        } else {
            Some(mask.sec_start)
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn next_section_cursor(&self, cursor: upos_type) -> Option<upos_type> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut mask = &self.mask[cursor as usize];
        let mut next;
        loop {
            if mask.right.is_none() {
                return None;
            }

            next = mask.sec_end;
            mask = &self.mask[next as usize];

            if mask.right.is_number() {
                return Some(self.number_cursor(mask.sec_start..mask.sec_end));
            } else if mask.right.is_separator() {
                continue;
            } else if mask.right.is_none() {
                return None;
            } else {
                return Some(mask.sec_start);
            }
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn prev_section_cursor(&self, cursor: upos_type) -> Option<upos_type> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut prev = self.mask[cursor as usize].sec_start;
        let mut mask = &self.mask[prev as usize];

        loop {
            if mask.peek_left.is_none() {
                return None;
            }

            prev = self.mask[mask.sec_start as usize - 1].sec_start;
            mask = &self.mask[prev as usize];

            if mask.right.is_number() {
                return Some(self.number_cursor(mask.sec_start..mask.sec_end));
            } else if mask.right.is_separator() {
                continue;
            } else {
                return Some(mask.sec_start);
            }
        }
    }

    /// Is the position at a word boundary?
    pub fn is_section_boundary(&self, pos: upos_type) -> bool {
        if pos == 0 {
            return false;
        }
        if pos as usize >= self.mask.len() {
            return false;
        }
        let prev = &self.mask[pos as usize - 1];
        let mask = &self.mask[pos as usize];
        prev.sec_id != mask.sec_id
    }

    /// Get the index of the section at the given cursor position.
    pub fn section_id(&self, cursor: upos_type) -> u16 {
        let mask = &self.mask[cursor as usize];
        if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
            return self.mask[cursor.saturating_sub(1) as usize].sec_id;
        } else {
            mask.sec_id
        }
    }

    /// Get the range for the section at the given cursor position,
    /// if it is an editable section.
    pub fn section_range(&self, cursor: upos_type) -> Option<Range<upos_type>> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mask = &self.mask[cursor as usize];
        if mask.right.is_number() {
            Some(mask.sec_start..mask.sec_end)
        } else if mask.right.is_separator() {
            None
        } else if mask.right.is_none() {
            None
        } else {
            Some(mask.sec_start..mask.sec_end)
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn next_section_range(&self, cursor: upos_type) -> Option<Range<upos_type>> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut mask = &self.mask[cursor as usize];
        let mut next;
        loop {
            if mask.right.is_none() {
                return None;
            }

            next = mask.sec_end;
            mask = &self.mask[next as usize];

            if mask.right.is_number() {
                return Some(mask.sec_start..mask.sec_end);
            } else if mask.right.is_separator() {
                continue;
            } else if mask.right.is_none() {
                return None;
            } else {
                return Some(mask.sec_start..mask.sec_end);
            }
        }
    }

    /// Get the default cursor position for the next editable section.
    pub fn prev_section_range(&self, cursor: upos_type) -> Option<Range<upos_type>> {
        if cursor as usize >= self.mask.len() {
            return None;
        }

        let mut prev = self.mask[cursor as usize].sec_start;
        let mut mask = &self.mask[prev as usize];
        loop {
            if mask.peek_left.is_none() {
                return None;
            }

            prev = self.mask[mask.sec_start as usize - 1].sec_start;
            mask = &self.mask[prev as usize];

            if mask.right.is_number() {
                return Some(mask.sec_start..mask.sec_end);
            } else if mask.right.is_separator() {
                continue;
            } else {
                return Some(mask.sec_start..mask.sec_end);
            }
        }
    }

    /// Place cursor at the decimal separator, if any.
    /// 0 otherwise.
    /// Scrolls the cursor to a visible position.  
    #[inline]
    pub fn set_default_cursor(&mut self) {
        self.scroll_cursor_to_visible();
        if let Some(pos) = self.section_cursor(0) {
            self.value.set_cursor(TextPosition::new(pos, 0), false);
        } else if let Some(pos) = self.next_section_cursor(0) {
            self.value.set_cursor(TextPosition::new(pos, 0), false);
        } else {
            self.value.set_cursor(TextPosition::new(0, 0), false);
        }
    }

    /// Selection anchor.
    #[inline]
    pub fn anchor(&self) -> upos_type {
        self.value.anchor().x
    }

    /// Selection.
    #[inline]
    pub fn has_selection(&self) -> bool {
        self.value.has_selection()
    }

    /// Selection.
    #[inline]
    pub fn selection(&self) -> Range<upos_type> {
        let mut v = self.value.selection();
        if v.start == TextPosition::new(0, 1) {
            v.start = TextPosition::new(self.line_width(), 0);
        }
        if v.end == TextPosition::new(0, 1) {
            v.end = TextPosition::new(self.line_width(), 0);
        }
        v.start.x..v.end.x
    }

    /// Selection.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn set_selection(&mut self, anchor: upos_type, cursor: upos_type) -> bool {
        self.scroll_cursor_to_visible();
        self.value
            .set_selection(TextPosition::new(anchor, 0), TextPosition::new(cursor, 0))
    }

    /// Selection.
    /// Scrolls the cursor to a visible position.
    #[inline]
    pub fn select_all(&mut self) -> bool {
        self.scroll_cursor_to_visible();
        if let Some(section) = self.section_range(self.cursor()) {
            if self.selection() == section {
                self.value.select_all()
            } else {
                self.value.set_selection(
                    TextPosition::new(section.start, 0),
                    TextPosition::new(section.end, 0),
                )
            }
        } else {
            self.value.select_all()
        }
    }

    /// Selection.
    #[inline]
    pub fn selected_text(&self) -> &str {
        match self
            .value
            .str_slice(self.value.selection())
            .expect("valid_range")
        {
            Cow::Borrowed(v) => v,
            Cow::Owned(_) => {
                unreachable!()
            }
        }
    }
}

impl MaskedInputState {
    /// Empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.text().as_str() == self.default_value()
    }

    /// Value with all punctuation and default values according to the mask type.
    #[inline]
    pub fn text(&self) -> &str {
        self.value.text().as_str()
    }

    /// Parse value.
    #[inline]
    pub fn value<T: FromStr>(&self) -> Result<T, <T as FromStr>::Err> {
        self.value.text().as_str().parse::<T>()
    }

    /// Parse the value of one section.
    ///
    /// __Panic__
    ///
    /// Panics on out of bounds.
    #[inline]
    pub fn section_value<T: FromStr>(&self, section: u16) -> Result<T, <T as FromStr>::Err> {
        self.section_text(section).trim().parse::<T>()
    }

    /// Set the value of one section.
    ///
    /// __Panic__
    ///
    /// Panics on out of bounds.
    #[inline]
    pub fn set_section_value<T: ToString>(&mut self, section: u16, value: T) {
        let mut len = None;
        let mut align_left = true;
        for mask in &self.mask {
            if mask.sec_id == section {
                len = Some((mask.sec_end - mask.sec_start) as usize);
                if mask.right.is_rtol() {
                    align_left = false;
                } else {
                    align_left = true;
                }
                break;
            }
        }
        if let Some(len) = len {
            let txt = if align_left {
                format!("{:1$}", value.to_string(), len)
            } else {
                format!("{:>1$}", value.to_string(), len)
            };
            self.set_section_text(section, txt);
        } else {
            panic!("invalid section {}", section);
        }
    }

    /// Get one section.
    ///
    /// __Panic__
    ///
    /// Panics on out of bounds.
    #[inline]
    pub fn section_text(&self, section: u16) -> &str {
        for v in &self.mask {
            if v.sec_id == section {
                match self.str_slice(v.sec_start..v.sec_end) {
                    Cow::Borrowed(s) => return s,
                    Cow::Owned(_) => {
                        unreachable!("should not be owned")
                    }
                };
            }
        }
        panic!("invalid section {}", section);
    }

    /// Set the text for a given section.
    /// The text-string is cut to size and filled with blanks if necessary.
    pub fn set_section_text<S: Into<String>>(&mut self, section: u16, txt: S) {
        let mut txt = txt.into();
        for v in &self.mask {
            if v.sec_id == section {
                let len = (v.sec_end - v.sec_start) as usize;
                while txt.graphemes(true).count() > len {
                    txt.pop();
                }
                while txt.graphemes(true).count() < len {
                    txt.push(' ');
                }
                assert_eq!(txt.graphemes(true).count(), len);

                self.value.begin_undo_seq();
                self.value
                    .remove_str_range(TextRange::from(
                        TextPosition::new(v.sec_start, 0)..TextPosition::new(v.sec_end, 0),
                    ))
                    .expect("valid-range");
                self.value
                    .insert_str(TextPosition::new(v.sec_start, 0), txt.as_str())
                    .expect("valid-range");
                self.value.end_undo_seq();
                return;
            }
        }
        panic!("invalid section {}", section);
    }

    /// Text slice as `Cow<str>`. Uses a byte range.
    #[inline]
    pub fn str_slice_byte(&self, range: Range<usize>) -> Cow<'_, str> {
        self.value.str_slice_byte(range).expect("valid_range")
    }

    /// Text slice as `Cow<str>`. Uses a byte range.
    #[inline]
    pub fn try_str_slice_byte(&self, range: Range<usize>) -> Result<Cow<'_, str>, TextError> {
        self.value.str_slice_byte(range)
    }

    /// Text slice as `Cow<str>`
    #[inline]
    pub fn str_slice(&self, range: Range<upos_type>) -> Cow<'_, str> {
        self.value
            .str_slice(TextRange::new((range.start, 0), (range.end, 0)))
            .expect("valid_range")
    }

    /// Text slice as `Cow<str>`
    #[inline]
    pub fn try_str_slice(&self, range: Range<upos_type>) -> Result<Cow<'_, str>, TextError> {
        self.value
            .str_slice(TextRange::new((range.start, 0), (range.end, 0)))
    }

    /// Length as grapheme count.
    #[inline]
    pub fn len(&self) -> upos_type {
        self.value.line_width(0).expect("valid_row")
    }

    /// Length in bytes.
    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.value.len_bytes()
    }

    /// Length as grapheme count.
    #[inline]
    pub fn line_width(&self) -> upos_type {
        self.value.line_width(0).expect("valid_row")
    }

    /// Get the grapheme at the given position.
    #[inline]
    pub fn grapheme_at(&self, pos: upos_type) -> Result<Option<Grapheme<'_>>, TextError> {
        self.value.grapheme_at(TextPosition::new(pos, 0))
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn text_graphemes(&self, pos: upos_type) -> <TextString as TextStore>::GraphemeIter<'_> {
        self.try_text_graphemes(pos).expect("valid_pos")
    }

    /// Get a cursor over all the text with the current position set at pos.
    #[inline]
    pub fn try_text_graphemes(
        &self,
        pos: upos_type,
    ) -> Result<<TextString as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.text_graphemes(TextPosition::new(pos, 0))
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> <TextString as TextStore>::GraphemeIter<'_> {
        self.try_graphemes(range, pos).expect("valid_args")
    }

    /// Get a cursor over the text-range the current position set at pos.
    #[inline]
    pub fn try_graphemes(
        &self,
        range: Range<upos_type>,
        pos: upos_type,
    ) -> Result<<TextString as TextStore>::GraphemeIter<'_>, TextError> {
        self.value.graphemes(
            TextRange::new((range.start, 0), (range.end, 0)),
            TextPosition::new(pos, 0),
        )
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn byte_at(&self, pos: upos_type) -> Range<usize> {
        self.try_byte_at(pos).expect("valid_pos")
    }

    /// Grapheme position to byte position.
    /// This is the (start,end) position of the single grapheme after pos.
    #[inline]
    pub fn try_byte_at(&self, pos: upos_type) -> Result<Range<usize>, TextError> {
        self.value.byte_at(TextPosition::new(pos, 0))
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn bytes_at_range(&self, range: Range<upos_type>) -> Range<usize> {
        self.try_bytes_at_range(range).expect("valid_range")
    }

    /// Grapheme range to byte range.
    #[inline]
    pub fn try_bytes_at_range(&self, range: Range<upos_type>) -> Result<Range<usize>, TextError> {
        self.value
            .bytes_at_range(TextRange::new((range.start, 0), (range.end, 0)))
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn byte_pos(&self, byte: usize) -> upos_type {
        self.try_byte_pos(byte).expect("valid_pos")
    }

    /// Byte position to grapheme position.
    /// Returns the position that contains the given byte index.
    #[inline]
    pub fn try_byte_pos(&self, byte: usize) -> Result<upos_type, TextError> {
        Ok(self.value.byte_pos(byte)?.x)
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn byte_range(&self, bytes: Range<usize>) -> Range<upos_type> {
        self.try_byte_range(bytes).expect("valid_range")
    }

    /// Byte range to grapheme range.
    #[inline]
    pub fn try_byte_range(&self, bytes: Range<usize>) -> Result<Range<upos_type>, TextError> {
        let r = self.value.byte_range(bytes)?;
        Ok(r.start.x..r.end.x)
    }
}

impl MaskedInputState {
    /// Create a default value according to the mask.
    #[inline]
    fn default_value(&self) -> String {
        MaskToken::empty_section(&self.mask)
    }

    /// Reset to empty.
    #[inline]
    pub fn clear(&mut self) -> bool {
        if self.is_empty() {
            false
        } else {
            self.offset = 0;
            self.value
                .set_text(TextString::new_string(self.default_value()));
            self.set_default_cursor();
            true
        }
    }

    /// Set text.
    ///
    /// Returns an error if the text contains line-breaks.
    #[inline]
    pub fn set_value<S: ToString>(&mut self, s: S) {
        self.set_text(s.to_string());
    }

    /// Set the value.
    ///
    /// No checks if the value conforms to the mask.
    /// If the value is too short it will be filled with space.
    /// if the value is too long it will be truncated.
    #[inline]
    pub fn set_text<S: Into<String>>(&mut self, s: S) {
        self.offset = 0;
        let mut text = s.into();
        while text.graphemes(true).count() > self.mask.len().saturating_sub(1) {
            text.pop();
        }
        while text.graphemes(true).count() < self.mask.len().saturating_sub(1) {
            text.push(' ');
        }
        let len = text.graphemes(true).count();

        assert_eq!(len, self.mask.len().saturating_sub(1));

        self.value.set_text(TextString::new_string(text));
        self.set_default_cursor();
    }

    /// Insert a char at the current position.
    #[inline]
    pub fn insert_char(&mut self, c: char) -> bool {
        self.begin_undo_seq();
        if self.has_selection() {
            let sel = self.selection();
            mask_op::remove_range(self, sel.clone()).expect("valid_selection");
            self.set_cursor(sel.start, false);
        }
        let c0 = mask_op::advance_cursor(self, c);
        let c1 = mask_op::insert_char(self, c);
        self.end_undo_seq();

        self.scroll_cursor_to_visible();
        c0 || c1
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn delete_range(&mut self, range: Range<upos_type>) -> bool {
        self.try_delete_range(range).expect("valid_range")
    }

    /// Remove the selected range. The text will be replaced with the default value
    /// as defined by the mask.
    #[inline]
    pub fn try_delete_range(&mut self, range: Range<upos_type>) -> Result<bool, TextError> {
        self.value.begin_undo_seq();
        let r = mask_op::remove_range(self, range.clone())?;
        if let Some(pos) = self.section_cursor(range.start) {
            self.set_cursor(pos, false);
        }
        self.value.end_undo_seq();

        self.scroll_cursor_to_visible();
        Ok(r)
    }
}

impl MaskedInputState {
    /// Delete the char after the cursor.
    #[inline]
    pub fn delete_next_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else if self.cursor() == self.len() {
            false
        } else {
            mask_op::remove_next(self);
            self.scroll_cursor_to_visible();
            true
        }
    }

    /// Delete the char before the cursor.
    #[inline]
    pub fn delete_prev_char(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else if self.cursor() == 0 {
            false
        } else {
            mask_op::remove_prev(self);
            self.scroll_cursor_to_visible();
            true
        }
    }

    /// Delete the previous section.
    #[inline]
    pub fn delete_prev_section(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            if let Some(range) = self.prev_section_range(self.cursor()) {
                self.delete_range(range)
            } else {
                false
            }
        }
    }

    /// Delete the next section.
    #[inline]
    pub fn delete_next_section(&mut self) -> bool {
        if self.has_selection() {
            self.delete_range(self.selection())
        } else {
            if let Some(range) = self.next_section_range(self.cursor()) {
                self.delete_range(range)
            } else {
                false
            }
        }
    }

    /// Move to the next char.
    #[inline]
    pub fn move_right(&mut self, extend_selection: bool) -> bool {
        let c = min(self.cursor() + 1, self.len());
        self.set_cursor(c, extend_selection)
    }

    /// Move to the previous char.
    #[inline]
    pub fn move_left(&mut self, extend_selection: bool) -> bool {
        let c = self.cursor().saturating_sub(1);
        self.set_cursor(c, extend_selection)
    }

    /// Start of line
    #[inline]
    pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
        if let Some(c) = self.section_cursor(self.cursor()) {
            if c != self.cursor() {
                self.set_cursor(c, extend_selection)
            } else {
                self.set_cursor(0, extend_selection)
            }
        } else {
            self.set_cursor(0, extend_selection)
        }
    }

    /// End of line
    #[inline]
    pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
        self.set_cursor(self.len(), extend_selection)
    }

    /// Move to start of previous section.
    #[inline]
    pub fn move_to_prev_section(&mut self, extend_selection: bool) -> bool {
        if let Some(curr) = self.section_range(self.cursor()) {
            if self.cursor() != curr.start {
                return self.set_cursor(curr.start, extend_selection);
            }
        }
        if let Some(range) = self.prev_section_range(self.cursor()) {
            self.set_cursor(range.start, extend_selection)
        } else {
            false
        }
    }

    /// Move to end of previous section.
    #[inline]
    pub fn move_to_next_section(&mut self, extend_selection: bool) -> bool {
        if let Some(curr) = self.section_range(self.cursor()) {
            if self.cursor() != curr.end {
                return self.set_cursor(curr.end, extend_selection);
            }
        }
        if let Some(range) = self.next_section_range(self.cursor()) {
            self.set_cursor(range.end, extend_selection)
        } else {
            false
        }
    }

    /// Select next section.
    #[inline]
    pub fn select_current_section(&mut self) -> bool {
        let selection = self.selection();

        if let Some(next) = self.section_range(selection.start.saturating_sub(1)) {
            if !next.is_empty() {
                self.set_selection(next.start, next.end)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Select next section.
    #[inline]
    pub fn select_next_section(&mut self) -> bool {
        let selection = self.selection();

        if let Some(next) = self.next_section_range(selection.start) {
            if !next.is_empty() {
                self.set_selection(next.start, next.end)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Select previous section.
    #[inline]
    pub fn select_prev_section(&mut self) -> bool {
        let selection = self.selection();

        if let Some(next) = self.prev_section_range(selection.start.saturating_sub(1)) {
            if !next.is_empty() {
                self.set_selection(next.start, next.end)
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl HasScreenCursor for MaskedInputState {
    /// The current text cursor as an absolute screen position.
    #[inline]
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        if self.is_focused() {
            if self.has_selection() {
                None
            } else {
                let cx = self.cursor();
                let ox = self.offset();

                if cx < ox {
                    None
                } else if cx > ox + (self.inner.width + self.dark_offset.0) as upos_type {
                    None
                } else {
                    self.col_to_screen(cx)
                        .map(|sc| (self.inner.x + sc, self.inner.y))
                }
            }
        } else {
            None
        }
    }
}

impl RelocatableState for MaskedInputState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        // clip offset for some corrections.
        self.dark_offset = relocate_dark_offset(self.inner, shift, clip);
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
    }
}

impl MaskedInputState {
    fn glyphs2(&self) -> impl Iterator<Item = Glyph2<'_>> {
        let left_margin = self.offset();
        let right_margin = self.offset() + self.rendered.width as upos_type;
        let compact = self.compact && !self.is_focused();

        let grapheme_iter = self
            .value
            .graphemes(TextRange::new((0, 0), (0, 1)), TextPosition::new(0, 0))
            .expect("valid-rows");
        let mask_iter = self.mask.iter();

        let iter = MaskedGraphemes {
            iter_str: grapheme_iter,
            iter_mask: mask_iter,
            compact,
            sym_neg: self.neg_sym().to_string(),
            sym_dec: self.dec_sep().to_string(),
            sym_grp: self.grp_sep().to_string(),
            sym_pos: self.pos_sym().to_string(),
            byte_pos: 0,
        };

        let mut it = GlyphIter2::new(TextPosition::new(0, 0), 0, iter, Default::default());
        it.set_tabs(0 /* no tabs */);
        it.set_show_ctrl(self.value.glyph_ctrl());
        it.set_lf_breaks(false);
        it.set_text_wrap(TextWrap2::Shift);
        it.set_left_margin(left_margin);
        it.set_right_margin(right_margin);
        it.set_word_margin(right_margin);
        it.prepare().expect("valid-rows");

        Box::new(it)
    }

    /// Converts from a widget relative screen coordinate to a grapheme index.
    /// x is the relative screen position.
    pub fn screen_to_col(&self, scx: i16) -> upos_type {
        let ox = self.offset();

        let scx = scx + self.dark_offset.0 as i16;

        if scx < 0 {
            ox.saturating_sub((scx as ipos_type).unsigned_abs())
        } else if scx as u16 >= (self.inner.width + self.dark_offset.0) {
            min(ox + scx as upos_type, self.len())
        } else {
            let scx = scx as u16;

            let line = self.glyphs2();

            let mut col = ox;
            for g in line {
                if g.contains_screen_x(scx) {
                    break;
                }
                col = g.pos().x + 1;
            }
            col
        }
    }

    /// Converts a grapheme based position to a screen position
    /// relative to the widget area.
    pub fn col_to_screen(&self, pos: upos_type) -> Option<u16> {
        let ox = self.offset();

        if pos < ox {
            return None;
        }

        let line = self.glyphs2();
        let mut screen_x = 0;
        for g in line {
            if g.pos().x == pos {
                break;
            }
            screen_x = g.screen_pos().0 + g.screen_width();
        }

        if screen_x >= self.dark_offset.0 {
            Some(screen_x - self.dark_offset.0)
        } else {
            None
        }
    }

    /// Set the cursor position from a screen position relative to the origin
    /// of the widget. This value can be negative, which selects a currently
    /// not visible position and scrolls to it.
    #[inline]
    pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
        let scx = cursor;

        let cx = self.screen_to_col(scx);

        self.set_cursor(cx, extend_selection)
    }

    /// Set the cursor position from screen coordinates,
    /// rounds the position to the next section bounds.
    ///
    /// The cursor positions are relative to the inner rect.
    /// They may be negative too, this allows setting the cursor
    /// to a position that is currently scrolled away.
    pub fn set_screen_cursor_sections(
        &mut self,
        screen_cursor: i16,
        extend_selection: bool,
    ) -> bool {
        let anchor = self.anchor();
        let cursor = self.screen_to_col(screen_cursor);

        let Some(range) = self.section_range(cursor) else {
            return false;
        };

        let cursor = if cursor < anchor {
            range.start
        } else {
            range.end
        };

        // extend anchor
        if !self.is_section_boundary(anchor) {
            if let Some(range) = self.section_range(anchor) {
                if cursor < anchor {
                    self.set_cursor(range.end, false);
                } else {
                    self.set_cursor(range.start, false);
                }
            };
        }

        self.set_cursor(cursor, extend_selection)
    }

    /// Scrolling
    pub fn scroll_left(&mut self, delta: upos_type) -> bool {
        self.set_offset(self.offset.saturating_sub(delta));
        true
    }

    /// Scrolling
    pub fn scroll_right(&mut self, delta: upos_type) -> bool {
        self.set_offset(self.offset + delta);
        true
    }

    /// Change the offset in a way that the cursor is visible.
    pub fn scroll_cursor_to_visible(&mut self) {
        self.scroll_to_cursor.set(true);
    }
}

impl HandleEvent<crossterm::event::Event, Regular, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> TextOutcome {
        // small helper ...
        fn tc(r: bool) -> TextOutcome {
            if r {
                TextOutcome::TextChanged
            } else {
                TextOutcome::Unchanged
            }
        }
        fn overwrite(state: &mut MaskedInputState) {
            if state.overwrite.get() {
                state.overwrite.set(false);
                state.clear();
            }
        }
        fn clear_overwrite(state: &mut MaskedInputState) {
            state.overwrite.set(false);
        }

        // // focus behaviour
        // if self.lost_focus() {
        //     match self.on_focus_lost {
        //         TextFocusLost::None => {}
        //         TextFocusLost::Position0 => {
        //             self.set_default_cursor();
        //             self.scroll_cursor_to_visible();
        //             // repaint is triggered by focus-change
        //         }
        //     }
        // }
        // if self.gained_focus() {
        //     match self.on_focus_gained {
        //         TextFocusGained::None => {}
        //         TextFocusGained::Overwrite => {
        //             self.overwrite = true;
        //         }
        //         TextFocusGained::SelectAll => {
        //             self.select_all();
        //             // repaint is triggered by focus-change
        //         }
        //     }
        // }

        let mut r = if self.is_focused() {
            match event {
                ct_event!(key press c)
                | ct_event!(key press SHIFT-c)
                | ct_event!(key press CONTROL_ALT-c) => {
                    overwrite(self);
                    tc(self.insert_char(*c))
                }
                ct_event!(keycode press Backspace) => {
                    clear_overwrite(self);
                    tc(self.delete_prev_char())
                }
                ct_event!(keycode press Delete) => {
                    clear_overwrite(self);
                    tc(self.delete_next_char())
                }
                ct_event!(keycode press CONTROL-Backspace)
                | ct_event!(keycode press ALT-Backspace) => {
                    clear_overwrite(self);
                    tc(self.delete_prev_section())
                }
                ct_event!(keycode press CONTROL-Delete) => {
                    clear_overwrite(self);
                    tc(self.delete_next_section())
                }
                ct_event!(key press CONTROL-'x') => {
                    clear_overwrite(self);
                    tc(self.cut_to_clip())
                }
                ct_event!(key press CONTROL-'v') => {
                    clear_overwrite(self);
                    tc(self.paste_from_clip())
                }
                ct_event!(key press CONTROL-'d') => {
                    clear_overwrite(self);
                    tc(self.clear())
                }
                ct_event!(key press CONTROL-'z') => {
                    clear_overwrite(self);
                    tc(self.undo())
                }
                ct_event!(key press CONTROL_SHIFT-'Z') => {
                    clear_overwrite(self);
                    tc(self.redo())
                }

                ct_event!(key release _)
                | ct_event!(key release SHIFT-_)
                | ct_event!(key release CONTROL_ALT-_)
                | ct_event!(keycode release Backspace)
                | ct_event!(keycode release Delete)
                | ct_event!(keycode release CONTROL-Backspace)
                | ct_event!(keycode release ALT-Backspace)
                | ct_event!(keycode release CONTROL-Delete)
                | ct_event!(key release CONTROL-'x')
                | ct_event!(key release CONTROL-'v')
                | ct_event!(key release CONTROL-'d')
                | ct_event!(key release CONTROL-'z')
                | ct_event!(key release CONTROL_SHIFT-'Z') => TextOutcome::Unchanged,

                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };

        if r == TextOutcome::Continue {
            r = self.handle(event, ReadOnly);
        }
        r
    }
}

impl HandleEvent<crossterm::event::Event, ReadOnly, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: ReadOnly) -> TextOutcome {
        fn clear_overwrite(state: &mut MaskedInputState) {
            state.overwrite.set(false);
        }

        let mut r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => {
                    clear_overwrite(self);
                    self.move_left(false).into()
                }
                ct_event!(keycode press Right) => {
                    clear_overwrite(self);
                    self.move_right(false).into()
                }
                ct_event!(keycode press CONTROL-Left) => {
                    clear_overwrite(self);
                    self.move_to_prev_section(false).into()
                }
                ct_event!(keycode press CONTROL-Right) => {
                    clear_overwrite(self);
                    self.move_to_next_section(false).into()
                }
                ct_event!(keycode press Home) => {
                    clear_overwrite(self);
                    self.move_to_line_start(false).into()
                }
                ct_event!(keycode press End) => {
                    clear_overwrite(self);
                    self.move_to_line_end(false).into()
                }
                ct_event!(keycode press SHIFT-Left) => {
                    clear_overwrite(self);
                    self.move_left(true).into()
                }
                ct_event!(keycode press SHIFT-Right) => {
                    clear_overwrite(self);
                    self.move_right(true).into()
                }
                ct_event!(keycode press CONTROL_SHIFT-Left) => {
                    clear_overwrite(self);
                    self.move_to_prev_section(true).into()
                }
                ct_event!(keycode press CONTROL_SHIFT-Right) => {
                    clear_overwrite(self);
                    self.move_to_next_section(true).into()
                }
                ct_event!(keycode press SHIFT-Home) => {
                    clear_overwrite(self);
                    self.move_to_line_start(true).into()
                }
                ct_event!(keycode press SHIFT-End) => {
                    clear_overwrite(self);
                    self.move_to_line_end(true).into()
                }
                ct_event!(keycode press Tab) => {
                    // ignore tab from focus
                    if !self.focus.gained() {
                        clear_overwrite(self);
                        self.select_next_section().into()
                    } else {
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(keycode press SHIFT-BackTab) => {
                    // ignore tab from focus
                    if !self.focus.gained() {
                        clear_overwrite(self);
                        self.select_prev_section().into()
                    } else {
                        TextOutcome::Unchanged
                    }
                }
                ct_event!(key press CONTROL-'a') => {
                    clear_overwrite(self);
                    self.select_all().into()
                }
                ct_event!(key press CONTROL-'c') => {
                    clear_overwrite(self);
                    self.copy_to_clip().into()
                }

                ct_event!(keycode release Left)
                | ct_event!(keycode release Right)
                | ct_event!(keycode release CONTROL-Left)
                | ct_event!(keycode release CONTROL-Right)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release SHIFT-Left)
                | ct_event!(keycode release SHIFT-Right)
                | ct_event!(keycode release CONTROL_SHIFT-Left)
                | ct_event!(keycode release CONTROL_SHIFT-Right)
                | ct_event!(keycode release SHIFT-Home)
                | ct_event!(keycode release SHIFT-End)
                | ct_event!(key release CONTROL-'a')
                | ct_event!(key release CONTROL-'c') => TextOutcome::Unchanged,

                _ => TextOutcome::Continue,
            }
        } else {
            TextOutcome::Continue
        };

        if r == TextOutcome::Continue {
            r = self.handle(event, MouseOnly);
        }
        r
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TextOutcome> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> TextOutcome {
        fn clear_overwrite(state: &mut MaskedInputState) {
            state.overwrite.set(false);
        }

        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.inner, m) => {
                let c = (m.column as i16) - (self.inner.x as i16);
                clear_overwrite(self);
                self.set_screen_cursor(c, true).into()
            }
            ct_event!(mouse any for m) if self.mouse.drag2(self.inner, m, KeyModifiers::ALT) => {
                let cx = m.column as i16 - self.inner.x as i16;
                clear_overwrite(self);
                self.set_screen_cursor_sections(cx, true).into()
            }
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.inner, m) => {
                let tx = self.screen_to_col(m.column as i16 - self.inner.x as i16);
                clear_overwrite(self);
                if let Some(range) = self.section_range(tx) {
                    self.set_selection(range.start, range.end).into()
                } else {
                    TextOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for column,row) => {
                if self.gained_focus() {
                    // don't react to the first click that's for
                    // focus. this one shouldn't demolish the selection.
                    TextOutcome::Unchanged
                } else if self.inner.contains((*column, *row).into()) {
                    let c = (column - self.inner.x) as i16;
                    clear_overwrite(self);
                    self.set_screen_cursor(c, false).into()
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down CONTROL-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    clear_overwrite(self);
                    self.set_screen_cursor(cx, true).into()
                } else {
                    TextOutcome::Continue
                }
            }
            ct_event!(mouse down ALT-Left for column,row) => {
                if self.inner.contains((*column, *row).into()) {
                    let cx = (column - self.inner.x) as i16;
                    clear_overwrite(self);
                    self.set_screen_cursor_sections(cx, true).into()
                } else {
                    TextOutcome::Continue
                }
            }
            _ => TextOutcome::Continue,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut MaskedInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only navigation events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_readonly_events(
    state: &mut TextInputState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.focus.set(focus);
    state.handle(event, ReadOnly)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut MaskedInputState,
    event: &crossterm::event::Event,
) -> TextOutcome {
    state.handle(event, MouseOnly)
}
