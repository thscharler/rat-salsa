//! Text input with an input mask.
//!
//! * Can do the usual insert/delete/move operations.
//! * Text selection
//! * Scrolls with the cursor.
//! * Can set the cursor or use its own block cursor.
//! * Can show an indicator for invalid input.
//!
//! * Accepts an input mask:
//!   * `0`: can enter digit, display as 0  
//!   * `9`: can enter digit, display as space
//!   * `#`: digit, plus or minus sign, display as space
//!   * `+`: sign. display '+' for positive
//!   * `-`: sign. display ' ' for positive
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
//!   * `:` `;` `-` `/`: separator characters move the cursor when entered.
//!   * `\`: escapes the following character and uses it as a separator.
//!   * all other ascii characters a reserved.
//!   * other unicode characters can be used as separators without escaping.
//!
//! * Accepts a display overlay used instead of the default chars of the input mask.
//!
//! ```
//! use rat_salsa::widget::mask_input::MaskedInputState;
//!
//! let mut date_state = MaskedInputState::new();
//! date_state.set_mask("99/99/9999");
//! date_state.set_display_mask("mm/dd/yyyy");
//!
//! let mut creditcard_state = MaskedInputState::new();
//! creditcard_state.set_mask("dddd dddd dddd dddd");
//!
//! ```

use crate::grapheme;
use crate::number::NumberSymbols;
use crate::widget::basic::ClearStyle;
use crate::widget::mask_input::core::InputMaskCore;
use crate::{ControlUI, ValidFlag};
use crate::{DefaultKeys, FrameWidget, HandleCrossterm, Input, MouseOnly};
use crate::{FocusFlag, HasFocusFlag, HasValidFlag};
#[allow(unused_imports)]
use log::debug;
use ratatui::layout::{Margin, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::Frame;
use std::cmp::min;
use std::ops::Range;
use std::rc::Rc;

/// Text input widget with input mask.
#[derive(Debug)]
pub struct MaskedInput {
    pub terminal_cursor: bool,
    pub without_focus: bool,
    pub insets: Margin,
    pub style: Style,
    pub focus_style: Style,
    pub select_style: Style,
    pub cursor_style: Option<Style>,
    pub invalid_style: Option<Style>,
    pub invalid_char: char,
}

/// Combined style.
#[derive(Debug, Default)]
pub struct MaskedInputStyle {
    pub style: Style,
    pub focus: Style,
    pub select: Style,
    pub cursor: Option<Style>,
    pub invalid: Option<Style>,
}

impl Default for MaskedInput {
    fn default() -> Self {
        Self {
            terminal_cursor: true,
            without_focus: false,
            insets: Default::default(),
            style: Default::default(),
            focus_style: Default::default(),
            select_style: Default::default(),
            cursor_style: None,
            invalid_style: None,
            invalid_char: '⁉',
        }
    }
}

impl MaskedInput {
    /// Use extra insets for the text input.
    pub fn insets(mut self, insets: Margin) -> Self {
        self.insets = insets;
        self
    }

    /// Use our own cursor indicator or the terminal cursor.
    pub fn terminal_cursor(mut self, terminal: bool) -> Self {
        self.terminal_cursor = terminal;
        self
    }

    /// Do accept keyboard events event without being focused.
    /// Useful for a catch field, eg "find stuff"
    pub fn without_focus(mut self, without_focus: bool) -> Self {
        self.without_focus = without_focus;
        self
    }

    /// Set the combined style.
    pub fn style(mut self, style: MaskedInputStyle) -> Self {
        self.style = style.style;
        self.focus_style = style.focus;
        self.select_style = style.select;
        self.cursor_style = style.cursor;
        self.invalid_style = style.invalid;
        self
    }

    /// Base text style.
    pub fn base_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Style when focused.
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = style.into();
        self
    }

    /// Style for selection
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = style.into();
        self
    }

    /// Style for our own cursor.
    pub fn cursor_style(mut self, style: impl Into<Style>) -> Self {
        self.cursor_style = Some(style.into());
        self
    }

    /// Style for the invalid indicator.
    pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
        self.invalid_style = Some(style.into());
        self
    }

    /// Marker character for invalid field.
    pub fn invalid_char(mut self, invalid: char) -> Self {
        self.invalid_char = invalid;
        self
    }

    fn active_style(&self, focus: bool) -> Style {
        if focus {
            self.focus_style
        } else {
            self.style
        }
    }

    fn active_select_style(&self, focus: bool) -> Style {
        if self.without_focus || focus {
            self.select_style
        } else {
            self.style
        }
    }
}

impl FrameWidget for MaskedInput {
    type State = MaskedInputState;

    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State) {
        state.without_focus = self.without_focus;

        let mut l_area = area.inner(&self.insets);
        let l_invalid = if !state.valid.get() {
            l_area.width -= 1;
            Rect::new(l_area.x + l_area.width, l_area.y, 1, 1)
        } else {
            Rect::new(l_area.x + l_area.width, l_area.y, 0, 1)
        };

        state.area = l_area;
        state.value.set_width(state.area.width as usize);

        let focus = state.focus.get();
        let l_input = state.area;

        let (before, cursor1, select, cursor2, after) = state.visible_part();

        let mut spans = Vec::new();
        if !before.is_empty() {
            spans.push(Span::styled(before, self.active_style(focus)));
        }
        if !cursor1.is_empty() {
            if let Some(cursor_style) = self.cursor_style {
                spans.push(Span::styled(cursor1, cursor_style));
            } else {
                spans.push(Span::styled(cursor1, self.active_select_style(focus)));
            }
        }
        if !select.is_empty() {
            spans.push(Span::styled(select, self.active_select_style(focus)));
        }
        if !cursor2.is_empty() {
            if let Some(cursor_style) = self.cursor_style {
                spans.push(Span::styled(cursor2, cursor_style));
            } else {
                spans.push(Span::styled(cursor2, self.active_style(focus)));
            }
        }
        if !after.is_empty() {
            spans.push(Span::styled(after, self.active_style(focus)));
        }

        let line = Line::from(spans);
        let clear = ClearStyle::default().style(self.active_style(focus));

        frame.render_widget(clear, area);
        frame.render_widget(line, l_input);
        if !state.valid.get() {
            let style = if let Some(style) = self.invalid_style {
                style
            } else {
                self.active_style(focus)
            };

            let invalid = Span::from(self.invalid_char.to_string()).style(style);
            frame.render_widget(invalid, l_invalid);
        }
        if focus {
            frame.set_cursor(l_input.x + state.visible_cursor(), l_input.y);
        }
    }
}

/// State of the input-mask.
#[derive(Debug, Clone, Default)]
pub struct MaskedInputState {
    /// Focus
    pub focus: FocusFlag,
    /// Valid.
    pub valid: ValidFlag,
    /// Work without focus for key input.
    pub without_focus: bool,
    /// Area
    pub area: Rect,
    /// Mouse selection in progress.
    pub mouse_select: bool,
    /// Editing core.
    pub value: InputMaskCore,
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for MaskedInputState {
    #[allow(non_snake_case)]
    fn handle(&mut self, event: &crossterm::event::Event, _: DefaultKeys) -> ControlUI<A, E> {
        use crossterm::event::KeyCode::*;
        use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyModifiers};

        const NONE: KeyModifiers = KeyModifiers::NONE;
        const CTRL: KeyModifiers = KeyModifiers::CONTROL;
        const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
        let CTRL_SHIFT: KeyModifiers = KeyModifiers::SHIFT | KeyModifiers::CONTROL;

        let req = match event {
            Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => 'f: {
                if !self.focus.get() && !self.without_focus {
                    break 'f None;
                }

                match (*code, *modifiers) {
                    (Left, NONE) => Some(InputRequest::GoToPrevChar(false)),
                    (Right, NONE) => Some(InputRequest::GoToNextChar(false)),
                    (Left, CTRL) => Some(InputRequest::GoToPrevWord(false)),
                    (Right, CTRL) => Some(InputRequest::GoToNextWord(false)),
                    (Home, NONE) => Some(InputRequest::GoToStart(false)),
                    (End, NONE) => Some(InputRequest::GoToEnd(false)),

                    (Left, SHIFT) => Some(InputRequest::GoToPrevChar(true)),
                    (Right, SHIFT) => Some(InputRequest::GoToNextChar(true)),
                    (Left, m) if m == CTRL_SHIFT => Some(InputRequest::GoToPrevWord(true)),
                    (Right, m) if m == CTRL_SHIFT => Some(InputRequest::GoToNextWord(true)),
                    (Home, SHIFT) => Some(InputRequest::GoToStart(true)),
                    (End, SHIFT) => Some(InputRequest::GoToEnd(true)),

                    (Char('a'), CTRL) => Some(InputRequest::SelectAll),

                    (Backspace, NONE) => Some(InputRequest::DeletePrevChar),
                    (Delete, NONE) => Some(InputRequest::DeleteNextChar),

                    (Backspace, m) if m == CTRL_SHIFT => Some(InputRequest::DeletePrevWord),
                    (Delete, m) if m == CTRL_SHIFT => Some(InputRequest::DeleteNextWord),

                    (Char('d'), CTRL) => Some(InputRequest::DeleteLine),
                    (Backspace, CTRL) => Some(InputRequest::DeleteTillStart),
                    (Delete, CTRL) => Some(InputRequest::DeleteTillEnd),

                    (Char(c), NONE) => Some(InputRequest::InsertChar(c)),
                    (Char(c), SHIFT) => Some(InputRequest::InsertChar(c)),
                    (_, _) => None,
                }
            }
            _ => return self.handle(event, MouseOnly),
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for MaskedInputState {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> ControlUI<A, E> {
        use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let req = match event {
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.mouse_select = true;
                    let c = column - self.area.x;
                    Some(InputRequest::SetCursor(c as usize, false))
                } else {
                    None
                }
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if self.mouse_select {
                    self.mouse_select = false;
                }
                None
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                if self.mouse_select {
                    let c = if *column >= self.area.x {
                        column - self.area.x
                    } else {
                        0
                    };
                    Some(InputRequest::SetCursor(c as usize, true))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            ControlUI::Continue
        }
    }
}

/// Mapping from events to abstract editing requests.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum InputRequest {
    SetCursor(usize, bool),
    Select(usize, usize),
    InsertChar(char),
    GoToPrevChar(bool),
    GoToNextChar(bool),
    GoToPrevWord(bool),
    GoToNextWord(bool),
    GoToStart(bool),
    GoToEnd(bool),
    SelectAll,
    DeletePrevChar,
    DeleteNextChar,
    DeletePrevWord,
    DeleteNextWord,
    DeleteLine,
    DeleteTillStart,
    DeleteTillEnd,
}

impl MaskedInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_symbols(sym: &Rc<NumberSymbols>) -> Self {
        Self {
            value: InputMaskCore::new_with_symbols(sym),
            ..Self::default()
        }
    }

    /// Reset to empty.
    pub fn reset(&mut self) {
        self.value.reset();
    }

    /// Offset shown.
    pub fn offset(&self) -> usize {
        self.value.offset()
    }

    /// Offset shown. This is corrected if the cursor wouldn't be visible.
    pub fn set_offset(&mut self, offset: usize) {
        self.value.set_offset(offset);
    }

    /// Display width.
    pub fn width(&self) -> usize {
        self.value.width()
    }

    /// Display width
    pub fn set_width(&mut self, width: usize) {
        self.value.set_width(width);
    }

    /// Set the cursor position, reset selection.
    pub fn set_cursor(&mut self, cursor: usize) {
        self.value.set_cursor(cursor, false);
    }

    /// Place cursor at decimal separator, if any. 0 otherwise.
    pub fn set_default_cursor(&mut self) {
        self.value.set_default_cursor();
    }

    /// Cursor position
    pub fn cursor(&self) -> usize {
        self.value.cursor()
    }

    /// Set the display mask. This text is used for parts that have
    /// no valid input yet. Part means consecutive characters of the
    /// input mask with the same mask type.
    ///
    /// There is a default representation for each mask type if this
    /// is not set.
    ///
    /// Panic
    /// Panics if the length differs from the  mask.
    pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
        self.value.set_display_mask(s);
    }

    /// Display mask.
    pub fn display_mask(&self) -> String {
        self.value.display_mask()
    }

    /// Set the input mask. This overwrites the display mask and the value
    /// with a default representation of the mask.
    ///
    /// The result value contains all punctuation and
    /// the value given as 'display' below. See [compact_value()](MaskedInputState::compact_value).
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
    /// * `:` `;` `-` `/`: separator characters move the cursor when entered.
    /// * `\`: escapes the following character and uses it as a separator.
    /// * all other ascii characters a reserved.
    ///
    /// Inspired by <https://support.microsoft.com/en-gb/office/control-data-entry-formats-with-input-masks-e125997a-7791-49e5-8672-4a47832de8da>
    pub fn set_mask<S: AsRef<str>>(&mut self, s: S) {
        self.value.set_mask(s);
    }

    /// Display mask.
    pub fn mask(&self) -> String {
        self.value.mask()
    }

    /// Debug mask.
    pub fn debug_mask(&self) -> String {
        self.value.debug_mask()
    }

    /// Set symbols for number display.
    ///
    /// These are only used for rendering and to map user input.
    /// The value itself uses ".", "," and "-".
    pub fn set_num_symbols(&mut self, sym: &Rc<NumberSymbols>) {
        self.value.set_num_symbols(sym);
    }

    /// Set the value.
    ///
    /// Panic
    /// Panics if the grapheme length of the value is not the same as the mask.
    pub fn set_value<S: Into<String>>(&mut self, s: S) {
        self.value.set_value(s);
    }

    /// Value with all punctuation and default values according to the mask type.
    pub fn value(&self) -> &str {
        self.value.value()
    }

    /// Value with optional spaces removed.
    pub fn compact_value(&self) -> String {
        self.value.compact_value()
    }

    /// Value.
    pub fn as_str(&self) -> &str {
        self.value.value()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Length in grapheme count.
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Selection
    pub fn has_selection(&self) -> bool {
        self.value.is_anchored()
    }

    /// Selection
    pub fn set_selection(&mut self, anchor: usize, cursor: usize) {
        self.value.set_cursor(anchor, false);
        self.value.set_cursor(cursor, true);
    }

    /// Selection
    pub fn select_all(&mut self) {
        // the other way round it fails if width is 0.
        self.value.set_cursor(self.value.len(), false);
        self.value.set_cursor(0, true);
    }

    /// Selection
    pub fn selection(&self) -> Range<usize> {
        self.value.selection()
    }

    /// Selection
    pub fn selection_str(&self) -> &str {
        grapheme::split3(self.value.value(), self.value.selection()).1
    }

    /// Extracts the visible part.
    pub fn visible_range(&self) -> Range<usize> {
        let len = min(self.value.offset() + self.value.width(), self.value.len());
        self.value.offset()..len
    }

    /// Extracts the visible selection.
    pub fn visible_selection(&self) -> Range<usize> {
        let width = self.value.width();
        let offset = self.value.offset();
        let Range { mut start, mut end } = self.value.selection();

        if start < offset {
            start = offset;
        } else if start > offset + width {
            start = offset + width;
        }
        if end < offset {
            end = offset;
        } else if end > offset + width {
            end = offset + width;
        }

        start..end
    }

    /// Extracts the visible part.
    pub fn visible_part(&mut self) -> (&str, &str, &str, &str, &str) {
        self.value.render_value();

        grapheme::split5(
            self.value.rendered(),
            self.cursor(),
            self.visible_range(),
            self.visible_selection(),
        )
    }

    /// Visible cursor position.
    pub fn visible_cursor(&mut self) -> u16 {
        (self.value.cursor() - self.value.offset()) as u16
    }
}

impl HasFocusFlag for MaskedInputState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasValidFlag for MaskedInputState {
    fn valid(&self) -> &ValidFlag {
        &self.valid
    }
}

impl<A, E> Input<ControlUI<A, E>> for MaskedInputState {
    type Request = InputRequest;

    fn perform(&mut self, action: Self::Request) -> ControlUI<A, E> {
        use InputRequest::*;

        match action {
            SetCursor(pos, anchor) => {
                let pos = pos + self.value.offset();
                if self.value.cursor() == pos {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(pos, anchor);
                    ControlUI::Change
                }
            }
            Select(anchor, cursor) => {
                self.value.set_cursor(anchor, false);
                self.value.set_cursor(cursor, true);
                ControlUI::Change
            }
            InsertChar(c) => {
                if self.value.is_anchored() {
                    self.value.remove_selection(self.value.selection());
                }
                self.value.advance_cursor(c);
                self.value.insert_char(c);
                ControlUI::Change
            }
            DeletePrevChar => {
                if self.value.is_select_all() {
                    self.value.reset();
                    ControlUI::Change
                } else if self.value.is_anchored() {
                    self.value.remove_selection(self.value.selection());
                    ControlUI::Change
                } else if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value.remove_prev();
                    ControlUI::Change
                }
            }
            DeleteNextChar => {
                if self.value.is_select_all() {
                    self.value.reset();
                    ControlUI::Change
                } else if self.value.is_anchored() {
                    self.value.remove_selection(self.value.selection());
                    ControlUI::Change
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value.remove_next();
                    ControlUI::Change
                }
            }
            GoToPrevChar(anchor) => {
                if !anchor && self.value.is_anchored() {
                    let c = self.value.selection().start;
                    self.value.set_cursor(c, false);
                    ControlUI::Change
                } else if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(self.value.cursor() - 1, anchor);
                    ControlUI::Change
                }
            }
            GoToNextChar(anchor) => {
                if !anchor && self.value.is_anchored() {
                    let c = self.value.selection().end;
                    self.value.set_cursor(c, false);
                    ControlUI::Change
                } else if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(self.value.cursor() + 1, anchor);
                    ControlUI::Change
                }
            }
            GoToPrevWord(anchor) => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    let cursor = self.value.prev_word_boundary();
                    self.value.set_cursor(cursor, anchor);
                    ControlUI::Change
                }
            }
            GoToNextWord(anchor) => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    let cursor = self.value.next_word_boundary();
                    self.value.set_cursor(cursor, anchor);
                    ControlUI::Change
                }
            }
            DeleteLine => {
                self.value.reset();
                ControlUI::Change
            }
            DeletePrevWord => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    let prev = self.value.prev_word_boundary();
                    self.value.remove_selection(prev..self.value.cursor());
                    ControlUI::Change
                }
            }
            DeleteNextWord => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    let next = self.value.next_word_boundary();
                    self.value.remove_selection(self.value.cursor()..next);
                    ControlUI::Change
                }
            }
            GoToStart(anchor) => {
                if self.value.cursor() == 0 {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(0, anchor);
                    ControlUI::Change
                }
            }
            GoToEnd(anchor) => {
                if self.value.cursor() == self.value.len() {
                    ControlUI::NoChange
                } else {
                    self.value.set_cursor(self.value.len(), anchor);
                    ControlUI::Change
                }
            }
            DeleteTillEnd => {
                self.value
                    .remove_selection(self.value.cursor()..self.value.len());
                ControlUI::Change
            }
            DeleteTillStart => {
                self.value.remove_selection(0..self.value.cursor());
                ControlUI::Change
            }
            SelectAll => {
                self.value.set_cursor(0, false);
                self.value.set_cursor(self.value.len(), true);
                ControlUI::Change
            }
        }
    }
}

pub mod core {
    use crate::grapheme;
    use crate::number::NumberSymbols;
    #[allow(unused_imports)]
    use log::debug;
    use std::fmt::{Debug, Display, Formatter};
    use std::iter::{once, repeat};
    use std::mem;
    use std::ops::Range;
    use std::rc::Rc;
    use unicode_segmentation::UnicodeSegmentation;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum EditDirection {
        Ltor,
        Rtol,
    }

    #[allow(variant_size_differences)]
    #[derive(Clone, PartialEq, Eq, Default)]
    pub enum Mask {
        Digit0(EditDirection),
        Digit(EditDirection),
        Numeric(EditDirection),
        DecimalSep,
        GroupingSep,
        Plus,
        Minus,
        Hex0,
        Hex,
        Oct0,
        Oct,
        Dec0,
        Dec,
        Letter,
        LetterOrDigit,
        LetterDigitSpace,
        AnyChar,
        Separator(Box<str>),
        #[default]
        None,
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct MaskToken {
        pub nr_id: usize,
        pub nr_start: usize,
        pub nr_end: usize,

        pub sec_id: usize,
        pub sec_start: usize,
        pub sec_end: usize,

        pub peek_left: Mask,
        pub right: Mask,
        pub edit: Box<str>,
        pub display: Box<str>,
    }

    impl Debug for EditDirection {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    EditDirection::Ltor => ">",
                    EditDirection::Rtol => "<",
                }
            )
        }
    }

    impl Display for Mask {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => "9",
                Mask::Numeric(_) => "#",
                Mask::DecimalSep => ".",
                Mask::GroupingSep => ",",
                Mask::Plus => "+",
                Mask::Minus => "-",
                Mask::Hex0 => "H",
                Mask::Hex => "h",
                Mask::Oct0 => "O",
                Mask::Oct => "o",
                Mask::Dec0 => "D",
                Mask::Dec => "d",
                Mask::Letter => "l",
                Mask::LetterOrDigit => "a",
                Mask::LetterDigitSpace => "c",
                Mask::AnyChar => "_",
                Mask::Separator(s) => {
                    if matches!(
                        s.as_ref(),
                        "0" | "9"
                            | "#"
                            | "."
                            | ","
                            | "H"
                            | "h"
                            | "O"
                            | "o"
                            | "D"
                            | "d"
                            | "l"
                            | "a"
                            | "c"
                            | "_"
                    ) {
                        write!(f, "\\")?;
                    }
                    s
                }
                Mask::None => "",
            };
            write!(f, "{}", s)
        }
    }

    impl Debug for Mask {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Mask::Digit0(d) => {
                    write!(f, "{:?}0", d)
                }
                Mask::Digit(d) => {
                    write!(f, "{:?}9", d)
                }
                Mask::Numeric(d) => {
                    write!(f, "{:?}#", d)
                }
                Mask::DecimalSep => write!(f, "."),
                Mask::GroupingSep => write!(f, ","),
                Mask::Plus => write!(f, "+"),
                Mask::Minus => write!(f, "-"),
                Mask::Hex0 => write!(f, "H"),
                Mask::Hex => write!(f, "h"),
                Mask::Oct0 => write!(f, "O"),
                Mask::Oct => write!(f, "o"),
                Mask::Dec0 => write!(f, "D"),
                Mask::Dec => write!(f, "d"),
                Mask::Letter => write!(f, "l"),
                Mask::LetterOrDigit => write!(f, "a"),
                Mask::LetterDigitSpace => write!(f, "c"),
                Mask::AnyChar => write!(f, "_"),
                Mask::Separator(s) => {
                    if matches!(
                        s.as_ref(),
                        "0" | "9"
                            | "#"
                            | "."
                            | ","
                            | "H"
                            | "h"
                            | "O"
                            | "o"
                            | "D"
                            | "d"
                            | "l"
                            | "a"
                            | "c"
                            | "_"
                    ) {
                        write!(f, "\\")?;
                    }
                    write!(f, "{}", s)
                }
                Mask::None => write!(f, ""),
            }
        }
    }

    impl EditDirection {
        fn is_ltor(&self) -> bool {
            *self == EditDirection::Ltor
        }

        fn is_rtol(&self) -> bool {
            *self == EditDirection::Rtol
        }
    }

    impl Mask {
        //
        fn is_none(&self) -> bool {
            *self == Mask::None
        }

        //
        fn is_number(&self) -> bool {
            match self {
                Mask::Digit0(_) => true,
                Mask::Digit(_) => true,
                Mask::Numeric(_) => true,
                Mask::DecimalSep => true,
                Mask::GroupingSep => true,
                Mask::Plus => true,
                Mask::Minus => true,
                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        // left to right editing
        fn is_ltor(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_ltor(),
                Mask::Digit(d) => d.is_ltor(),
                Mask::Numeric(d) => d.is_ltor(),
                Mask::GroupingSep => false,
                Mask::Plus => false,
                Mask::Minus => false,
                Mask::DecimalSep => true,
                Mask::Hex0 => true,
                Mask::Hex => true,
                Mask::Oct0 => true,
                Mask::Oct => true,
                Mask::Dec0 => true,
                Mask::Dec => true,
                Mask::Letter => true,
                Mask::LetterOrDigit => true,
                Mask::LetterDigitSpace => true,
                Mask::AnyChar => true,
                Mask::Separator(_) => true,
                Mask::None => false,
            }
        }

        // right to left editing
        fn is_rtol(&self) -> bool {
            match self {
                Mask::Digit0(d) => d.is_rtol(),
                Mask::Digit(d) => d.is_rtol(),
                Mask::Numeric(d) => d.is_rtol(),
                Mask::GroupingSep => true,
                Mask::Plus => true,
                Mask::Minus => true,
                Mask::DecimalSep => false,
                Mask::Hex0 => false,
                Mask::Hex => false,
                Mask::Oct0 => false,
                Mask::Oct => false,
                Mask::Dec0 => false,
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => false,
            }
        }

        // which masks fall in the same section
        fn section(&self) -> u8 {
            match self {
                Mask::Digit0(_) => 0,
                Mask::Digit(_) => 0,
                Mask::Numeric(_) => 0,
                Mask::GroupingSep => 0,
                Mask::Plus => 1,
                Mask::Minus => 1,
                Mask::DecimalSep => 2,
                Mask::Hex0 => 3,
                Mask::Hex => 3,
                Mask::Oct0 => 4,
                Mask::Oct => 4,
                Mask::Dec0 => 5,
                Mask::Dec => 5,
                Mask::Letter => 6,
                Mask::LetterOrDigit => 6,
                Mask::LetterDigitSpace => 6,
                Mask::AnyChar => 6,
                Mask::Separator(_) => 7,
                Mask::None => 8,
            }
        }

        // which masks count for the same number
        fn number(&self) -> u8 {
            match self {
                Mask::Digit0(_) => 0,
                Mask::Digit(_) => 0,
                Mask::Numeric(_) => 0,
                Mask::GroupingSep => 0,
                Mask::Plus => 0,
                Mask::Minus => 0,
                Mask::DecimalSep => 0,

                Mask::Hex0 => 1,
                Mask::Hex => 1,
                Mask::Oct0 => 1,
                Mask::Oct => 1,
                Mask::Dec0 => 1,
                Mask::Dec => 1,
                Mask::Letter => 1,
                Mask::LetterOrDigit => 1,
                Mask::LetterDigitSpace => 1,
                Mask::AnyChar => 1,

                Mask::Separator(_) => 2,
                Mask::None => 3,
            }
        }

        // mask should overwrite instead of insert
        fn can_overwrite(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(d) | Mask::Digit(d) | Mask::Numeric(d) => *d == EditDirection::Ltor,
                Mask::DecimalSep => "." == c,
                Mask::GroupingSep => false,
                Mask::Plus => "-" == c || "+" == c,
                Mask::Minus => "-" == c || " " == c,
                Mask::Hex0 => c == "0",
                Mask::Hex => false,
                Mask::Oct0 => c == "0",
                Mask::Oct => false,
                Mask::Dec0 => c == "0",
                Mask::Dec => false,
                Mask::Letter => false,
                Mask::LetterOrDigit => false,
                Mask::LetterDigitSpace => false,
                Mask::AnyChar => false,
                Mask::Separator(sep) => sep.as_ref() == c,
                Mask::None => false,
            }
        }

        // char can be dropped
        fn can_drop(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) => c == "0",
                Mask::Digit(_) => c == " ",
                Mask::Numeric(_) => c == " ",
                Mask::DecimalSep => false,
                Mask::Plus => false,
                Mask::Minus => false,
                Mask::GroupingSep => true,
                Mask::Hex0 => c == "0",
                Mask::Hex => c == " ",
                Mask::Oct0 => c == "0",
                Mask::Oct => c == " ",
                Mask::Dec0 => c == "0",
                Mask::Dec => c == " ",
                Mask::Letter => c == " ",
                Mask::LetterOrDigit => c == " ",
                Mask::LetterDigitSpace => c == " ",
                Mask::AnyChar => c == " ",
                Mask::Separator(_sep) => false,
                Mask::None => false,
            }
        }

        // can be skipped when generating the condensed form
        fn can_skip(&self, c: &str) -> bool {
            match self {
                Mask::Digit0(_) => false,
                Mask::Digit(_) => c == " ",
                Mask::Numeric(_) => c == " ",
                Mask::DecimalSep => false,
                Mask::Plus => false,
                Mask::Minus => false,
                Mask::GroupingSep => true,
                Mask::Hex0 => false,
                Mask::Hex => c == " ",
                Mask::Oct0 => false,
                Mask::Oct => c == " ",
                Mask::Dec0 => false,
                Mask::Dec => c == " ",
                Mask::Letter => c == " ",
                Mask::LetterOrDigit => c == " ",
                Mask::LetterDigitSpace => c == " ",
                Mask::AnyChar => false,
                Mask::Separator(_) => false,
                Mask::None => true,
            }
        }

        fn replace_char(&self, test: char, dec: char) -> char {
            // todo: does this make any sense?
            match self {
                Mask::DecimalSep => {
                    if test == dec {
                        '.'
                    } else {
                        test
                    }
                }
                _ => test,
            }
        }

        fn is_valid_char(&self, test_grapheme: &str, dec: char) -> bool {
            // todo: does this make any sense?
            let Some(test) = test_grapheme.chars().next() else {
                return false;
            };
            match self {
                Mask::Digit0(_) => test.is_ascii_digit(),
                Mask::Digit(_) => test.is_ascii_digit() || test == ' ',
                Mask::Numeric(_) => test.is_ascii_digit() || test == ' ' || test == '-',
                Mask::DecimalSep => test == dec,
                Mask::GroupingSep => false,
                Mask::Plus => test == '+' || test == '-',
                Mask::Minus => test == ' ' || test == '-',
                Mask::Hex0 => test.is_ascii_hexdigit(),
                Mask::Hex => test.is_ascii_hexdigit() || test == ' ',
                Mask::Oct0 => test.is_digit(8),
                Mask::Oct => test.is_digit(8) || test == ' ',
                Mask::Dec0 => test.is_ascii_digit(),
                Mask::Dec => test.is_ascii_digit() || test == ' ',
                Mask::Letter => test.is_alphabetic(),
                Mask::LetterOrDigit => test.is_alphanumeric(),
                Mask::LetterDigitSpace => test.is_alphanumeric() || test == ' ',
                Mask::AnyChar => true,
                Mask::Separator(g) => g.as_ref() == test_grapheme,
                Mask::None => false,
            }
        }

        fn edit_value(&self) -> &str {
            match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => " ",
                Mask::Numeric(_) => " ",
                Mask::DecimalSep => ".",
                Mask::GroupingSep => " ", // don't show. remap_number fills it in if necessary.
                Mask::Plus => "+",
                Mask::Minus => " ",
                Mask::Hex0 => "0",
                Mask::Hex => " ",
                Mask::Oct0 => "0",
                Mask::Oct => " ",
                Mask::Dec0 => "0",
                Mask::Dec => " ",
                Mask::Letter => " ",
                Mask::LetterOrDigit => " ",
                Mask::LetterDigitSpace => " ",
                Mask::AnyChar => " ",
                Mask::Separator(g) => g.as_ref(),
                Mask::None => "",
            }
        }

        fn disp_value(&self) -> &str {
            match self {
                Mask::Digit0(_) => "0",
                Mask::Digit(_) => " ",
                Mask::Numeric(_) => " ",
                Mask::DecimalSep => ".",  // only used by get_display_mask()
                Mask::GroupingSep => ",", // only used by get_display_mask()
                Mask::Plus => "+",
                Mask::Minus => " ",
                Mask::Hex0 => "0",
                Mask::Hex => " ",
                Mask::Oct0 => "0",
                Mask::Oct => " ",
                Mask::Dec0 => "0",
                Mask::Dec => " ",
                Mask::Letter => " ",
                Mask::LetterOrDigit => " ",
                Mask::LetterDigitSpace => " ",
                Mask::AnyChar => " ",
                Mask::Separator(g) => g.as_ref(),
                Mask::None => "",
            }
        }

        fn can_drop_first(&self, s: &str) -> bool {
            if s.is_empty() {
                false
            } else {
                let (c, _a) = grapheme::split_at(s, 1);
                self.can_drop(c)
            }
        }

        fn can_drop_last(&self, s: &str) -> bool {
            if s.is_empty() {
                false
            } else {
                let end = s.graphemes(true).count();
                let (_, c) = grapheme::split_at(s, end - 1);
                self.can_drop(c)
            }
        }

        fn can_overwrite_first(&self, s: &str) -> bool {
            if s.is_empty() {
                false
            } else {
                let (c, _) = grapheme::split_at(s, 1);
                self.can_overwrite(c)
            }
        }
    }

    impl Debug for MaskToken {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Mask #{}:{}:{}-{} {:?} | {:?}",
                self.nr_id, self.sec_id, self.sec_start, self.sec_end, self.peek_left, self.right
            )
        }
    }

    impl MaskToken {
        /// Number range as Range.
        fn nr_range(&self) -> Range<usize> {
            self.nr_start..self.nr_end
        }

        /// Range as Range.
        fn range(&self) -> Range<usize> {
            self.sec_start..self.sec_end
        }

        /// Create a string with the default edit mask.
        fn empty_section(mask: &[MaskToken]) -> String {
            let mut buf = String::new();
            for m in mask {
                buf.push_str(&m.edit);
            }
            buf
        }

        fn remap_number(submask: &[MaskToken], v: &str) -> String {
            // remove all non numbers and leading 0.
            let mut clean = String::new();
            let mut seen_non_0 = false;
            for c in v.graphemes(true) {
                let Some(test) = c.chars().next() else {
                    unreachable!();
                };
                if test.is_ascii_digit() || test == '-' || test == '.' {
                    if seen_non_0 || c != "0" {
                        clean.push_str(c);
                    }
                    if !seen_non_0 {
                        seen_non_0 = test.is_ascii_digit() && test != '0';
                    }
                }
            }

            let mut out_vec = Vec::from_iter(repeat(" ").take(submask.len()));
            let mut out_idx = out_vec.len() as isize - 1;

            let mut it = submask.iter().rev();
            let mut jt = clean.graphemes(true).rev();

            let mut m = None;
            let mut v = None;
            loop {
                if m.is_none() {
                    m = it.next();
                }
                if v.is_none() {
                    v = jt.next();
                }

                let Some(mm) = m else {
                    break;
                };

                if matches!(mm.right, Mask::Digit0(_)) {
                    if let Some(vv) = v {
                        if mm.right.is_valid_char(vv, '.') {
                            out_vec[out_idx as usize] = vv;
                            out_idx -= 1;
                            v = None;
                        } else {
                            out_vec[out_idx as usize] = &mm.edit;
                            out_idx -= 1;
                            // keep v
                        }
                    } else {
                        out_vec[out_idx as usize] = &mm.edit;
                        out_idx -= 1;
                        // keep v
                    }
                    m = None;
                } else if matches!(mm.right, Mask::Digit(_) | Mask::Numeric(_)) {
                    if let Some(vv) = v {
                        out_vec[out_idx as usize] = vv;
                        out_idx -= 1;
                        v = None;
                    }
                    m = None;
                } else if matches!(mm.right, Mask::DecimalSep) {
                    if let Some(vv) = v {
                        out_vec[out_idx as usize] = vv;
                        out_idx -= 1;
                        v = None;
                    }
                    m = None;
                } else if matches!(mm.right, Mask::GroupingSep) {
                    if let Some(vv) = v {
                        if vv == "-" {
                            out_vec[out_idx as usize] = vv;
                            out_idx -= 1;
                            v = None;
                        } else {
                            out_vec[out_idx as usize] = ",";
                            out_idx -= 1;
                            // keep v
                        }
                    }
                    m = None;
                } else if matches!(mm.right, Mask::Minus) {
                    if let Some(vv) = v {
                        if vv == "-" {
                            out_vec[out_idx as usize] = vv;
                            out_idx -= 1;
                            v = None;
                        } else {
                            out_vec[out_idx as usize] = " ";
                            out_idx -= 1;
                            // keep v
                        }
                    }
                    m = None;
                } else if matches!(mm.right, Mask::Plus) {
                    if let Some(vv) = v {
                        if vv == "-" {
                            out_vec[out_idx as usize] = vv;
                            out_idx -= 1;
                            v = None;
                        } else {
                            out_vec[out_idx as usize] = "+";
                            out_idx -= 1;
                            // keep v
                        }
                    }
                    m = None;
                } else {
                    unreachable!("{:?}", mm.right);
                }
            }

            let out = out_vec.iter().fold(String::new(), |mut s, v| {
                s.push_str(v);
                s
            });

            out
        }
    }

    /// Text editing core.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct InputMaskCore {
        mask: Vec<MaskToken>,
        value: String,
        rendered: String,
        len: usize,

        offset: usize,
        width: usize,

        cursor: usize,
        anchor: usize,

        sym: Option<Rc<NumberSymbols>>,
    }

    impl Default for InputMaskCore {
        fn default() -> Self {
            Self {
                mask: Vec::new(),
                value: Default::default(),
                rendered: Default::default(),
                len: 0,
                offset: 0,
                width: 0,
                cursor: 0,
                anchor: 0,
                sym: None,
            }
        }
    }

    impl InputMaskCore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn new_with_symbols(sym: &Rc<NumberSymbols>) -> Self {
            Self {
                mask: Default::default(),
                value: Default::default(),
                rendered: Default::default(),
                len: 0,
                offset: 0,
                width: 0,
                cursor: 0,
                anchor: 0,
                sym: Some(Rc::clone(sym)),
            }
        }

        /// Tokens used for the mask.
        pub fn tokens(&self) -> &[MaskToken] {
            &self.mask
        }

        /// Reset value but not the mask and width.
        /// Resets offset and cursor position too.
        pub fn reset(&mut self) {
            self.offset = 0;
            self.set_value(MaskToken::empty_section(&self.mask));
            self.set_default_cursor();
        }

        /// Offset
        pub fn offset(&self) -> usize {
            self.offset
        }

        /// Change the offset.
        ///
        /// Ensures the cursor is visible and modifies any given offset.
        /// Ensures the offset is not beyond the length.
        pub fn set_offset(&mut self, offset: usize) {
            if offset > self.len {
                self.offset = self.len;
            } else if offset > self.cursor {
                self.offset = self.cursor;
            } else if offset + self.width < self.cursor {
                self.offset = self.cursor - self.width;
            } else {
                self.offset = offset;
            }
        }

        /// Display width
        pub fn width(&self) -> usize {
            self.width
        }

        /// Display width
        pub fn set_width(&mut self, width: usize) {
            self.width = width;

            if self.offset + width < self.cursor {
                self.offset = self.cursor - self.width;
            }
        }

        /// Cursor position as grapheme-idx.
        pub fn cursor(&self) -> usize {
            self.cursor
        }

        pub fn anchor(&self) -> usize {
            self.anchor
        }

        pub fn is_anchored(&self) -> bool {
            self.cursor != self.anchor
        }

        pub fn is_select_all(&self) -> bool {
            let selection = self.selection();
            selection.start == 0 && selection.end == self.mask.len() - 1
        }

        pub fn selection(&self) -> Range<usize> {
            if self.cursor < self.anchor {
                self.cursor..self.anchor
            } else {
                self.anchor..self.cursor
            }
        }

        pub fn set_cursor(&mut self, cursor: usize, anchor: bool) {
            if cursor > self.len {
                self.cursor = self.len;
            } else {
                self.cursor = cursor;
            }

            if !anchor {
                self.anchor = self.cursor;
            }

            self.fix_offset();
        }

        fn fix_offset(&mut self) {
            if self.offset > self.cursor {
                self.offset = self.cursor;
            } else if self.offset + self.width < self.cursor {
                self.offset = self.cursor - self.width;
            }
        }

        /// Place cursor at decimal separator, if any. 0 otherwise.
        pub fn set_default_cursor(&mut self) {
            'f: {
                for (i, t) in self.mask.iter().enumerate() {
                    if t.right == Mask::DecimalSep {
                        self.cursor = i;
                        self.anchor = i;
                        break 'f;
                    }
                }
                self.cursor = 0;
                self.anchor = 0;
                self.fix_offset();
            }
        }

        /// Set the decimal separator.
        ///
        /// Only used for rendering and to map user input.
        /// The value itself uses "."
        pub fn set_num_symbols(&mut self, sym: &Rc<NumberSymbols>) {
            self.sym = Some(Rc::clone(sym));
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
                sym.decimal_grp
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
                '-'
            }
        }

        /// Changes the mask.
        /// Resets the value to a default.
        pub fn set_mask<S: AsRef<str>>(&mut self, s: S) {
            self.mask = parse_mask(s.as_ref());
            self.set_value(MaskToken::empty_section(&self.mask));
        }

        /// Return the mask.
        pub fn mask(&self) -> String {
            use std::fmt::Write;

            let mut buf = String::new();
            for t in self.mask.iter() {
                _ = write!(buf, "{}", t.right);
            }
            buf
        }

        /// Return the mask.
        pub fn debug_mask(&self) -> String {
            use std::fmt::Write;

            let mut buf = String::new();
            for t in self.mask.iter() {
                _ = write!(buf, "{:?}", t.right);
            }
            buf
        }

        /// Set the mask that is shown.
        pub fn set_display_mask<S: Into<String>>(&mut self, s: S) {
            let display_mask = s.into();

            for (t, m) in self.mask.iter_mut().zip(display_mask.graphemes(true)) {
                if t.right != Mask::DecimalSep && t.right != Mask::GroupingSep {
                    t.display = Box::from(m);
                }
            }
        }

        /// Display mask
        pub fn display_mask(&self) -> String {
            let mut buf = String::new();
            for t in &self.mask {
                buf.push_str(&t.display);
            }
            buf
        }

        /// Sets the value.
        /// No checks if the value conforms to the mask.
        /// If the value is too short it will be filled with space.
        /// if the value is too long it will be truncated.
        pub fn set_value<S: Into<String>>(&mut self, s: S) {
            let mut value = s.into();
            let len = value.graphemes(true).count();

            if len > self.mask.len() - 1 {
                for _ in len..self.mask.len() - 1 {
                    value.pop();
                }
            } else if len < self.mask.len() - 1 {
                for _ in len..self.mask.len() - 1 {
                    value.push(' ');
                }
            }
            assert_eq!(len, self.mask.len() - 1);

            self.value = value;
            self.len = len;

            if self.offset > self.len {
                self.offset = self.len;
            }
            if self.cursor > self.len {
                self.cursor = self.len;
            }
        }

        /// Value
        pub fn value(&self) -> &str {
            self.value.as_str()
        }

        /// Value without whitespace.
        pub fn compact_value(&self) -> String {
            let mut buf = String::new();
            for (c, m) in self.value.graphemes(true).zip(self.mask.iter()) {
                if !m.right.can_skip(c) {
                    buf.push_str(c);
                }
            }
            buf
        }

        /// No value different from the default.
        pub fn is_empty(&self) -> bool {
            for (m, c) in self.mask.iter().zip(self.value.graphemes(true)) {
                if c != m.edit.as_ref() {
                    return false;
                }
            }
            true
        }

        /// Length
        pub fn len(&self) -> usize {
            self.len
        }

        /// Rendered string for display.
        pub fn rendered(&self) -> &str {
            self.rendered.as_str()
        }

        /// Create the rendered value.
        #[allow(unused_variables)]
        pub fn render_value(&mut self) {
            let mut rendered = mem::take(&mut self.rendered);
            rendered.clear();

            let mut idx = 0;
            loop {
                let mask = &self.mask[idx];

                if mask.right == Mask::None {
                    break;
                }

                let (b, sec, a) = grapheme::split3(&self.value, mask.sec_start..mask.sec_end);
                let sec_mask = &self.mask[mask.sec_start..mask.sec_end];
                let empty = MaskToken::empty_section(sec_mask);

                if sec == empty {
                    for t in sec_mask {
                        if t.right == Mask::GroupingSep {
                            rendered.push(' ');
                        } else if t.right == Mask::DecimalSep {
                            rendered.push(self.dec_sep());
                        } else {
                            rendered.push_str(t.display.as_ref());
                        };
                    }
                } else {
                    for (t, s) in sec_mask.iter().zip(sec.graphemes(true)) {
                        if t.right == Mask::GroupingSep {
                            if s == "," {
                                rendered.push(self.grp_sep());
                            } else if s == "-" {
                                rendered.push(self.neg_sym());
                            } else if s == "+" {
                                rendered.push(self.pos_sym());
                            } else {
                                rendered.push(' ');
                            }
                        } else if t.right == Mask::DecimalSep {
                            if s == "." {
                                rendered.push(self.dec_sep());
                            } else {
                                rendered.push(' ');
                            }
                        } else {
                            if s == "-" {
                                rendered.push(self.neg_sym());
                            } else if s == "+" {
                                rendered.push(self.pos_sym());
                            } else {
                                rendered.push_str(s);
                            }
                        };
                    }
                }

                idx = mask.sec_end;
            }

            self.rendered = rendered;
        }

        /// Next boundary.
        pub fn next_word_boundary(&self) -> usize {
            if self.cursor == self.len {
                self.len
            } else {
                self.value
                    .graphemes(true)
                    .enumerate()
                    .skip(self.cursor)
                    .skip_while(|(_, c)| grapheme::is_alphanumeric(c))
                    .find(|(_, c)| grapheme::is_alphanumeric(c))
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| self.len)
            }
        }

        /// Previous boundary.
        pub fn prev_word_boundary(&self) -> usize {
            if self.cursor == 0 {
                0
            } else {
                self.value
                    .graphemes(true)
                    .rev()
                    .skip(self.len - self.cursor)
                    .skip_while(|c| grapheme::is_alphanumeric(c))
                    .skip_while(|c| grapheme::is_alphanumeric(c))
                    .count()
            }
        }

        /// Advance the cursor to the next section, if new char matches
        /// certain conditions.
        pub fn advance_cursor(&mut self, c: char) {
            let buf = String::from(c);
            let cc = buf.as_str();

            let mut new_cursor = self.cursor;

            debug!("// ADVANCE CURSOR {:?}  ", cc);
            debug!("#[rustfmt::skip]");
            debug!("let mut b = {};", test_state(self));
            debug!("b.advance_cursor({:?});", c);

            loop {
                let mask = &self.mask[new_cursor];

                // at the gap between rtol and ltor field
                if mask.peek_left.is_rtol()
                    && (mask.right.is_ltor() || mask.right.is_none())
                    && ({
                        let left = &self.mask[new_cursor - 1];
                        let mask0 = &self.mask[left.sec_start];
                        let (_b, c0, _c1, _a) =
                            grapheme::split_mask(&self.value, new_cursor, left.range());
                        // can insert at mask gap?
                        mask0.right.can_drop_first(c0)
                            && left.right.is_valid_char(cc, self.dec_sep())
                    })
                {
                    break;
                } else if mask.right.is_rtol()
                    && ({
                        let (_b, a) = grapheme::split_at(&self.value, new_cursor);
                        // stop at real digit, that is the first non-droppable grapheme.
                        !mask.right.can_drop_first(a)
                            && mask.right.is_valid_char(cc, self.dec_sep())
                    })
                {
                    break;
                } else if mask.right == Mask::DecimalSep
                    && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    new_cursor += 1; // todo: can be removed? should be a replace position
                    break;
                } else if mask.right == Mask::GroupingSep {
                    // never stop here
                } else if matches!(mask.right, Mask::Separator(_))
                    && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    new_cursor += 1; // todo: can be removed? should be a replace position
                    break;
                } else if matches!(
                    mask.right,
                    Mask::Digit0(EditDirection::Ltor)
                        | Mask::Digit(EditDirection::Ltor)
                        | Mask::Numeric(EditDirection::Ltor)
                ) && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    break;
                } else if matches!(
                    mask.right,
                    Mask::Hex0 | Mask::Hex | Mask::Dec0 | Mask::Dec | Mask::Oct0 | Mask::Oct
                ) && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    break;
                } else if matches!(
                    mask.right,
                    Mask::Letter | Mask::LetterOrDigit | Mask::LetterDigitSpace | Mask::AnyChar
                ) && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    break;
                } else if mask.right == Mask::None {
                    new_cursor = self.cursor;
                    break;
                }

                new_cursor += 1;
            }

            // debug!("CURSOR {} => {}", self.cursor, new_cursor);
            self.cursor = new_cursor;
            self.anchor = self.cursor;
            self.fix_offset();

            debug!("#[rustfmt::skip]");
            debug!("let a = {};", test_state(self));
            debug!("assert_eq_core(&b,&a);");
        }

        /// Insert the char if it matches the cursor mask and the current
        /// section is not full.
        ///
        /// `advance_cursor()` must be called before for correct functionality.
        /// Otherwise: your mileage might vary.
        pub fn insert_char(&mut self, c: char) {
            let buf = String::from(c);
            let cc = buf.as_str();

            let mut mask = &self.mask[self.cursor];

            debug!("// INSERT CHAR {:?} {:?}", mask, cc);
            debug!("#[rustfmt::skip]");
            debug!("let mut b = {};", test_state(self));
            debug!("b.insert_char({:?});", c);

            if mask.right.is_number() && (c == self.neg_sym() || c == self.pos_sym()) {
                'f: {
                    for i in mask.nr_range() {
                        match &self.mask[i] {
                            MaskToken {
                                right: Mask::Plus | Mask::Minus,
                                ..
                            } => {
                                let (b, c0, a) = grapheme::split3(&self.value(), i..i + 1);
                                let repl = if c0 == "-" {
                                    " "
                                } else if c0 == " " && c == self.neg_sym() {
                                    "-"
                                } else {
                                    c0
                                };

                                let mut buf = String::new();
                                buf.push_str(b);
                                buf.push_str(repl);
                                buf.push_str(a);
                                debug_assert_eq!(buf.len(), self.value.len());
                                self.value = buf;

                                // todo: probably no remap necessary?

                                break 'f;
                            }
                            _ => {}
                        }
                    } // else
                    {
                        let (b, c0, p, c1, a) =
                            grapheme::split_mask_match(&self.value, "-", mask.nr_range());
                        if p == "-" {
                            let mut buf = String::new();
                            buf.push_str(b);
                            buf.push_str(c0);
                            buf.push_str(" ");
                            buf.push_str(c1);
                            buf.push_str(a);
                            debug_assert_eq!(buf.len(), self.value.len());
                            self.value = buf;

                            // todo: probably no remap necessary?

                            break 'f;
                        }
                    } // else
                    if c == self.neg_sym() {
                        for i in mask.nr_range() {
                            let mask = &self.mask[i];
                            match mask {
                                MaskToken {
                                    right: Mask::Numeric(EditDirection::Rtol),
                                    ..
                                } => {
                                    let submask = &self.mask[mask.nr_range()];
                                    let (b, c0, c1, a) =
                                        grapheme::split_mask(&self.value(), i, mask.nr_range());

                                    if self.mask[i].right.can_drop_first(c1) {
                                        let mut mstr = String::new();
                                        mstr.push_str(c0);
                                        mstr.push_str("-");
                                        mstr.push_str(grapheme::drop_first(c1));
                                        let mmstr = MaskToken::remap_number(submask, &mstr);

                                        let mut buf = String::new();
                                        buf.push_str(b);
                                        buf.push_str(mmstr.as_str());
                                        buf.push_str(a);
                                        debug_assert_eq!(buf.len(), self.value.len());
                                        self.value = buf;
                                    };

                                    break 'f;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } else if mask.right.is_rtol()
                || mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none())
            {
                // boundary right/left. prefer right, change mask.
                if mask.peek_left.is_rtol() && (mask.right.is_ltor() || mask.right.is_none()) {
                    mask = &self.mask[self.cursor - 1];
                }
                let mask0 = &self.mask[mask.sec_start];
                let (b, c0, c1, a) = grapheme::split_mask(&self.value, self.cursor, mask.range());

                if mask0.right.can_drop_first(c0) && mask.right.is_valid_char(cc, self.dec_sep()) {
                    let mut mstr = String::new();
                    mstr.push_str(grapheme::drop_first(c0));
                    mstr.push(mask.right.replace_char(c, self.dec_sep()));
                    mstr.push_str(c1);

                    let submask = &self.mask[mask.sec_start..mask.sec_end];
                    let mmstr = MaskToken::remap_number(submask, &mstr);

                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(mmstr.as_str());
                    buf.push_str(a);
                    debug_assert_eq!(buf.len(), self.value.len());
                    self.value = buf;
                    // cursor stays
                }
            } else if mask.right.is_ltor() {
                let mask9 = &self.mask[mask.sec_end - 1];
                let (b, c0, c1, a) = grapheme::split_mask(&self.value, self.cursor, mask.range());

                if mask.right.can_overwrite_first(c1)
                    && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(c0);
                    buf.push(mask.right.replace_char(c, self.dec_sep()));
                    buf.push_str(grapheme::drop_first(c1));
                    buf.push_str(a);
                    debug_assert_eq!(buf.len(), self.value.len());
                    self.value = buf;

                    self.cursor += 1;
                    self.anchor = self.cursor;
                } else if mask9.right.can_drop_last(c1)
                    && mask.right.is_valid_char(cc, self.dec_sep())
                {
                    let mut buf = String::new();
                    buf.push_str(b);
                    buf.push_str(c0);
                    buf.push(mask.right.replace_char(c, self.dec_sep()));
                    buf.push_str(grapheme::drop_last(c1));
                    buf.push_str(a);
                    debug_assert_eq!(buf.len(), self.value.len());
                    self.value = buf;

                    self.cursor += 1;
                    self.anchor = self.cursor;
                }
            }

            debug!("#[rustfmt::skip]");
            debug!("let a = {};", test_state(self));
            debug!("assert_eq_core(&b,&a);");
        }

        /// Remove the selection.
        pub fn remove_selection(&mut self, selection: Range<usize>) {
            let mut buf = String::new();

            // remove section by section.
            let mut mask = &self.mask[selection.start];

            debug!("// REMOVE SELECTION {:?} {:?}", mask, selection);
            debug!("#[rustfmt::skip]");
            debug!("let mut b = {};", test_state(self));
            debug!("b.remove_selection({:?});", selection);

            let (a, _, _, _, _) =
                grapheme::split_remove_mask(self.value.as_str(), selection.clone(), mask.range());
            buf.push_str(a);

            loop {
                let (_, c0, s, c1, _) = grapheme::split_remove_mask(
                    self.value.as_str(),
                    selection.clone(),
                    mask.range(),
                );

                if mask.right.is_rtol() {
                    let remove_count = s.graphemes(true).count();
                    let fill_before = &self.mask[mask.sec_start..mask.sec_start + remove_count];

                    let mut mstr = String::new();
                    mstr.push_str(MaskToken::empty_section(fill_before).as_str());
                    mstr.push_str(c0);
                    mstr.push_str(c1);

                    let mmstr =
                        MaskToken::remap_number(&self.mask[mask.sec_start..mask.sec_end], &mstr);

                    buf.push_str(&mmstr);
                } else if mask.right.is_ltor() {
                    let c0_count = c0.graphemes(true).count();
                    let c1_count = c1.graphemes(true).count();
                    let fill_after = &self.mask[mask.sec_start + c0_count + c1_count..mask.sec_end];

                    let mut mstr = String::new();
                    mstr.push_str(c0);
                    mstr.push_str(c1);
                    mstr.push_str(MaskToken::empty_section(fill_after).as_str());

                    buf.push_str(&mstr);
                }

                if mask.sec_end >= selection.end {
                    let (_, _, a) = grapheme::split3(&self.value, mask.sec_end..mask.sec_end);
                    buf.push_str(a);
                    break;
                }

                mask = &self.mask[mask.sec_end];
            }
            debug_assert_eq!(buf.len(), self.value.len());
            self.value = buf;

            self.cursor = selection.start;
            self.anchor = self.cursor;
            self.fix_offset();

            debug!("#[rustfmt::skip]");
            debug!("let a = {};", test_state(self));
            debug!("assert_eq_core(&b,&a);");
        }

        /// Remove the previous char.
        pub fn remove_prev(&mut self) {
            if self.cursor == 0 {
                return;
            }

            let left = &self.mask[self.cursor - 1];

            debug!("// REMOVE PREV {:?} ", left);
            debug!("#[rustfmt::skip]");
            debug!("let mut b = {};", test_state(self));
            debug!("b.remove_prev();");

            let (b, c0, _s, c1, a) = grapheme::split_remove_mask(
                self.value.as_str(),
                self.cursor - 1..self.cursor,
                left.range(),
            );

            // remove and fill with empty
            if left.right.is_rtol() {
                let fill_mask = &self.mask[left.sec_start..left.sec_start + 1];
                let mut mstr = String::new();
                mstr.push_str(MaskToken::empty_section(fill_mask).as_str());
                mstr.push_str(c0);
                mstr.push_str(c1);
                let mmstr =
                    MaskToken::remap_number(&self.mask[left.sec_start..left.sec_end], &mstr);

                let mut buf = String::new();
                buf.push_str(b);
                buf.push_str(&mmstr);
                buf.push_str(a);
                debug_assert_eq!(buf.len(), self.value.len());
                self.value = buf;
            } else if left.right.is_ltor() {
                let mut buf = String::new();
                buf.push_str(b);
                buf.push_str(c0);
                buf.push_str(c1);

                let c0_count = c0.graphemes(true).count();
                let c1_count = c1.graphemes(true).count();
                let fill_mask = &self.mask[left.sec_start + c0_count + c1_count..left.sec_end];
                buf.push_str(MaskToken::empty_section(fill_mask).as_str());

                buf.push_str(a);
                debug_assert_eq!(buf.len(), self.value.len());
                self.value = buf;
            }

            // place cursor after deletion
            if left.right.is_rtol() {
                let (_b, s, _a) = grapheme::split3(&self.value(), left.sec_start..left.sec_end);
                let sec_mask = &self.mask[left.sec_start..left.sec_end];
                if s == MaskToken::empty_section(sec_mask) {
                    self.cursor = left.sec_start;
                    self.anchor = self.cursor;
                } else {
                    // cursor stays
                }
            } else if left.right.is_ltor() {
                self.cursor -= 1;
                self.anchor = self.cursor;
            }
            self.fix_offset();

            debug!("#[rustfmt::skip]");
            debug!("let a = {};", test_state(self));
            debug!("assert_eq_core(&b,&a);");
        }

        /// Remove the next char.
        pub fn remove_next(&mut self) {
            if self.cursor == self.mask.len() - 1 {
                return;
            }

            let right = &self.mask[self.cursor];

            debug!("// REMOVE NEXT {:?} ", right);
            debug!("#[rustfmt::skip]");
            debug!("let mut b = {};", test_state(self));
            debug!("b.remove_next();");

            let (b, c0, _, c1, a) = grapheme::split_remove_mask(
                self.value.as_str(),
                self.cursor..self.cursor + 1,
                right.range(),
            );

            // remove and fill with empty
            if right.right.is_rtol() {
                let mut mstr = String::new();
                let fill_mask = &self.mask[right.sec_start..right.sec_start + 1];
                mstr.push_str(MaskToken::empty_section(fill_mask).as_str());
                mstr.push_str(c0);
                mstr.push_str(c1);
                let mmstr =
                    MaskToken::remap_number(&self.mask[right.sec_start..right.sec_end], &mstr);

                let mut buf = String::new();
                buf.push_str(b);
                buf.push_str(&mmstr);
                buf.push_str(a);
                debug_assert_eq!(buf.len(), self.value.len());
                self.value = buf;
            } else if right.right.is_ltor() {
                let mut buf = String::new();
                buf.push_str(b);
                buf.push_str(c0);
                buf.push_str(c1);

                let c0_count = c0.graphemes(true).count();
                let c1_count = c1.graphemes(true).count();
                let fill_mask = &self.mask[right.sec_start + c0_count + c1_count..right.sec_end];
                buf.push_str(MaskToken::empty_section(fill_mask).as_str());

                buf.push_str(a);
                debug_assert_eq!(buf.len(), self.value.len());
                self.value = buf;
            }

            // place cursor after deletion
            if right.right.is_rtol() {
                self.cursor += 1;
                self.anchor = self.cursor;
            } else if right.right.is_ltor() {
                // cursor stays
            }
            self.fix_offset();

            debug!("#[rustfmt::skip]");
            debug!("let a = {};", test_state(self));
            debug!("assert_eq_core(&b,&a);");
        }
    }

    pub fn parse_mask(mask_str: &str) -> Vec<MaskToken> {
        let mut out = Vec::<MaskToken>::new();

        let mut start_id = 0;
        let mut id = 0;
        let mut start_nr = 0;
        let mut nr_id = 0;
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
                    "+" => Mask::Plus,
                    "-" => Mask::Minus,
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
                    "\\" => {
                        esc = true;
                        continue;
                    }
                    s => Mask::Separator(Box::from(s)),
                }
            };

            match mask {
                Mask::Digit0(_)
                | Mask::Digit(_)
                | Mask::Numeric(_)
                | Mask::GroupingSep
                | Mask::Plus
                | Mask::Minus => {
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

            if matches!(mask, Mask::Separator(_)) || mask.number() != last_mask.number() {
                for j in start_nr..idx {
                    out[j].nr_id = nr_id;
                    out[j].nr_start = start_nr;
                    out[j].nr_end = idx;
                }
                nr_id += 1;
                start_nr = idx;
            }
            if matches!(mask, Mask::Separator(_)) || mask.section() != last_mask.section() {
                for j in start_id..idx {
                    out[j].sec_id = id;
                    out[j].sec_start = start_id;
                    out[j].sec_end = idx;
                }
                id += 1;
                start_id = idx;
            }

            let tok = MaskToken {
                nr_id: 0,
                nr_start: 0,
                nr_end: 0,
                sec_id: 0,
                sec_start: 0,
                sec_end: 0,
                peek_left: last_mask,
                right: mask.clone(),
                edit: mask.edit_value().into(),
                display: mask.disp_value().into(),
            };
            out.push(tok);

            idx += 1;
            last_mask = mask;
        }
        for j in start_nr..out.len() {
            out[j].nr_id = nr_id;
            out[j].nr_start = start_nr;
            out[j].nr_end = mask_str.graphemes(true).count();
        }
        for j in start_id..out.len() {
            out[j].sec_id = id;
            out[j].sec_start = start_id;
            out[j].sec_end = mask_str.graphemes(true).count();
        }

        out
    }

    /// dump the current state as code.
    pub fn test_state(m: &InputMaskCore) -> String {
        use std::fmt::Write;

        let mut buf = String::new();
        _ = write!(buf, "test_input_mask_core(");
        _ = write!(buf, "{:?}, ", m.mask());
        _ = write!(buf, "{:?}, ", m.value);
        _ = write!(buf, "{:?}, ", m.rendered);
        _ = write!(buf, "{:?}, ", m.len);
        _ = write!(buf, "{:?}, ", m.offset);
        _ = write!(buf, "{:?}, ", m.width);
        _ = write!(buf, "{:?}, ", m.cursor);
        _ = write!(buf, "{:?},", m.anchor);
        if let Some(sym) = &m.sym {
            _ = write!(
                buf,
                "Some(\"{}{}{}{}{}{}\")",
                sym.decimal_sep,
                sym.decimal_grp,
                sym.negative_sym,
                sym.positive_sym,
                sym.exponent_upper_sym,
                sym.exponent_lower_sym
            );
        } else {
            _ = write!(buf, "sym: None, ");
        }
        _ = write!(buf, ")");
        buf
    }

    #[track_caller]
    pub fn assert_eq_core(a: &InputMaskCore, b: &InputMaskCore) {
        assert_eq!(b.value, a.value);
        assert_eq!(b.rendered, a.rendered);
        assert_eq!(b.len, a.len);
        assert_eq!(b.offset, a.offset);
        assert_eq!(b.width, a.width);
        assert_eq!(b.cursor, a.cursor);
        assert_eq!(b.anchor, a.anchor);
    }

    pub fn test_input_mask_core(
        mask: &str,
        value: &str,
        rendered: &str,
        len: usize,
        offset: usize,
        width: usize,
        cursor: usize,
        anchor: usize,
        sym: Option<&str>,
    ) -> InputMaskCore {
        InputMaskCore {
            mask: parse_mask(mask),
            value: value.to_string(),
            rendered: rendered.to_string(),
            len,
            offset,
            width,
            cursor,
            anchor,
            sym: sym.map(|sym| parse_number_symbols(sym)),
        }
    }

    pub fn parse_number_symbols(s: &str) -> Rc<NumberSymbols> {
        let mut s = s.chars();
        Rc::new(NumberSymbols {
            decimal_sep: s.next().expect("decimal_sep"),
            decimal_grp: s.next().expect("decimal_grp"),
            negative_sym: s.next().expect("negative_sym"),
            positive_sym: s.next().expect("positive_sym"),
            exponent_upper_sym: s.next().expect("exponent_upper_sym"),
            exponent_lower_sym: s.next().expect("exponent_lower_sym"),
        })
    }
}
