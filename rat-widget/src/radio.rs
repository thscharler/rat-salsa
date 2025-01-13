use crate::_private::NonExhaustive;
use crate::choice::core::ChoiceCore;
use crate::event::RadioOutcome;
use crate::util::{block_size, fill_buf_area, revert_style, union_non_empty};
use rat_event::util::{item_at, MouseFlags};
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, relocate_areas, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Direction, Rect, Size};
use ratatui::prelude::{BlockExt, StatefulWidget};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Widget};
use std::cmp::max;
use unicode_segmentation::UnicodeSegmentation;

/// Radio style.
///
/// This is used, if you don't provide your own layout constraints.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RadioLayout {
    /// Stacked one item after the other.
    #[default]
    Stacked,
    /// Equally spaced items.
    Spaced,
}

/// Horizontally aligned radio buttons.
#[derive(Debug, Clone)]
pub struct Radio<'a, T>
where
    T: PartialEq + Clone + Default,
{
    values: Vec<T>,
    default_value: Option<T>,
    items: Vec<Text<'a>>,
    direction: Direction,
    layout: RadioLayout,

    true_str: Span<'a>,
    false_str: Span<'a>,
    continue_str: Span<'a>,

    style: Style,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,
}

/// Composite style.
#[derive(Debug, Clone)]
pub struct RadioStyle {
    /// Radio layout
    pub layout: Option<RadioLayout>,

    /// Base style.
    pub style: Style,
    /// Selected style.
    pub select: Option<Style>,
    /// Focused style
    pub focus: Option<Style>,
    /// Border
    pub block: Option<Block<'static>>,

    /// Display text for 'true'
    pub true_str: Option<Span<'static>>,
    /// Display text for 'false'
    pub false_str: Option<Span<'static>>,
    /// Continue text.
    pub continue_str: Option<Span<'static>>,

    pub non_exhaustive: NonExhaustive,
}

/// State
#[derive(Debug)]
pub struct RadioState<T = usize>
where
    T: PartialEq + Clone + Default,
{
    /// Complete area
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Area inside the block.
    /// __read only__. renewed for each render.
    pub inner: Rect,

    /// Area for the focus marker.
    /// __read only__. renewed for each render.
    pub marker_area: Rect,
    /// Area for a continue marker.
    /// This is displayed if not all items can be displayed.
    pub continue_area: Rect,
    /// __read only__. renewed for each render.
    /// Area of the check marks.
    /// __read only__. renewed for each render.
    pub check_areas: Vec<Rect>,
    /// Area for the texts.
    /// __read only__. renewed for each render.
    pub text_areas: Vec<Rect>,

    /// Core
    pub core: ChoiceCore<T>,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Mouse helper
    /// __read+write__
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result value for event-handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum RadioOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
        /// An item has been selected.
        Value,
    }

    impl ConsumedEvent for RadioOutcome {
        fn is_consumed(&self) -> bool {
            *self != RadioOutcome::Continue
        }
    }

    impl From<RadioOutcome> for Outcome {
        fn from(value: RadioOutcome) -> Self {
            match value {
                RadioOutcome::Continue => Outcome::Continue,
                RadioOutcome::Unchanged => Outcome::Unchanged,
                RadioOutcome::Changed => Outcome::Changed,
                RadioOutcome::Value => Outcome::Changed,
            }
        }
    }
}

impl Default for RadioStyle {
    fn default() -> Self {
        Self {
            layout: None,
            style: Default::default(),
            select: None,
            focus: None,
            block: Default::default(),
            true_str: None,
            false_str: None,
            continue_str: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> Default for Radio<'_, T>
where
    T: PartialEq + Clone + Default,
{
    fn default() -> Self {
        Self {
            values: Default::default(),
            items: Default::default(),
            direction: Default::default(),
            layout: Default::default(),
            default_value: Default::default(),
            true_str: Span::from("\u{2B24}"),
            false_str: Span::from("\u{25EF}"),
            continue_str: Span::from("...").on_yellow(),
            style: Default::default(),
            select_style: None,
            focus_style: None,
            block: None,
        }
    }
}

impl<'a> Radio<'a, usize> {
    /// Add items with auto-generated values.
    #[inline]
    pub fn auto_items<V: Into<Text<'a>>>(mut self, items: impl IntoIterator<Item = V>) -> Self {
        {
            self.values.clear();
            self.items.clear();

            for (k, v) in items.into_iter().enumerate() {
                self.values.push(k);
                self.items.push(v.into());
            }
        }

        self
    }

    /// Add an item with an auto generated value.
    pub fn auto_item(mut self, item: impl Into<Text<'a>>) -> Self {
        let idx = self.values.len();
        self.values.push(idx);
        self.items.push(item.into());
        self
    }
}

impl<'a, T> Radio<'a, T>
where
    T: PartialEq + Clone + Default,
{
    /// New.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set all styles.
    pub fn styles(mut self, styles: RadioStyle) -> Self {
        self.style = styles.style;
        if let Some(layout) = styles.layout {
            self.layout = layout;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.select.is_some() {
            self.select_style = styles.focus;
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        if let Some(true_str) = styles.true_str {
            self.true_str = true_str;
        }
        if let Some(false_str) = styles.false_str {
            self.false_str = false_str;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Set the base-style.
    #[inline]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Style when selected.
    #[inline]
    pub fn select_style(mut self, style: impl Into<Style>) -> Self {
        self.select_style = Some(style.into());
        self
    }

    /// Style when focused.
    #[inline]
    pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
        self.focus_style = Some(style.into());
        self
    }

    /// Radio direction
    #[inline]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Layout type, stacked or evenly spaced.
    #[inline]
    pub fn layout(mut self, layout: RadioLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Button text.
    #[inline]
    pub fn items<V: Into<Text<'a>>>(mut self, items: impl IntoIterator<Item = (T, V)>) -> Self {
        {
            self.values.clear();
            self.items.clear();

            for (k, v) in items.into_iter() {
                self.values.push(k);
                self.items.push(v.into());
            }
        }

        self
    }

    /// Add an item.
    pub fn item(mut self, value: T, item: impl Into<Text<'a>>) -> Self {
        self.values.push(value);
        self.items.push(item.into());
        self
    }

    /// Can return to default with user interaction.
    pub fn default_value(mut self, default: T) -> Self {
        self.default_value = Some(default);
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Text for true
    pub fn true_str(mut self, str: Span<'a>) -> Self {
        self.true_str = str;
        self
    }

    /// Text for false
    pub fn false_str(mut self, str: Span<'a>) -> Self {
        self.false_str = str;
        self
    }

    /// Inherent size
    pub fn size(&self) -> Size {
        if self.direction == Direction::Horizontal {
            self.horizontal_size()
        } else {
            self.vertical_size()
        }
    }

    /// Inherent width.
    pub fn width(&self) -> u16 {
        self.size().width
    }

    /// Inherent height.
    pub fn height(&self) -> u16 {
        self.size().height
    }
}

impl<T> Radio<'_, T>
where
    T: PartialEq + Clone + Default,
{
    /// Length of the check
    fn check_len(&self) -> u16 {
        max(
            self.true_str.content.graphemes(true).count(),
            self.false_str.content.graphemes(true).count(),
        ) as u16
    }

    fn horizontal_size(&self) -> Size {
        let block_size = block_size(&self.block);
        let check_len = self.check_len();
        let marker_len = 2;

        if self.layout == RadioLayout::Spaced {
            let (max_width, max_height) = self
                .items
                .iter()
                .map(|v| (v.width() as u16, v.height() as u16))
                .max()
                .unwrap_or_default();
            let n = self.items.len() as u16;
            let spacing = n.saturating_sub(1);

            Size::new(
                marker_len + n * (check_len + 1 + max_width) + spacing + block_size.width,
                max_height + block_size.height,
            )
        } else {
            let sum_width = self
                .items //
                .iter()
                .map(|v| v.width() as u16)
                .sum::<u16>();
            let max_height = self
                .items
                .iter()
                .map(|v| v.height() as u16)
                .max()
                .unwrap_or_default();

            let n = self.items.len() as u16;
            let spacing = n.saturating_sub(1);

            Size::new(
                marker_len + n * (check_len + 1) + sum_width + spacing + block_size.width,
                max_height + block_size.height,
            )
        }
    }

    fn vertical_size(&self) -> Size {
        let block_size = block_size(&self.block);
        let check_len = self.check_len();
        let marker_len = 2;

        if self.layout == RadioLayout::Spaced {
            let (max_width, max_height) = self
                .items
                .iter()
                .map(|v| (v.width() as u16, v.height() as u16))
                .max()
                .unwrap_or_default();

            let n = self.items.len() as u16;

            Size::new(
                marker_len + check_len + 1 + max_width + block_size.width,
                n * max_height + block_size.width,
            )
        } else {
            let max_width = self
                .items
                .iter()
                .map(|v| v.width() as u16)
                .max()
                .unwrap_or_default();

            let sum_height = self
                .items //
                .iter()
                .map(|v| v.height() as u16)
                .sum::<u16>();

            Size::new(
                marker_len + check_len + 1 + max_width + block_size.width,
                sum_height + block_size.height,
            )
        }
    }

    fn horizontal_spaced_layout(&self, area: Rect, state: &mut RadioState<T>) {
        state.inner = self.block.inner_if_some(area);

        let check_len = self.check_len();
        let continue_len = self.continue_str.width() as u16;
        let n = self.items.len() as u16;

        let text_width = max(
            7,
            (state.inner.width.saturating_sub(n * check_len) / n).saturating_sub(1),
        );
        let item_width = text_width + check_len + 1;

        state.continue_area = Rect::new(
            state.inner.right().saturating_sub(continue_len), //
            state.inner.y,
            continue_len,
            1,
        )
        .intersection(state.inner);

        state.marker_area = Rect::new(
            state.inner.x, //
            state.inner.y,
            1,
            state.inner.height,
        )
        .intersection(state.inner);

        state.check_areas.clear();
        state.text_areas.clear();

        let mut need_continue = false;
        for (i, item) in self.items.iter().enumerate() {
            let i = i as u16;

            state.check_areas.push(
                Rect::new(
                    state.inner.x + 2 + (i * item_width),
                    state.inner.y,
                    check_len,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            state.text_areas.push(
                Rect::new(
                    state.inner.x + 2 + (i * item_width) + check_len + 1,
                    state.inner.y,
                    item.width() as u16,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            need_continue = state.text_areas.last().expect("area").is_empty()
        }

        if !need_continue {
            state.continue_area = Rect::new(state.inner.x, state.inner.y, 0, 0);
        }
    }

    fn horizontal_stack_layout(&self, area: Rect, state: &mut RadioState<T>) {
        state.inner = self.block.inner_if_some(area);

        let check_len = self.check_len();
        let continue_len = self.continue_str.width() as u16;

        state.check_areas.clear();
        state.text_areas.clear();

        let mut x = state.inner.x;

        state.continue_area = Rect::new(
            state.inner.right().saturating_sub(continue_len), //
            state.inner.y,
            continue_len,
            1,
        )
        .intersection(state.inner);

        state.marker_area = Rect::new(
            x, //
            state.inner.y,
            1,
            state.inner.height,
        )
        .intersection(state.inner);
        x += 2;

        let mut need_continue = false;
        for item in self.items.iter() {
            state.check_areas.push(
                Rect::new(
                    x, //
                    state.inner.y,
                    check_len,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            x += check_len + 1;

            state.text_areas.push(
                Rect::new(
                    x, //
                    state.inner.y,
                    item.width() as u16,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            x += item.width() as u16 + 1;

            need_continue = state.text_areas.last().expect("area").is_empty()
        }

        if !need_continue {
            state.continue_area = Rect::new(state.inner.x, state.inner.y, 0, 0);
        }
    }

    fn vertical_spaced_layout(&self, area: Rect, state: &mut RadioState<T>) {
        state.inner = self.block.inner_if_some(area);

        let check_len = self.check_len();
        let n = self.items.len() as u16;

        let text_height = max(1, state.inner.height / n);

        state.continue_area = Rect::new(
            state.inner.x + 2,
            state.inner.bottom().saturating_sub(1),
            state.inner.width.saturating_sub(2),
            1,
        )
        .intersection(state.inner);

        state.marker_area = Rect::new(
            state.inner.x, //
            state.inner.y,
            1,
            state.inner.height,
        )
        .intersection(state.inner);

        state.check_areas.clear();
        state.text_areas.clear();

        let mut need_continue = false;
        for (i, item) in self.items.iter().enumerate() {
            let i = i as u16;

            state.check_areas.push(
                Rect::new(
                    state.inner.x + 2,
                    state.inner.y + (i * text_height),
                    check_len,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            state.text_areas.push(
                Rect::new(
                    state.inner.x + 2 + check_len + 1,
                    state.inner.y + (i * text_height),
                    item.width() as u16,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            need_continue = state.text_areas.last().expect("area").is_empty()
        }

        if !need_continue {
            state.continue_area = Rect::new(state.inner.x, state.inner.y, 0, 0);
        }
    }

    fn vertical_stack_layout(&self, area: Rect, state: &mut RadioState<T>) {
        state.inner = self.block.inner_if_some(area);

        let check_len = self.check_len();

        state.continue_area = Rect::new(
            state.inner.x + 2,
            state.inner.bottom().saturating_sub(1),
            state.inner.width.saturating_sub(2),
            1,
        )
        .intersection(state.inner);

        state.marker_area = Rect::new(
            state.inner.x, //
            state.inner.y,
            1,
            state.inner.height,
        )
        .intersection(state.inner);

        state.check_areas.clear();
        state.text_areas.clear();

        let mut need_continue = false;
        let mut y = state.inner.y;
        for item in self.items.iter() {
            state.check_areas.push(
                Rect::new(
                    state.inner.x + 2, //
                    y,
                    check_len,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            state.text_areas.push(
                Rect::new(
                    state.inner.x + 2 + check_len + 1,
                    y,
                    item.width() as u16,
                    item.height() as u16,
                )
                .intersection(state.inner),
            );

            y += item.height() as u16;

            need_continue = state.text_areas.last().expect("area").is_empty()
        }

        if !need_continue {
            state.continue_area = Rect::new(state.inner.x, state.inner.y, 0, 0);
        }
    }
}

impl<T> StatefulWidget for Radio<'_, T>
where
    T: PartialEq + Clone + Default,
{
    type State = RadioState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        assert!(!self.items.is_empty());

        state.area = area;

        match (self.direction, self.layout) {
            (Direction::Horizontal, RadioLayout::Stacked) => {
                self.horizontal_stack_layout(area, state);
            }
            (Direction::Horizontal, RadioLayout::Spaced) => {
                self.horizontal_spaced_layout(area, state);
            }
            (Direction::Vertical, RadioLayout::Stacked) => {
                self.vertical_stack_layout(area, state);
            }
            (Direction::Vertical, RadioLayout::Spaced) => {
                self.vertical_spaced_layout(area, state);
            }
        }

        state.core.set_values(self.values);
        if let Some(default_value) = self.default_value {
            state.core.set_default_value(default_value);
        }

        let focus_style = if let Some(focus_style) = self.focus_style {
            focus_style
        } else {
            revert_style(self.style)
        };
        let select_style = if let Some(select_style) = self.select_style {
            select_style
        } else {
            self.style
        };

        if self.block.is_some() {
            self.block.render(area, buf);
        } else {
            buf.set_style(state.area, self.style);
        }

        if state.is_focused() {
            buf.set_style(state.marker_area, focus_style);
        }

        for (i, item) in self.items.iter().enumerate() {
            if Some(i) == state.core.selected() {
                buf.set_style(
                    union_non_empty(state.check_areas[i], state.text_areas[i]),
                    if state.is_focused() {
                        focus_style
                    } else {
                        select_style
                    },
                );
                (&self.true_str).render(state.check_areas[i], buf);
            } else {
                (&self.false_str).render(state.check_areas[i], buf);
            }
            item.render(state.text_areas[i], buf);
        }

        if !state.continue_area.is_empty() {
            fill_buf_area(buf, state.continue_area, " ", self.style);
            self.continue_str.render(state.continue_area, buf);
        }
    }
}

impl<T> Clone for RadioState<T>
where
    T: PartialEq + Clone + Default,
{
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            marker_area: self.marker_area,
            continue_area: self.continue_area,
            check_areas: self.check_areas.clone(),
            text_areas: self.text_areas.clone(),
            core: self.core.clone(),
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> Default for RadioState<T>
where
    T: PartialEq + Clone + Default,
{
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            marker_area: Default::default(),
            continue_area: Default::default(),
            check_areas: Default::default(),
            text_areas: Default::default(),
            core: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> HasFocus for RadioState<T>
where
    T: PartialEq + Clone + Default,
{
    fn build(&self, builder: &mut FocusBuilder) {
        builder.append_leaf(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<T> RelocatableState for RadioState<T>
where
    T: PartialEq + Clone + Default,
{
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
        relocate_areas(self.check_areas.as_mut_slice(), shift, clip);
        relocate_areas(self.text_areas.as_mut_slice(), shift, clip);
    }
}

impl<T> RadioState<T>
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

    pub fn is_empty(&self) -> bool {
        self.text_areas.is_empty()
    }

    pub fn len(&self) -> usize {
        self.text_areas.len()
    }

    /// Returns the selected index or None if the
    /// value is not in the list or the list is empty.
    ///
    /// You can still get the value set with set_value() though.
    pub fn selected(&self) -> Option<usize> {
        self.core.selected()
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

    /// Set the default value.
    pub fn clear(&mut self) -> bool {
        self.core.clear()
    }

    /// Select the next item.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.core.values().is_empty() {
            false // noop
        } else {
            if let Some(selected) = self.core.selected() {
                if selected + 1 >= self.core.values().len() {
                    self.core.set_selected(0)
                } else {
                    self.core.set_selected(selected + 1)
                }
            } else {
                self.core.set_selected(0)
            }
        }
    }

    /// Select the previous item.
    pub fn prev(&mut self) -> bool {
        if self.core.values().is_empty() {
            false // noop
        } else {
            if let Some(selected) = self.core.selected() {
                if selected == 0 {
                    self.core.set_selected(self.core.values().len() - 1)
                } else {
                    self.core.set_selected(selected - 1)
                }
            } else {
                self.core.set_selected(self.core.values().len() - 1)
            }
        }
    }

    /// Set the given value.
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

    /// Get the selected value or None if no value
    /// is selected or there are no options.
    pub fn value(&self) -> T {
        self.core.value()
    }
}

impl<T> HandleEvent<crossterm::event::Event, Regular, RadioOutcome> for RadioState<T>
where
    T: PartialEq + Clone + Default,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> RadioOutcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => {
                    if self.prev() {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Right) => {
                    if self.next() {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Up) => {
                    if self.prev() {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Down) => {
                    if self.next() {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Home) => {
                    if self.select(0) {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                ct_event!(keycode press End) => {
                    if !self.is_empty() {
                        if self.select(self.len() - 1) {
                            RadioOutcome::Value
                        } else {
                            RadioOutcome::Unchanged
                        }
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Delete) | ct_event!(keycode press Backspace) => {
                    if self.clear() {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                }
                _ => RadioOutcome::Continue,
            }
        } else {
            RadioOutcome::Continue
        };

        if r == RadioOutcome::Continue {
            HandleEvent::handle(self, event, MouseOnly)
        } else {
            r
        }
    }
}

impl<T> HandleEvent<crossterm::event::Event, MouseOnly, RadioOutcome> for RadioState<T>
where
    T: PartialEq + Clone + Default,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> RadioOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.drag(self.area, m) => {
                if let Some(sel) = item_at(self.text_areas.as_slice(), m.column, m.row)
                    .or_else(|| item_at(self.check_areas.as_slice(), m.column, m.row))
                {
                    if self.select(sel) {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                } else {
                    RadioOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for x,y) if self.area.contains((*x, *y).into()) => {
                if let Some(sel) = item_at(self.text_areas.as_slice(), *x, *y)
                    .or_else(|| item_at(self.check_areas.as_slice(), *x, *y))
                {
                    if self.select(sel) {
                        RadioOutcome::Value
                    } else {
                        RadioOutcome::Unchanged
                    }
                } else {
                    RadioOutcome::Unchanged
                }
            }
            _ => RadioOutcome::Continue,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events<T: PartialEq + Clone + Default>(
    state: &mut RadioState<T>,
    focus: bool,
    event: &crossterm::event::Event,
) -> RadioOutcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events<T: PartialEq + Clone + Default>(
    state: &mut RadioState<T>,
    event: &crossterm::event::Event,
) -> RadioOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
