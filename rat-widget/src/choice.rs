//!
//! Choice/Select widget.
//!
//! ```rust no_run
//! use rat_popup::Placement;
//! use rat_scrolled::Scroll;
//! use rat_widget::choice::{Choice, ChoiceState};
//! # use ratatui::prelude::*;
//! # use ratatui::widgets::Block;
//! # let mut buf = Buffer::default();
//! # let mut cstate = ChoiceState::default();
//! # let mut max_bounds: Rect = Rect::default();
//!
//! let (widget, popup) = Choice::new()
//!         .item(1, "Carrots")
//!         .item(2, "Potatoes")
//!         .item(3, "Onions")
//!         .item(4, "Peas")
//!         .item(5, "Beans")
//!         .item(6, "Tomatoes")
//!         .popup_block(Block::bordered())
//!         .popup_placement(Placement::AboveOrBelow)
//!         .popup_boundary(max_bounds)
//!         .into_widgets();
//!  widget.render(Rect::new(3,3,15,1), &mut buf, &mut cstate);
//!
//!  // ... render other widgets
//!
//!  popup.render(Rect::new(3,3,15,1), &mut buf, &mut cstate);
//!
//! ```
//!
use crate::_private::NonExhaustive;
use crate::choice::core::ChoiceCore;
use crate::event::ChoiceOutcome;
use crate::util::{block_size, revert_style};
use rat_event::util::{item_at, mouse_trap, MouseFlags};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Popup};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_popup::event::PopupOutcome;
use rat_popup::{Placement, PopupCore, PopupCoreState, PopupStyle};
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::marker::PhantomData;
use std::rc::Rc;

/// Enum controling the behaviour of the Choice.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceSelect {
    /// Change the selection with the mouse-wheel.
    #[default]
    MouseScroll,
    /// Change the selection just by moving.
    MouseMove,
    /// Change the selection with a click only
    MouseClick,
}

/// Enum controlling the behaviour of the Choice.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceClose {
    /// Close the popup with a single click.
    #[default]
    SingleClick,
    /// Close the popup with double-click.
    DoubleClick,
}

/// Choice.
///
/// Select one of a list. No editable mode for this widget.
///
/// This doesn't render itself. [into_widgets](Choice::into_widgets)
/// creates the base part and the popup part, which are rendered
/// separately.
///
#[derive(Debug, Clone)]
pub struct Choice<'a, T>
where
    T: PartialEq + Clone + Default,
{
    values: Rc<RefCell<Vec<T>>>,
    default_value: Option<T>,
    items: Rc<RefCell<Vec<Line<'a>>>>,

    style: Style,
    button_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,

    popup_alignment: Alignment,
    popup_placement: Placement,
    popup_len: Option<u16>,
    popup: PopupCore<'a>,

    behave_select: ChoiceSelect,
    behave_close: ChoiceClose,
}

/// Renders the main widget.
#[derive(Debug)]
pub struct ChoiceWidget<'a, T>
where
    T: PartialEq,
{
    values: Rc<RefCell<Vec<T>>>,
    default_value: Option<T>,
    items: Rc<RefCell<Vec<Line<'a>>>>,

    style: Style,
    button_style: Option<Style>,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,
    len: Option<u16>,

    behave_select: ChoiceSelect,
    behave_close: ChoiceClose,

    _phantom: PhantomData<T>,
}

/// Renders the popup. This is called after the rest
/// of the area is rendered and overwrites to display itself.
#[derive(Debug)]
pub struct ChoicePopup<'a, T>
where
    T: PartialEq,
{
    items: Rc<RefCell<Vec<Line<'a>>>>,

    style: Style,
    select_style: Option<Style>,

    popup_alignment: Alignment,
    popup_placement: Placement,
    popup_len: Option<u16>,
    popup: PopupCore<'a>,

    _phantom: PhantomData<T>,
}

/// Combined style.
#[derive(Debug, Clone)]
pub struct ChoiceStyle {
    pub style: Style,
    pub button: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub block: Option<Block<'static>>,

    pub popup: PopupStyle,
    pub popup_len: Option<u16>,

    pub behave_select: Option<ChoiceSelect>,
    pub behave_close: Option<ChoiceClose>,

    pub non_exhaustive: NonExhaustive,
}

/// State.
#[derive(Debug)]
pub struct ChoiceState<T = usize>
where
    T: PartialEq + Clone + Default,
{
    /// Total area.
    /// __read only__. renewed with each render.
    pub area: Rect,
    /// First char of each item for navigation.
    /// __read only__. renewed with each render.
    pub nav_char: Vec<Vec<char>>,
    /// Item area in the main widget.
    /// __read only__. renewed with each render.
    pub item_area: Rect,
    /// Button area in the main widget.
    /// __read only__. renewed with each render.
    pub button_area: Rect,
    /// Visible items in the popup.
    /// __read only__. renewed with each render.
    pub item_areas: Vec<Rect>,
    /// Core
    pub core: ChoiceCore<T>,
    /// Popup state.
    pub popup: PopupCoreState,
    /// Behaviour for selecting from the choice popup.
    /// __read only__ renewed with each render.
    pub behave_select: ChoiceSelect,
    /// Behaviour for closing the choice popup.
    /// __read only__ renewed with each render.
    pub behave_close: ChoiceClose,

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
    pub enum ChoiceOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
        /// An item has been selected.
        Value,
    }

    impl ConsumedEvent for ChoiceOutcome {
        fn is_consumed(&self) -> bool {
            *self != ChoiceOutcome::Continue
        }
    }

    impl From<Outcome> for ChoiceOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => ChoiceOutcome::Continue,
                Outcome::Unchanged => ChoiceOutcome::Unchanged,
                Outcome::Changed => ChoiceOutcome::Changed,
            }
        }
    }

    impl From<ChoiceOutcome> for Outcome {
        fn from(value: ChoiceOutcome) -> Self {
            match value {
                ChoiceOutcome::Continue => Outcome::Continue,
                ChoiceOutcome::Unchanged => Outcome::Unchanged,
                ChoiceOutcome::Changed => Outcome::Changed,
                ChoiceOutcome::Value => Outcome::Changed,
            }
        }
    }

    impl From<PopupOutcome> for ChoiceOutcome {
        fn from(value: PopupOutcome) -> Self {
            match value {
                PopupOutcome::Continue => ChoiceOutcome::Continue,
                PopupOutcome::Unchanged => ChoiceOutcome::Unchanged,
                PopupOutcome::Changed => ChoiceOutcome::Changed,
                PopupOutcome::Hide => ChoiceOutcome::Changed,
            }
        }
    }
}

pub mod core {
    #[derive(Debug, Default, Clone)]
    pub struct ChoiceCore<T>
    where
        T: PartialEq + Clone + Default,
    {
        /// Values.
        /// __read only__. renewed for each render.
        values: Vec<T>,
        /// Can return to default with a user interaction.
        default_value: Option<T>,
        /// Selected value, or a value set with set_value().
        /// There may be a value and still no selected index,
        /// if the values-vec is empty, or if the value is not in
        /// values.
        value: T,
        /// Index of value.
        selected: Option<usize>,
    }

    impl<T> ChoiceCore<T>
    where
        T: PartialEq + Clone + Default,
    {
        pub fn set_values(&mut self, values: Vec<T>) {
            self.values = values;
            // ensure integrity
            if self.values.is_empty() {
                self.selected = None;
            } else {
                self.selected = self.values.iter().position(|v| *v == self.value);
            }
        }

        /// List of values.
        pub fn values(&self) -> &[T] {
            &self.values
        }

        /// Set a default-value other than T::default()
        ///
        /// The starting value will still be T::default()
        /// after this. You must call clear() to use this
        /// default.
        pub fn set_default_value(&mut self, default_value: Option<T>) {
            self.default_value = default_value.clone();
        }

        /// A default value.
        pub fn default_value(&self) -> &Option<T> {
            &self.default_value
        }

        /// Selected item index.
        ///
        /// This may be None and there may still be a valid value.
        /// This can happen if a value is not in the value-list,
        /// or if the value-list is empty before the first render.
        ///
        /// Use of value() is preferred.
        pub fn selected(&self) -> Option<usize> {
            self.selected
        }

        /// Set the selected item by index.
        ///
        /// If the select-idx doesn't match the list of values,
        /// selected will be None. This may happen before the
        /// first render, while values is still empty.
        ///
        /// Use of set_value() is preferred.
        pub fn set_selected(&mut self, select: usize) -> bool {
            let old_sel = self.selected;
            if self.values.is_empty() {
                self.selected = None;
            } else {
                if let Some(value) = self.values.get(select) {
                    self.value = value.clone();
                    self.selected = Some(select);
                } else {
                    // don't change value
                    self.selected = None;
                }
            }
            old_sel != self.selected
        }

        /// Set the value for this Choice.
        ///
        /// The value will be retained even if it is not in
        /// the value-list. This can happen before the first
        /// render while the value-list is still empty.
        /// Or because a divergent value is set here.
        ///
        /// The starting value will be T::default().
        pub fn set_value(&mut self, value: T) -> bool {
            let old_value = self.value.clone();

            self.value = value;
            self.selected = self.values.iter().position(|v| *v == self.value);

            old_value != self.value
        }

        /// Return the selected value, or any value set with set_value()
        pub fn value(&self) -> T {
            self.value.clone()
        }

        pub fn is_empty(&self) -> bool {
            self.values.is_empty()
        }

        pub fn clear(&mut self) -> bool {
            let old_selected = self.selected;
            let old_value = self.value.clone();

            if let Some(default_value) = &self.default_value {
                self.value = default_value.clone();
            }

            self.selected = self.values.iter().position(|v| *v == self.value);

            old_selected != self.selected || old_value != self.value
        }
    }
}

impl Default for ChoiceStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            button: None,
            select: None,
            focus: None,
            block: None,
            popup: Default::default(),
            popup_len: None,
            behave_select: None,
            behave_close: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> Default for Choice<'_, T>
where
    T: PartialEq + Clone + Default,
{
    fn default() -> Self {
        Self {
            values: Default::default(),
            default_value: Default::default(),
            items: Default::default(),
            style: Default::default(),
            button_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            block: Default::default(),
            popup_len: Default::default(),
            popup_alignment: Alignment::Left,
            popup_placement: Placement::BelowOrAbove,
            popup: Default::default(),
            behave_select: Default::default(),
            behave_close: Default::default(),
        }
    }
}

impl<'a> Choice<'a, usize> {
    /// Add items with auto-generated values.
    #[inline]
    pub fn auto_items<V: Into<Line<'a>>>(self, items: impl IntoIterator<Item = V>) -> Self {
        {
            let mut values = self.values.borrow_mut();
            let mut itemz = self.items.borrow_mut();

            values.clear();
            itemz.clear();

            for (k, v) in items.into_iter().enumerate() {
                values.push(k);
                itemz.push(v.into());
            }
        }

        self
    }

    /// Add an item with an auto generated value.
    pub fn auto_item(self, item: impl Into<Line<'a>>) -> Self {
        let idx = self.values.borrow().len();
        self.values.borrow_mut().push(idx);
        self.items.borrow_mut().push(item.into());
        self
    }
}

impl<'a, T> Choice<'a, T>
where
    T: PartialEq + Clone + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Button text.
    #[inline]
    pub fn items<V: Into<Line<'a>>>(self, items: impl IntoIterator<Item = (T, V)>) -> Self {
        {
            let mut values = self.values.borrow_mut();
            let mut itemz = self.items.borrow_mut();

            values.clear();
            itemz.clear();

            for (k, v) in items.into_iter() {
                values.push(k);
                itemz.push(v.into());
            }
        }

        self
    }

    /// Add an item.
    pub fn item(self, value: T, item: impl Into<Line<'a>>) -> Self {
        self.values.borrow_mut().push(value);
        self.items.borrow_mut().push(item.into());
        self
    }

    /// Can return to default with user interaction.
    pub fn default_value(mut self, default: T) -> Self {
        self.default_value = Some(default);
        self
    }

    /// Combined styles.
    pub fn styles(mut self, styles: ChoiceStyle) -> Self {
        self.style = styles.style;
        if styles.button.is_some() {
            self.button_style = styles.button;
        }
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(select) = styles.behave_select {
            self.behave_select = select;
        }
        if let Some(close) = styles.behave_close {
            self.behave_close = close;
        }
        self.block = self.block.map(|v| v.style(self.style));
        if let Some(alignment) = styles.popup.alignment {
            self.popup_alignment = alignment;
        }
        if let Some(placement) = styles.popup.placement {
            self.popup_placement = placement;
        }
        if styles.popup_len.is_some() {
            self.popup_len = styles.popup_len;
        }
        self.popup = self.popup.styles(styles.popup);
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Style for the down button.
    pub fn button_style(mut self, style: Style) -> Self {
        self.button_style = Some(style);
        self
    }

    /// Selection in the list.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Focused style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Block for the main widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Alignment of the popup.
    ///
    /// __Default__
    /// Default is Left.
    pub fn popup_alignment(mut self, alignment: Alignment) -> Self {
        self.popup_alignment = alignment;
        self
    }

    /// Placement of the popup.
    ///
    /// __Default__
    /// Default is BelowOrAbove.
    pub fn popup_placement(mut self, placement: Placement) -> Self {
        self.popup_placement = placement;
        self
    }

    /// Outer boundary for the popup.
    pub fn popup_boundary(mut self, boundary: Rect) -> Self {
        self.popup = self.popup.boundary(boundary);
        self
    }

    /// Override the popup length.
    ///
    /// __Default__
    /// Defaults to the number of items or 5.
    pub fn popup_len(mut self, len: u16) -> Self {
        self.popup_len = Some(len);
        self
    }

    /// Base style for the popup.
    pub fn popup_style(mut self, style: Style) -> Self {
        self.popup = self.popup.style(style);
        self
    }

    /// Block for the popup.
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
        self
    }

    /// Scroll for the popup.
    pub fn popup_scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.popup = self.popup.v_scroll(scroll);
        self
    }

    /// Adds an extra offset to the widget area.
    ///
    /// This can be used to
    /// * place the widget under the mouse cursor.
    /// * align the widget not by the outer bounds but by
    ///   the text content.
    pub fn popup_offset(mut self, offset: (i16, i16)) -> Self {
        self.popup = self.popup.offset(offset);
        self
    }

    /// Sets only the x offset.
    /// See [offset](Self::popup_offset)
    pub fn popup_x_offset(mut self, offset: i16) -> Self {
        self.popup = self.popup.x_offset(offset);
        self
    }

    /// Sets only the y offset.
    /// See [offset](Self::popup_offset)
    pub fn popup_y_offset(mut self, offset: i16) -> Self {
        self.popup = self.popup.y_offset(offset);
        self
    }

    /// Sets the behaviour for selecting from the list.
    pub fn behave_select(mut self, select: ChoiceSelect) -> Self {
        self.behave_select = select;
        self
    }

    /// Sets the behaviour for closing the list.
    pub fn behave_close(mut self, close: ChoiceClose) -> Self {
        self.behave_close = close;
        self
    }

    /// Inherent width.
    pub fn width(&self) -> u16 {
        let w = self
            .items
            .borrow()
            .iter()
            .map(|v| v.width())
            .max()
            .unwrap_or_default();

        w as u16 + block_size(&self.block).width
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        1 + block_size(&self.block).height
    }

    /// Choice itself doesn't render.
    ///
    /// This builds the widgets from the parameters set for Choice.
    pub fn into_widgets(self) -> (ChoiceWidget<'a, T>, ChoicePopup<'a, T>) {
        (
            ChoiceWidget {
                values: self.values,
                default_value: self.default_value,
                items: self.items.clone(),
                style: self.style,
                button_style: self.button_style,
                focus_style: self.focus_style,
                block: self.block,
                len: self.popup_len,
                behave_select: self.behave_select,
                behave_close: self.behave_close,
                _phantom: Default::default(),
            },
            ChoicePopup {
                items: self.items.clone(),
                style: self.style,
                select_style: self.select_style,
                popup: self.popup,
                popup_alignment: self.popup_alignment,
                popup_placement: self.popup_placement,
                popup_len: self.popup_len,
                _phantom: Default::default(),
            },
        )
    }
}

impl<'a, T> ChoiceWidget<'a, T>
where
    T: PartialEq + Clone + Default,
{
    /// Inherent width.
    pub fn width(&self) -> u16 {
        let w = self
            .items
            .borrow()
            .iter()
            .map(|v| v.width())
            .max()
            .unwrap_or_default();

        w as u16 + block_size(&self.block).width
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        1 + block_size(&self.block).height
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a, T> StatefulWidgetRef for ChoiceWidget<'a, T>
where
    T: PartialEq + Clone + Default,
{
    type State = ChoiceState<T>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.core.set_values(self.values.borrow().clone());
        if let Some(default_value) = self.default_value.clone() {
            state.core.set_default_value(Some(default_value));
        }

        render_choice(self, area, buf, state);
    }
}

impl<T> StatefulWidget for ChoiceWidget<'_, T>
where
    T: PartialEq + Clone + Default,
{
    type State = ChoiceState<T>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.core.set_values(self.values.take());
        if let Some(default_value) = self.default_value.take() {
            state.core.set_default_value(Some(default_value));
        }

        render_choice(&self, area, buf, state);
    }
}

fn render_choice<T: PartialEq + Clone + Default>(
    widget: &ChoiceWidget<'_, T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ChoiceState<T>,
) {
    state.area = area;
    state.behave_select = widget.behave_select;
    state.behave_close = widget.behave_close;

    if !state.popup.is_active() {
        let len = widget
            .len
            .unwrap_or_else(|| min(5, widget.items.borrow().len()) as u16);
        state.popup.v_scroll.max_offset = widget.items.borrow().len().saturating_sub(len as usize);
        state.popup.v_scroll.page_len = len as usize;
        if let Some(selected) = state.core.selected() {
            state.popup.v_scroll.scroll_to_pos(selected);
        }
    }

    state.nav_char.clear();
    state.nav_char.extend(widget.items.borrow().iter().map(|v| {
        v.spans
            .first()
            .and_then(|v| v.content.as_ref().chars().next())
            .map_or(Vec::default(), |c| c.to_lowercase().collect::<Vec<_>>())
    }));

    let inner = widget.block.inner_if_some(area);

    state.item_area = Rect::new(
        inner.x,
        inner.y,
        inner.width.saturating_sub(3),
        inner.height,
    );
    state.button_area = Rect::new(
        inner.right().saturating_sub(min(3, inner.width)),
        inner.y,
        3,
        inner.height,
    );

    let style = widget.style;
    let focus_style = widget.focus_style.unwrap_or(revert_style(widget.style));

    if state.is_focused() {
        if widget.block.is_some() {
            widget.block.render(area, buf);
        } else {
            buf.set_style(inner, style);
        }
        buf.set_style(inner, focus_style);
    } else {
        if widget.block.is_some() {
            widget.block.render(area, buf);
        } else {
            buf.set_style(inner, style);
        }
        if let Some(button_style) = widget.button_style {
            buf.set_style(state.button_area, button_style);
        }
    }

    if let Some(selected) = state.core.selected() {
        if let Some(item) = widget.items.borrow().get(selected) {
            item.render(state.item_area, buf);
        }
    }

    let dy = if (state.button_area.height & 1) == 1 {
        state.button_area.height / 2
    } else {
        state.button_area.height.saturating_sub(1) / 2
    };
    let bc = if state.is_popup_active() {
        " ◆ "
    } else {
        " ▼ "
    };
    Span::from(bc).render(
        Rect::new(state.button_area.x, state.button_area.y + dy, 3, 1),
        buf,
    );
}

impl<T> ChoicePopup<'_, T>
where
    T: PartialEq + Clone + Default,
{
    /// Calculate the layout for the popup before rendering.
    /// Area is the area of the ChoiceWidget not the ChoicePopup.
    pub fn layout(&self, area: Rect, buf: &mut Buffer, state: &mut ChoiceState<T>) -> Rect {
        if state.popup.is_active() {
            let len = min(
                self.popup_len.unwrap_or(5),
                self.items.borrow().len() as u16,
            );
            let popup_len = len + self.popup.get_block_size().height;
            let pop_area = Rect::new(0, 0, area.width, popup_len);

            self.popup
                .ref_constraint(
                    self.popup_placement
                        .into_constraint(self.popup_alignment, area),
                )
                .layout(pop_area, buf)
        } else {
            Rect::default()
        }
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<T> StatefulWidgetRef for ChoicePopup<'_, T>
where
    T: PartialEq + Clone + Default,
{
    type State = ChoiceState<T>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(&self, area, buf, state);
    }
}

impl<T> StatefulWidget for ChoicePopup<'_, T>
where
    T: PartialEq + Clone + Default,
{
    type State = ChoiceState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(&self, area, buf, state);
    }
}

fn render_popup<T: PartialEq + Clone + Default>(
    widget: &ChoicePopup<'_, T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ChoiceState<T>,
) {
    if state.popup.is_active() {
        let len = min(
            widget.popup_len.unwrap_or(5),
            widget.items.borrow().len() as u16,
        );
        let popup_len = len + widget.popup.get_block_size().height;
        let pop_area = Rect::new(0, 0, area.width, popup_len);

        let popup_style = widget.popup.style;
        let select_style = widget.select_style.unwrap_or(revert_style(widget.style));

        widget
            .popup
            .ref_constraint(
                widget
                    .popup_placement
                    .into_constraint(widget.popup_alignment, area),
            )
            .render(pop_area, buf, &mut state.popup);

        let inner = state.popup.widget_area;

        state.popup.v_scroll.max_offset = widget
            .items
            .borrow()
            .len()
            .saturating_sub(inner.height as usize);
        state.popup.v_scroll.page_len = inner.height as usize;

        state.item_areas.clear();
        let mut row = inner.y;
        let mut idx = state.popup.v_scroll.offset;
        loop {
            if row >= inner.bottom() {
                break;
            }

            let item_area = Rect::new(inner.x, row, inner.width, 1);
            state.item_areas.push(item_area);

            if let Some(item) = widget.items.borrow().get(idx) {
                let style = if state.core.selected() == Some(idx) {
                    popup_style.patch(select_style)
                } else {
                    popup_style
                };

                buf.set_style(item_area, style);
                item.render(item_area, buf);
            } else {
                // noop?
            }

            row += 1;
            idx += 1;
        }
    } else {
        state.popup.clear_areas();
    }
}

impl<T> Clone for ChoiceState<T>
where
    T: PartialEq + Clone + Default,
{
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            nav_char: self.nav_char.clone(),
            item_area: self.item_area,
            button_area: self.button_area,
            item_areas: self.item_areas.clone(),
            core: self.core.clone(),
            popup: self.popup.clone(),
            behave_select: self.behave_select,
            behave_close: self.behave_close,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> Default for ChoiceState<T>
where
    T: PartialEq + Clone + Default,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            nav_char: Default::default(),
            item_area: Default::default(),
            button_area: Default::default(),
            item_areas: Default::default(),
            core: Default::default(),
            popup: Default::default(),
            behave_select: Default::default(),
            behave_close: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> HasFocus for ChoiceState<T>
where
    T: PartialEq + Clone + Default,
{
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget_with_flags(self.focus(), self.area(), 0, self.navigable());
        builder.widget_with_flags(self.focus(), self.popup.area, 1, Navigation::Mouse);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<T> RelocatableState for ChoiceState<T>
where
    T: PartialEq + Clone + Default,
{
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.item_area = relocate_area(self.item_area, shift, clip);
        self.button_area = relocate_area(self.button_area, shift, clip);
        relocate_areas(&mut self.item_areas, shift, clip);
        self.popup.relocate(shift, clip);
    }
}

impl<T> ChoiceState<T>
where
    T: PartialEq + Clone + Default,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Default::default()
        }
    }

    /// Popup is active?
    pub fn is_popup_active(&self) -> bool {
        self.popup.is_active()
    }

    /// Flip the popup state.
    pub fn flip_popup_active(&mut self) {
        self.popup.flip_active();
    }

    /// Show the popup.
    pub fn set_popup_active(&mut self, active: bool) -> bool {
        self.popup.set_active(active)
    }

    /// Set a default-value other than T::default()
    ///
    /// The starting value will still be T::default()
    /// after this. You must call clear() to use this
    /// default.
    ///
    /// This default will be overridden by a default set
    /// on the widget.
    pub fn set_default_value(&mut self, default_value: Option<T>) {
        self.core.set_default_value(default_value);
    }

    /// A default value.
    pub fn default_value(&self) -> &Option<T> {
        self.core.default_value()
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
    pub fn set_value(&mut self, value: T) -> bool {
        self.core.set_value(value)
    }

    /// Get the selected value.
    pub fn value(&self) -> T {
        self.core.value()
    }

    /// Select the default value or T::default.
    pub fn clear(&mut self) -> bool {
        self.core.clear()
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
        self.core.set_selected(select)
    }

    /// Returns the selected index or None if the
    /// value is not in the list or the list is empty.
    ///
    /// You can still get the value set with set_value() though.
    pub fn selected(&self) -> Option<usize> {
        self.core.selected()
    }

    /// Any items?
    pub fn is_empty(&self) -> bool {
        self.core.values().is_empty()
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        self.core.values().len()
    }

    /// Scroll offset for the item list.
    pub fn clear_offset(&mut self) {
        self.popup.v_scroll.set_offset(0);
    }

    /// Scroll offset for the item list.
    pub fn set_offset(&mut self, offset: usize) -> bool {
        self.popup.v_scroll.set_offset(offset)
    }

    /// Scroll offset for the item list.
    pub fn offset(&self) -> usize {
        self.popup.v_scroll.offset()
    }

    /// Scroll offset for the item list.
    pub fn max_offset(&self) -> usize {
        self.popup.v_scroll.max_offset()
    }

    /// Page length for the item list.
    pub fn page_len(&self) -> usize {
        self.popup.v_scroll.page_len()
    }

    /// Scroll unit for the item list.
    pub fn scroll_by(&self) -> usize {
        self.popup.v_scroll.scroll_by()
    }

    /// Scroll the item list to the selected value.
    pub fn scroll_to_selected(&mut self) -> bool {
        if let Some(selected) = self.core.selected() {
            self.popup.v_scroll.scroll_to_pos(selected)
        } else {
            false
        }
    }
}

impl<T> ChoiceState<T>
where
    T: PartialEq + Clone + Default,
{
    /// Select by first character.
    pub fn select_by_char(&mut self, c: char) -> bool {
        if self.nav_char.is_empty() {
            return false;
        }
        let c = c.to_lowercase().collect::<Vec<_>>();

        let (mut idx, end_loop) = if let Some(idx) = self.core.selected() {
            (idx + 1, idx)
        } else {
            if self.nav_char[0] == c {
                self.core.set_selected(0);
                return true;
            } else {
                (1, 0)
            }
        };
        loop {
            if idx >= self.nav_char.len() {
                idx = 0;
            }
            if idx == end_loop {
                break;
            }

            if self.nav_char[idx] == c {
                self.core.set_selected(idx);
                return true;
            }

            idx += 1;
        }
        false
    }

    /// Select by first character. Reverse direction
    pub fn reverse_select_by_char(&mut self, c: char) -> bool {
        if self.nav_char.is_empty() {
            return false;
        }
        let c = c.to_lowercase().collect::<Vec<_>>();

        let (mut idx, end_loop) = if let Some(idx) = self.core.selected() {
            if idx == 0 {
                (self.nav_char.len() - 1, 0)
            } else {
                (idx - 1, idx)
            }
        } else {
            if self.nav_char.last() == Some(&c) {
                self.core.set_selected(self.nav_char.len() - 1);
                return true;
            } else {
                (self.nav_char.len() - 1, 0)
            }
        };
        loop {
            if self.nav_char[idx] == c {
                self.core.set_selected(idx);
                return true;
            }

            if idx == end_loop {
                break;
            }

            if idx == 0 {
                idx = self.nav_char.len() - 1;
            } else {
                idx -= 1;
            }
        }
        false
    }

    /// Select at position
    pub fn move_to(&mut self, n: usize) -> ChoiceOutcome {
        let old_selected = self.selected();
        let r1 = self.popup.set_active(true);
        let r2 = self.select(n);
        let r3 = self.scroll_to_selected();
        if old_selected != self.selected() {
            ChoiceOutcome::Value
        } else if r1 || r2 || r3 {
            ChoiceOutcome::Changed
        } else {
            ChoiceOutcome::Continue
        }
    }

    /// Select next entry.
    pub fn move_down(&mut self, n: usize) -> ChoiceOutcome {
        if self.core.is_empty() {
            return ChoiceOutcome::Continue;
        }

        let old_selected = self.selected();
        let r1 = self.popup.set_active(true);
        let idx = if let Some(idx) = self.core.selected() {
            idx + n
        } else {
            n.saturating_sub(1)
        };
        let idx = idx.clamp(0, self.len() - 1);
        let r2 = self.core.set_selected(idx);
        let r3 = self.scroll_to_selected();

        if old_selected != self.selected() {
            ChoiceOutcome::Value
        } else if r1 || r2 || r3 {
            ChoiceOutcome::Changed
        } else {
            ChoiceOutcome::Continue
        }
    }

    /// Select prev entry.
    pub fn move_up(&mut self, n: usize) -> ChoiceOutcome {
        if self.core.is_empty() {
            return ChoiceOutcome::Continue;
        }

        let old_selected = self.selected();
        let r1 = self.popup.set_active(true);
        let idx = if let Some(idx) = self.core.selected() {
            idx.saturating_sub(n)
        } else {
            0
        };
        let idx = idx.clamp(0, self.len() - 1);
        let r2 = self.core.set_selected(idx);
        let r3 = self.scroll_to_selected();

        if old_selected != self.selected() {
            ChoiceOutcome::Value
        } else if r1 || r2 || r3 {
            ChoiceOutcome::Changed
        } else {
            ChoiceOutcome::Continue
        }
    }
}

impl<T: PartialEq + Clone + Default> HandleEvent<crossterm::event::Event, Popup, ChoiceOutcome>
    for ChoiceState<T>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> ChoiceOutcome {
        if self.lost_focus() {
            self.set_popup_active(false);
            // focus change triggers the repaint.
        }

        let r = if self.is_focused() {
            match event {
                ct_event!(key press ' ') | ct_event!(keycode press Enter) => {
                    self.flip_popup_active();
                    ChoiceOutcome::Changed
                }
                ct_event!(keycode press Esc) => {
                    if self.set_popup_active(false) {
                        ChoiceOutcome::Changed
                    } else {
                        ChoiceOutcome::Continue
                    }
                }
                ct_event!(key press c) => {
                    if self.select_by_char(*c) {
                        self.scroll_to_selected();
                        ChoiceOutcome::Value
                    } else {
                        ChoiceOutcome::Continue
                    }
                }
                ct_event!(key press SHIFT-c) => {
                    if self.reverse_select_by_char(*c) {
                        self.scroll_to_selected();
                        ChoiceOutcome::Value
                    } else {
                        ChoiceOutcome::Continue
                    }
                }
                ct_event!(keycode press Delete) | ct_event!(keycode press Backspace) => {
                    if self.clear() {
                        ChoiceOutcome::Value
                    } else {
                        ChoiceOutcome::Continue
                    }
                }
                ct_event!(keycode press Down) => self.move_down(1),
                ct_event!(keycode press Up) => self.move_up(1),
                ct_event!(keycode press PageUp) => self.move_up(self.page_len()),
                ct_event!(keycode press PageDown) => self.move_down(self.page_len()),
                ct_event!(keycode press Home) => self.move_to(0),
                ct_event!(keycode press End) => self.move_to(self.len().saturating_sub(1)),
                _ => ChoiceOutcome::Continue,
            }
        } else {
            ChoiceOutcome::Continue
        };

        if !r.is_consumed() {
            self.handle(event, MouseOnly)
        } else {
            r
        }
    }
}

impl<T: PartialEq + Clone + Default> HandleEvent<crossterm::event::Event, MouseOnly, ChoiceOutcome>
    for ChoiceState<T>
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> ChoiceOutcome {
        let r0 = handle_mouse(self, event);
        let r1 = handle_select(self, event);
        let r2 = handle_close(self, event);
        let mut r = max(r0, max(r1, r2));

        r = r.or_else(|| mouse_trap(event, self.popup.area).into());

        self.popup.active.set_lost(false);
        self.popup.active.set_gained(false);
        r
    }
}

fn handle_mouse<T: PartialEq + Clone + Default>(
    state: &mut ChoiceState<T>,
    event: &crossterm::event::Event,
) -> ChoiceOutcome {
    match event {
        ct_event!(mouse down Left for x,y)
            if state.item_area.contains((*x, *y).into())
                || state.button_area.contains((*x, *y).into()) =>
        {
            if !state.gained_focus() && !state.popup.active.lost() {
                state.flip_popup_active();
                ChoiceOutcome::Changed
            } else {
                // hide is down by self.popup.handle() as this click
                // is outside the popup area!!
                ChoiceOutcome::Continue
            }
        }
        ct_event!(mouse down Left for x,y)
        | ct_event!(mouse down Right for x,y)
        | ct_event!(mouse down Middle for x,y)
            if !state.item_area.contains((*x, *y).into())
                && !state.button_area.contains((*x, *y).into()) =>
        {
            match state.popup.handle(event, Popup) {
                PopupOutcome::Hide => {
                    state.set_popup_active(false);
                    ChoiceOutcome::Changed
                }
                r => r.into(),
            }
        }
        _ => ChoiceOutcome::Continue,
    }
}

fn handle_select<T: PartialEq + Clone + Default>(
    state: &mut ChoiceState<T>,
    event: &crossterm::event::Event,
) -> ChoiceOutcome {
    match state.behave_select {
        ChoiceSelect::MouseScroll => {
            let mut sas = ScrollAreaState::new()
                .area(state.popup.area)
                .v_scroll(&mut state.popup.v_scroll);
            let mut r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(n) => state.move_up(n),
                ScrollOutcome::Down(n) => state.move_down(n),
                ScrollOutcome::VPos(n) => state.move_to(n),
                _ => ChoiceOutcome::Continue,
            };

            r = r.or_else(|| match event {
                ct_event!(mouse down Left for x,y)
                    if state.popup.widget_area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                ct_event!(mouse drag Left for x,y)
                    if state.popup.widget_area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                _ => ChoiceOutcome::Continue,
            });
            r
        }
        ChoiceSelect::MouseMove => {
            // effect: move the content below the mouse and keep visible selection.
            let mut r = if let Some(selected) = state.core.selected() {
                let rel_sel = selected.saturating_sub(state.offset());
                let mut sas = ScrollAreaState::new()
                    .area(state.popup.area)
                    .v_scroll(&mut state.popup.v_scroll);
                match sas.handle(event, MouseOnly) {
                    ScrollOutcome::Up(n) => {
                        state.popup.v_scroll.scroll_up(n);
                        if state.select(state.offset() + rel_sel) {
                            ChoiceOutcome::Value
                        } else {
                            ChoiceOutcome::Unchanged
                        }
                    }
                    ScrollOutcome::Down(n) => {
                        state.popup.v_scroll.scroll_down(n);
                        if state.select(state.offset() + rel_sel) {
                            ChoiceOutcome::Value
                        } else {
                            ChoiceOutcome::Unchanged
                        }
                    }
                    ScrollOutcome::VPos(n) => {
                        if state.popup.v_scroll.set_offset(n) {
                            ChoiceOutcome::Value
                        } else {
                            ChoiceOutcome::Unchanged
                        }
                    }
                    _ => ChoiceOutcome::Continue,
                }
            } else {
                ChoiceOutcome::Continue
            };

            r = r.or_else(|| match event {
                ct_event!(mouse moved for x,y)
                    if state.popup.widget_area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                _ => ChoiceOutcome::Continue,
            });
            r
        }
        ChoiceSelect::MouseClick => {
            // effect: move the content below the mouse and keep visible selection.
            let mut sas = ScrollAreaState::new()
                .area(state.popup.area)
                .v_scroll(&mut state.popup.v_scroll);
            let mut r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(n) => {
                    if state.popup.v_scroll.scroll_up(n) {
                        ChoiceOutcome::Changed
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                ScrollOutcome::Down(n) => {
                    if state.popup.v_scroll.scroll_down(n) {
                        ChoiceOutcome::Changed
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                ScrollOutcome::VPos(n) => {
                    if state.popup.v_scroll.set_offset(n) {
                        ChoiceOutcome::Changed
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                _ => ChoiceOutcome::Continue,
            };

            r = r.or_else(|| match event {
                ct_event!(mouse down Left for x,y)
                    if state.popup.widget_area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                ct_event!(mouse drag Left for x,y)
                    if state.popup.widget_area.contains((*x, *y).into()) =>
                {
                    if let Some(n) = item_at(&state.item_areas, *x, *y) {
                        state.move_to(state.offset() + n)
                    } else {
                        ChoiceOutcome::Unchanged
                    }
                }
                _ => ChoiceOutcome::Continue,
            });
            r
        }
    }
}

fn handle_close<T: PartialEq + Clone + Default>(
    state: &mut ChoiceState<T>,
    event: &crossterm::event::Event,
) -> ChoiceOutcome {
    match state.behave_close {
        ChoiceClose::SingleClick => match event {
            ct_event!(mouse down Left for x,y)
                if state.popup.widget_area.contains((*x, *y).into()) =>
            {
                if let Some(n) = item_at(&state.item_areas, *x, *y) {
                    let r = state.move_to(state.offset() + n);
                    let s = if state.set_popup_active(false) {
                        ChoiceOutcome::Changed
                    } else {
                        ChoiceOutcome::Unchanged
                    };
                    max(r, s)
                } else {
                    ChoiceOutcome::Unchanged
                }
            }
            _ => ChoiceOutcome::Continue,
        },
        ChoiceClose::DoubleClick => match event {
            ct_event!(mouse any for m) if state.mouse.doubleclick(state.popup.widget_area, m) => {
                if let Some(n) = item_at(&state.item_areas, m.column, m.row) {
                    let r = state.move_to(state.offset() + n);
                    let s = if state.set_popup_active(false) {
                        ChoiceOutcome::Changed
                    } else {
                        ChoiceOutcome::Unchanged
                    };
                    max(r, s)
                } else {
                    ChoiceOutcome::Unchanged
                }
            }
            _ => ChoiceOutcome::Continue,
        },
    }
}

/// Handle events for the popup.
/// Call before other handlers to deal with intersections
/// with other widgets.
pub fn handle_popup<T: PartialEq + Clone + Default>(
    state: &mut ChoiceState<T>,
    focus: bool,
    event: &crossterm::event::Event,
) -> ChoiceOutcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events<T: PartialEq + Clone + Default>(
    state: &mut ChoiceState<T>,
    event: &crossterm::event::Event,
) -> ChoiceOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
