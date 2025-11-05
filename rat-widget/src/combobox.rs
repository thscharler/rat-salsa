use crate::_private::NonExhaustive;
use crate::choice::{
    Choice, ChoiceClose, ChoiceFocus, ChoicePopup, ChoiceSelect, ChoiceState, ChoiceStyle,
    ChoiceWidget,
};
use crate::combobox::event::ComboboxOutcome;
use crate::event::ChoiceOutcome;
use crate::text::HasScreenCursor;
use rat_event::util::{MouseFlags, item_at, mouse_trap};
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Popup, Regular, ct_event};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_popup::Placement;
use rat_popup::event::PopupOutcome;
use rat_reloc::{RelocatableState, relocate_area};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollAreaState};
use rat_text::TextStyle;
use rat_text::event::TextOutcome;
use rat_text::text_input::{TextInput, TextInputState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget};
use std::cmp::max;

#[derive(Debug)]
pub struct Combobox<'a> {
    choice: Choice<'a, String>,
    text: TextInput<'a>,
}

#[derive(Debug)]
pub struct ComboboxWidget<'a> {
    choice: ChoiceWidget<'a, String>,
    text: TextInput<'a>,
}

#[derive(Debug)]
pub struct ComboboxPopup<'a> {
    choice: ChoicePopup<'a, String>,
}

#[derive(Debug, Clone)]
pub struct ComboboxStyle {
    pub choice: ChoiceStyle,
    pub text: TextStyle,

    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug)]
pub struct ComboboxState {
    /// Total area.
    /// __read only__. renewed with each render.
    pub area: Rect,
    /// Core
    pub choice: ChoiceState<String>,
    /// Text
    pub text: TextInputState,

    /// Focus flag.
    /// __read+write__
    pub focus: FocusFlag,
    /// Mouse util.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};
    use rat_popup::event::PopupOutcome;

    /// Result value for event-handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ComboboxOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
        /// An item has been selected.
        Value,
        /// Textinput has changed
        TextChanged,
    }

    impl ConsumedEvent for ComboboxOutcome {
        fn is_consumed(&self) -> bool {
            *self != ComboboxOutcome::Continue
        }
    }

    impl From<Outcome> for ComboboxOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => ComboboxOutcome::Continue,
                Outcome::Unchanged => ComboboxOutcome::Unchanged,
                Outcome::Changed => ComboboxOutcome::Changed,
            }
        }
    }

    impl From<ComboboxOutcome> for Outcome {
        fn from(value: ComboboxOutcome) -> Self {
            match value {
                ComboboxOutcome::Continue => Outcome::Continue,
                ComboboxOutcome::Unchanged => Outcome::Unchanged,
                ComboboxOutcome::Changed => Outcome::Changed,
                ComboboxOutcome::Value => Outcome::Changed,
                ComboboxOutcome::TextChanged => Outcome::Changed,
            }
        }
    }

    impl From<PopupOutcome> for ComboboxOutcome {
        fn from(value: PopupOutcome) -> Self {
            match value {
                PopupOutcome::Continue => ComboboxOutcome::Continue,
                PopupOutcome::Unchanged => ComboboxOutcome::Unchanged,
                PopupOutcome::Changed => ComboboxOutcome::Changed,
                PopupOutcome::Hide => ComboboxOutcome::Changed,
            }
        }
    }
}

impl Default for ComboboxStyle {
    fn default() -> Self {
        Self {
            choice: Default::default(),
            text: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for Combobox<'_> {
    fn default() -> Self {
        Self {
            choice: Choice::default().skip_item_render(true),
            text: Default::default(),
        }
    }
}

impl<'a> Combobox<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Button text.
    #[inline]
    pub fn items<V: Into<Line<'a>>>(
        mut self,
        items: impl IntoIterator<Item = (String, V)>,
    ) -> Self {
        self.choice = self.choice.items(items);
        self
    }

    /// Add an item.
    pub fn item(mut self, value: String, item: impl Into<Line<'a>>) -> Self {
        self.choice = self.choice.item(value, item);
        self
    }

    /// Can return to default with user interaction.
    pub fn default_value(mut self, default: String) -> Self {
        self.choice = self.choice.default_value(default);
        self
    }

    /// Combined styles.
    pub fn styles(mut self, styles: ComboboxStyle) -> Self {
        self.choice = self.choice.styles(styles.choice);
        self.text = self.text.styles(styles.text);
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.choice = self.choice.style(style);
        self.text = self.text.style(style);
        self
    }

    /// Style for the down button.
    pub fn button_style(mut self, style: Style) -> Self {
        self.choice = self.choice.button_style(style);
        self
    }

    /// Selection in the list.
    pub fn select_style(mut self, style: Style) -> Self {
        self.choice = self.choice.select_style(style);
        self
    }

    /// Focused style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.choice = self.choice.focus_style(style);
        self
    }

    /// Textstyle
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text = self.text.styles(style);
        self
    }

    /// Block for the main widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.choice = self.choice.block(block);
        self
    }

    /// Alignment of the popup.
    ///
    /// __Default__
    /// Default is Left.
    pub fn popup_alignment(mut self, alignment: Alignment) -> Self {
        self.choice = self.choice.popup_alignment(alignment);
        self
    }

    /// Placement of the popup.
    ///
    /// __Default__
    /// Default is BelowOrAbove.
    pub fn popup_placement(mut self, placement: Placement) -> Self {
        self.choice = self.choice.popup_placement(placement);
        self
    }

    /// Outer boundary for the popup.
    pub fn popup_boundary(mut self, boundary: Rect) -> Self {
        self.choice = self.choice.popup_boundary(boundary);
        self
    }

    /// Override the popup length.
    ///
    /// __Default__
    /// Defaults to the number of items or 5.
    pub fn popup_len(mut self, len: u16) -> Self {
        self.choice = self.choice.popup_len(len);
        self
    }

    /// Base style for the popup.
    pub fn popup_style(mut self, style: Style) -> Self {
        self.choice = self.choice.popup_style(style);
        self
    }

    /// Block for the popup.
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.choice = self.choice.popup_block(block);
        self
    }

    /// Scroll for the popup.
    pub fn popup_scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.choice = self.choice.popup_scroll(scroll);
        self
    }

    /// Adds an extra offset to the widget area.
    ///
    /// This can be used to
    /// * place the widget under the mouse cursor.
    /// * align the widget not by the outer bounds but by
    ///   the text content.
    pub fn popup_offset(mut self, offset: (i16, i16)) -> Self {
        self.choice = self.choice.popup_offset(offset);
        self
    }

    /// Sets only the x offset.
    /// See [offset](Self::popup_offset)
    pub fn popup_x_offset(mut self, offset: i16) -> Self {
        self.choice = self.choice.popup_x_offset(offset);
        self
    }

    /// Sets only the y offset.
    /// See [offset](Self::popup_offset)
    pub fn popup_y_offset(mut self, offset: i16) -> Self {
        self.choice = self.choice.popup_y_offset(offset);
        self
    }

    /// Sets the behaviour for selecting from the list.
    pub fn behave_focus(mut self, focus: ChoiceFocus) -> Self {
        self.choice = self.choice.behave_focus(focus);
        self
    }

    /// Sets the behaviour for selecting from the list.
    pub fn behave_select(mut self, select: ChoiceSelect) -> Self {
        self.choice = self.choice.behave_select(select);
        self
    }

    /// Sets the behaviour for closing the list.
    pub fn behave_close(mut self, close: ChoiceClose) -> Self {
        self.choice = self.choice.behave_close(close);
        self
    }

    /// Inherent width.
    pub fn width(&self) -> u16 {
        self.choice.width()
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        self.choice.height()
    }

    /// Choice itself doesn't render.
    ///
    /// This builds the widgets from the parameters set for Choice.
    pub fn into_widgets(self) -> (ComboboxWidget<'a>, ComboboxPopup<'a>) {
        let (choice, choice_popup) = self.choice.into_widgets();
        (
            ComboboxWidget {
                choice,
                text: self.text,
            },
            ComboboxPopup {
                choice: choice_popup,
            },
        )
    }
}

impl<'a> ComboboxWidget<'a> {
    /// Inherent width.
    pub fn width(&self) -> u16 {
        self.choice.width()
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        self.choice.height()
    }
}

impl<'a> StatefulWidget for &ComboboxWidget<'a> {
    type State = ComboboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_combobox(self, area, buf, state);
    }
}

impl StatefulWidget for ComboboxWidget<'_> {
    type State = ComboboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_combobox(&self, area, buf, state);
    }
}

fn render_combobox(
    widget: &ComboboxWidget<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ComboboxState,
) {
    state.area = area;
    (&widget.choice).render(area, buf, &mut state.choice);
    (&widget.text).render(state.choice.item_area, buf, &mut state.text);
}

impl ComboboxPopup<'_> {
    /// Calculate the layout for the popup before rendering.
    /// Area is the area of the ChoiceWidget not the ChoicePopup.
    pub fn layout(&self, area: Rect, buf: &mut Buffer, state: &mut ComboboxState) -> Rect {
        self.choice.layout(area, buf, &mut state.choice)
    }
}

impl StatefulWidget for &ComboboxPopup<'_> {
    type State = ComboboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(self, area, buf, state);
    }
}

impl StatefulWidget for ComboboxPopup<'_> {
    type State = ComboboxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(&self, area, buf, state);
    }
}

fn render_popup(
    widget: &ComboboxPopup<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ComboboxState,
) {
    (&widget.choice).render(area, buf, &mut state.choice);
}

impl Clone for ComboboxState {
    fn clone(&self) -> Self {
        let mut text = self.text.clone();
        let mut choice = self.choice.clone();
        let focus = focus_cb("", text.focus, choice.focus);
        text.focus = focus.clone();
        choice.focus = focus.clone();

        Self {
            area: self.area,
            choice,
            text,
            focus,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for ComboboxState {
    fn default() -> Self {
        let mut text = TextInputState::default();
        let mut choice = ChoiceState::default();
        let focus = focus_cb("", text.focus, choice.focus);
        text.focus = focus.clone();
        choice.focus = focus.clone();

        Self {
            area: Default::default(),
            choice,
            text,
            focus,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

fn focus_cb(name: &str, choice: FocusFlag, text: FocusFlag) -> FocusFlag {
    let flag = FocusFlag::named(name);

    let choice_clone = choice.clone();
    let text_clone = text.clone();
    flag.on_lost(move || {
        choice_clone.call_on_lost();
        text_clone.call_on_lost();
    });
    let choice_clone = choice.clone();
    let text_clone = text.clone();
    flag.on_gained(move || {
        choice_clone.call_on_gained();
        text_clone.call_on_gained();
    });
    flag
}

impl HasScreenCursor for ComboboxState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.text.screen_cursor()
    }
}

impl HasFocus for ComboboxState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl RelocatableState for ComboboxState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.choice.relocate(shift, clip);
        self.text.relocate(shift, clip);
    }
}

impl ComboboxState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        let mut text = TextInputState::default();
        let mut choice = ChoiceState::default();
        let focus = focus_cb(name, text.focus, choice.focus);
        text.focus = focus.clone();
        choice.focus = focus.clone();

        Self {
            area: Default::default(),
            choice,
            text,
            focus,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// Popup is active?
    pub fn is_popup_active(&self) -> bool {
        self.choice.is_popup_active()
    }

    /// Flip the popup state.
    pub fn flip_popup_active(&mut self) {
        self.choice.flip_popup_active();
    }

    /// Show the popup.
    pub fn set_popup_active(&mut self, active: bool) -> bool {
        self.choice.set_popup_active(active)
    }

    /// Set a default-value other than T::default()
    ///
    /// The starting value will still be T::default()
    /// after this. You must call clear() to use this
    /// default.
    ///
    /// This default will be overridden by a default set
    /// on the widget.
    pub fn set_default_value(&mut self, default_value: Option<String>) {
        self.choice.set_default_value(default_value);
    }

    /// A default value.
    pub fn default_value(&self) -> &Option<String> {
        self.choice.default_value()
    }

    /// Select the given value.
    ///
    /// If the value doesn't exist in the list or the list is
    /// empty the value will still be set, but selected will be
    /// None. The list will be empty before the first render, but
    /// the first thing render will do is set the list of values.
    /// This will adjust the selected index if possible.
    /// It's still ok to set a value here that can not be represented.
    /// As long as there is no user interaction, the same value
    /// will be returned by value().
    pub fn set_value(&mut self, value: impl Into<String>) -> bool {
        let value = value.into();
        self.text.set_value(value.clone());
        self.choice.set_value(value)
    }

    /// Get the selected value.
    pub fn value(&self) -> String {
        self.text.value()
    }

    /// Select the default value or T::default.
    pub fn clear(&mut self) -> bool {
        self.text.clear() || self.choice.clear()
    }

    /// Select the value at index. This will set the value
    /// to the given index in the value-list. If the index is
    /// out of bounds or the value-list is empty it will
    /// set selected to None and leave the value as is.
    /// The list is empty before the first render so this
    /// may not work as expected.
    ///
    /// The selected index is a best effort artefact, the main
    /// thing is the value itself.
    ///
    /// Use of set_value() is preferred.
    pub fn select(&mut self, select: usize) -> bool {
        if self.choice.select(select) {
            self.text.set_value(self.choice.value());
            true
        } else {
            false
        }
    }

    /// Returns the selected index or None if the
    /// value is not in the list or the list is empty.
    ///
    /// You can still get the value set with set_value() though.
    pub fn selected(&self) -> Option<usize> {
        self.choice.selected()
    }

    /// Any items?
    pub fn is_empty(&self) -> bool {
        self.choice.is_empty()
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        self.choice.len()
    }

    /// Scroll offset for the item list.
    pub fn clear_offset(&mut self) {
        self.choice.set_offset(0);
    }

    /// Scroll offset for the item list.
    pub fn set_offset(&mut self, offset: usize) -> bool {
        self.choice.set_offset(offset)
    }

    /// Scroll offset for the item list.
    pub fn offset(&self) -> usize {
        self.choice.offset()
    }

    /// Scroll offset for the item list.
    pub fn max_offset(&self) -> usize {
        self.choice.max_offset()
    }

    /// Page length for the item list.
    pub fn page_len(&self) -> usize {
        self.choice.page_len()
    }

    /// Scroll unit for the item list.
    pub fn scroll_by(&self) -> usize {
        self.choice.scroll_by()
    }

    /// Scroll the item list to the selected value.
    pub fn scroll_to_selected(&mut self) -> bool {
        self.choice.scroll_to_selected()
    }
}

impl ComboboxState {
    /// Select by first character.
    pub fn select_by_char(&mut self, c: char) -> bool {
        if self.choice.select_by_char(c) {
            self.text.set_value(self.choice.value());
            true
        } else {
            false
        }
    }

    /// Select by first character. Reverse direction
    pub fn reverse_select_by_char(&mut self, c: char) -> bool {
        if self.choice.reverse_select_by_char(c) {
            self.text.set_value(self.choice.value());
            true
        } else {
            false
        }
    }

    /// Select at position
    pub fn move_to(&mut self, n: usize) -> ComboboxOutcome {
        match self.choice.move_to(n) {
            ChoiceOutcome::Continue => ComboboxOutcome::Continue,
            ChoiceOutcome::Unchanged => ComboboxOutcome::Unchanged,
            ChoiceOutcome::Changed => ComboboxOutcome::Changed,
            ChoiceOutcome::Value => {
                self.text.set_value(self.choice.value());
                ComboboxOutcome::Value
            }
        }
    }

    /// Select next entry.
    pub fn move_down(&mut self, n: usize) -> ComboboxOutcome {
        match self.choice.move_down(n) {
            ChoiceOutcome::Continue => ComboboxOutcome::Continue,
            ChoiceOutcome::Unchanged => ComboboxOutcome::Unchanged,
            ChoiceOutcome::Changed => ComboboxOutcome::Changed,
            ChoiceOutcome::Value => {
                self.text.set_value(self.choice.value());
                ComboboxOutcome::Value
            }
        }
    }

    /// Select prev entry.
    pub fn move_up(&mut self, n: usize) -> ComboboxOutcome {
        match self.choice.move_up(n) {
            ChoiceOutcome::Continue => ComboboxOutcome::Continue,
            ChoiceOutcome::Unchanged => ComboboxOutcome::Unchanged,
            ChoiceOutcome::Changed => ComboboxOutcome::Changed,
            ChoiceOutcome::Value => {
                self.text.set_value(self.choice.value());
                ComboboxOutcome::Value
            }
        }
    }
}

impl HandleEvent<crossterm::event::Event, Popup, ComboboxOutcome> for ComboboxState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> ComboboxOutcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Esc) => {
                    if self.set_popup_active(false) {
                        ComboboxOutcome::Changed
                    } else {
                        ComboboxOutcome::Continue
                    }
                }
                ct_event!(keycode press Down) => self.move_down(1),
                ct_event!(keycode press Up) => self.move_up(1),
                ct_event!(keycode press PageUp) => self.move_up(self.page_len()),
                ct_event!(keycode press PageDown) => self.move_down(self.page_len()),
                ct_event!(keycode press ALT-Home) => self.move_to(0),
                ct_event!(keycode press ALT-End) => self.move_to(self.len().saturating_sub(1)),
                crossterm::event::Event::Key(_) => match self.text.handle(event, Regular) {
                    TextOutcome::Continue => ComboboxOutcome::Continue,
                    TextOutcome::Unchanged => ComboboxOutcome::Unchanged,
                    TextOutcome::Changed => ComboboxOutcome::Changed,
                    TextOutcome::TextChanged => ComboboxOutcome::TextChanged,
                },
                _ => ComboboxOutcome::Continue,
            }
        } else {
            ComboboxOutcome::Continue
        };

        if !r.is_consumed() {
            self.handle(event, MouseOnly)
        } else {
            r
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, ComboboxOutcome> for ComboboxState {
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _qualifier: MouseOnly,
    ) -> ComboboxOutcome {
        let r0 = handle_mouse(self, event);
        let r1 = handle_select(self, event);
        let r2 = handle_close(self, event);
        let mut r = max(r0, max(r1, r2));

        r = r.or_else(|| match self.text.handle(event, MouseOnly) {
            TextOutcome::Continue => ComboboxOutcome::Continue,
            TextOutcome::Unchanged => ComboboxOutcome::Unchanged,
            TextOutcome::Changed => ComboboxOutcome::Changed,
            TextOutcome::TextChanged => ComboboxOutcome::TextChanged,
        });
        r = r.or_else(|| mouse_trap(event, self.choice.popup.area).into());

        r
    }
}

fn handle_mouse(state: &mut ComboboxState, event: &crossterm::event::Event) -> ComboboxOutcome {
    match event {
        ct_event!(mouse down Left for x,y)
            if state.choice.button_area.contains((*x, *y).into()) =>
        {
            if !state.gained_focus() {
                state.flip_popup_active();
                ComboboxOutcome::Changed
            } else {
                // hide is down by self.popup.handle() as this click
                // is outside the popup area!!
                ComboboxOutcome::Continue
            }
        }
        ct_event!(mouse down Left for x,y)
        | ct_event!(mouse down Right for x,y)
        | ct_event!(mouse down Middle for x,y)
            if !state.choice.item_area.contains((*x, *y).into())
                && !state.choice.button_area.contains((*x, *y).into()) =>
        {
            match state.choice.popup.handle(event, Popup) {
                PopupOutcome::Hide => {
                    state.set_popup_active(false);
                    ComboboxOutcome::Changed
                }
                r => r.into(),
            }
        }
        _ => ComboboxOutcome::Continue,
    }
}

fn handle_select(state: &mut ComboboxState, event: &crossterm::event::Event) -> ComboboxOutcome {
    match state.choice.behave_select {
        ChoiceSelect::MouseScroll => {
            let mut sas = ScrollAreaState::new()
                .area(state.choice.popup.area)
                .v_scroll(&mut state.choice.popup_scroll);
            let mut r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(n) => state.move_up(n),
                ScrollOutcome::Down(n) => state.move_down(n),
                ScrollOutcome::VPos(n) => state.move_to(n),
                _ => ComboboxOutcome::Continue,
            };

            r = r.or_else(|| match event {
                ct_event!(mouse down Left for x,y)
                    if state.choice.popup.area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.choice.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                ct_event!(mouse drag Left for x,y)
                    if state.choice.popup.area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.choice.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                _ => ComboboxOutcome::Continue,
            });
            r
        }
        ChoiceSelect::MouseMove => {
            // effect: move the content below the mouse and keep visible selection.
            let mut r = if let Some(selected) = state.choice.core.selected() {
                let rel_sel = selected.saturating_sub(state.offset());
                let mut sas = ScrollAreaState::new()
                    .area(state.choice.popup.area)
                    .v_scroll(&mut state.choice.popup_scroll);
                match sas.handle(event, MouseOnly) {
                    ScrollOutcome::Up(n) => {
                        state.choice.popup_scroll.scroll_up(n);
                        if state.select(state.offset() + rel_sel) {
                            ComboboxOutcome::Value
                        } else {
                            ComboboxOutcome::Unchanged
                        }
                    }
                    ScrollOutcome::Down(n) => {
                        state.choice.popup_scroll.scroll_down(n);
                        if state.select(state.offset() + rel_sel) {
                            ComboboxOutcome::Value
                        } else {
                            ComboboxOutcome::Unchanged
                        }
                    }
                    ScrollOutcome::VPos(n) => {
                        if state.choice.popup_scroll.set_offset(n) {
                            ComboboxOutcome::Value
                        } else {
                            ComboboxOutcome::Unchanged
                        }
                    }
                    _ => ComboboxOutcome::Continue,
                }
            } else {
                ComboboxOutcome::Continue
            };

            r = r.or_else(|| match event {
                ct_event!(mouse moved for x,y)
                    if state.choice.popup.area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.choice.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                _ => ComboboxOutcome::Continue,
            });
            r
        }
        ChoiceSelect::MouseClick => {
            // effect: move the content below the mouse and keep visible selection.
            let mut sas = ScrollAreaState::new()
                .area(state.choice.popup.area)
                .v_scroll(&mut state.choice.popup_scroll);
            let mut r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(n) => {
                    if state.choice.popup_scroll.scroll_up(n) {
                        ComboboxOutcome::Changed
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                ScrollOutcome::Down(n) => {
                    if state.choice.popup_scroll.scroll_down(n) {
                        ComboboxOutcome::Changed
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                ScrollOutcome::VPos(n) => {
                    if state.choice.popup_scroll.set_offset(n) {
                        ComboboxOutcome::Changed
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                _ => ComboboxOutcome::Continue,
            };

            r = r.or_else(|| match event {
                ct_event!(mouse down Left for x,y)
                    if state.choice.popup.area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.choice.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                ct_event!(mouse drag Left for x,y)
                    if state.choice.popup.area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.choice.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ComboboxOutcome::Unchanged
                    }
                }
                _ => ComboboxOutcome::Continue,
            });
            r
        }
    }
}

fn handle_close(state: &mut ComboboxState, event: &crossterm::event::Event) -> ComboboxOutcome {
    match state.choice.behave_close {
        ChoiceClose::SingleClick => match event {
            ct_event!(mouse down Left for x,y)
                if state.choice.popup.area.contains((*x, *y).into()) =>
            {
                if let Some(n) = item_at(&state.choice.item_areas, *x, *y) {
                    let r = state.move_to(state.offset() + n);
                    let s = if state.set_popup_active(false) {
                        ComboboxOutcome::Changed
                    } else {
                        ComboboxOutcome::Unchanged
                    };
                    max(r, s)
                } else {
                    ComboboxOutcome::Unchanged
                }
            }
            _ => ComboboxOutcome::Continue,
        },
        ChoiceClose::DoubleClick => match event {
            ct_event!(mouse any for m) if state.mouse.doubleclick(state.choice.popup.area, m) => {
                if let Some(n) = item_at(&state.choice.item_areas, m.column, m.row) {
                    let r = state.move_to(state.offset() + n);
                    let s = if state.set_popup_active(false) {
                        ComboboxOutcome::Changed
                    } else {
                        ComboboxOutcome::Unchanged
                    };
                    max(r, s)
                } else {
                    ComboboxOutcome::Unchanged
                }
            }
            _ => ComboboxOutcome::Continue,
        },
    }
}

/// Handle events for the popup.
/// Call before other handlers to deal with intersections
/// with other widgets.
pub fn handle_events(
    state: &mut ComboboxState,
    focus: bool,
    event: &crossterm::event::Event,
) -> ComboboxOutcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut ComboboxState,
    event: &crossterm::event::Event,
) -> ComboboxOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
