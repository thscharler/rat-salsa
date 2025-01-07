//!
//! Slider widget.
//!
//! ```rust no_run
//! use rat_widget::slider::{Slider, SliderState};
//! # use ratatui::layout::Rect;
//! # use ratatui::prelude::*;
//! #
//! # let slider_area = Rect::ZERO;
//! # let mut buf = Buffer::default();
//!
//! let mut state = SliderState::<u8>::new_range((0,255), 1);
//! state.set_value(42);
//!
//! Slider::new().render(slider_area, &mut buf, &mut state);
//!
//! ```
//!

use crate::_private::NonExhaustive;
use crate::range_op::RangeOp;
use crate::util::revert_style;
use map_range_int::MapRange;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{relocate_area, RelocatableState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Direction, Position, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, Widget};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use unicode_width::UnicodeWidthStr;

/// Slider widget for a type T.
///
/// T has to implement [RangeOp] and [MapRange] to and from u16.
///
#[derive(Debug, Clone)]
pub struct Slider<'a, T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    style: Style,
    bounds_style: Option<Style>,
    knob_style: Option<Style>,
    focus_style: Option<Style>,

    direction: Direction,

    range: Option<(T, T)>,
    step: Option<<T as RangeOp>::Step>,
    long_step: Option<<T as RangeOp>::Step>,

    text_align: Alignment,
    lower_bound: Option<Cow<'a, str>>,
    upper_bound: Option<Cow<'a, str>>,

    track_char: Option<Cow<'a, str>>,

    horizontal_knob: Option<Cow<'a, str>>,
    vertical_knob: Option<Cow<'a, str>>,

    block: Option<Block<'a>>,

    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct SliderStyle {
    /// Base style.
    pub style: Style,
    /// Style for the upper/lower bounds text.
    pub bounds: Option<Style>,
    /// Style for the knob.
    pub knob: Option<Style>,
    /// Style when focused.
    pub focus: Option<Style>,

    /// Alignment for all text.
    pub text_align: Option<Alignment>,
    /// Text string for the lower bound. Can contain newlines.
    pub lower_bound: Option<&'static str>,
    /// Text string for the upper bound. Can contain newlines.
    pub upper_bound: Option<&'static str>,

    /// Fill char for the track.
    pub track_char: Option<&'static str>,

    /// Text for the knob in vertical mode.
    pub vertical_knob: Option<&'static str>,
    /// Text for the knob in horizontal mode.
    pub horizontal_knob: Option<&'static str>,

    /// Border
    pub block: Option<Block<'static>>,

    pub non_exhaustive: NonExhaustive,
}

/// State.
pub struct SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    /// Complete area
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Area inside the block without padding due to alignment.
    /// __read only__. renewed for each render.
    pub inner: Rect,
    /// Lower bounds area.
    /// __read only__. renewed for each render.
    pub lower_bound: Rect,
    /// Upper bounds area.
    /// __read only__. renewed for each render.
    pub upper_bound: Rect,
    /// Track char.
    /// __read only__. renewed for each render.
    pub track: Rect,
    /// Knob text
    /// __read only__. renewed for each render.
    pub knob: Rect,
    /// Length of the track without the knob
    pub scale_len: u16,
    /// Direction
    /// __read only__. renewed for each render.
    pub direction: Direction,

    /// Value range
    pub range: (T, T),
    /// Minor step.
    pub step: <T as RangeOp>::Step,
    /// Major step.
    pub long_step: Option<<T as RangeOp>::Step>,

    /// Value
    pub value: T,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Mouse helper
    /// __read+write__
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            bounds: None,
            knob: None,
            focus: None,
            text_align: None,
            lower_bound: None,
            upper_bound: None,
            track_char: None,
            vertical_knob: None,
            horizontal_knob: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<T> Default for Slider<'_, T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    fn default() -> Self {
        Self {
            style: Default::default(),
            bounds_style: None,
            knob_style: None,
            focus_style: None,
            direction: Direction::Horizontal,
            range: None,
            step: None,
            long_step: None,
            text_align: Alignment::Left,
            lower_bound: None,
            upper_bound: None,
            track_char: None,
            horizontal_knob: None,
            vertical_knob: None,
            block: None,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T> Slider<'a, T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Direction for the slider.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Overrides the range of the slider.
    pub fn range(mut self, range: (T, T)) -> Self {
        self.range = Some(range);
        self
    }

    /// First step size.
    pub fn step(mut self, step: <T as RangeOp>::Step) -> Self {
        self.step = Some(step);
        self
    }

    /// Second step size.
    pub fn long_step(mut self, step: <T as RangeOp>::Step) -> Self {
        self.long_step = Some(step);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: SliderStyle) -> Self {
        self.style = styles.style;
        if styles.bounds.is_some() {
            self.bounds_style = styles.bounds;
        }
        if styles.knob.is_some() {
            self.knob_style = styles.knob;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if let Some(align) = styles.text_align {
            self.text_align = align;
        }
        if styles.lower_bound.is_some() {
            self.lower_bound = styles.lower_bound.map(Cow::Borrowed);
        }
        if styles.upper_bound.is_some() {
            self.upper_bound = styles.upper_bound.map(Cow::Borrowed);
        }
        if styles.track_char.is_some() {
            self.track_char = styles.track_char.map(Cow::Borrowed);
        }
        if styles.vertical_knob.is_some() {
            self.vertical_knob = styles.vertical_knob.map(Cow::Borrowed);
        }
        if styles.horizontal_knob.is_some() {
            self.horizontal_knob = styles.horizontal_knob.map(Cow::Borrowed);
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self
    }

    /// Style for focus.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Style for the bounds text.
    pub fn bounds_style(mut self, style: Style) -> Self {
        self.bounds_style = Some(style);
        self
    }

    /// Style for the knob.
    pub fn knob_style(mut self, style: Style) -> Self {
        self.knob_style = Some(style);
        self
    }

    /// Text alignment. Used for the bounds and the knob.
    pub fn text_align(mut self, align: Alignment) -> Self {
        self.text_align = align;
        self
    }

    /// Text for the lower bound. Can contain newlines.
    pub fn lower_bound(mut self, bound: impl Into<Cow<'a, str>>) -> Self {
        self.lower_bound = Some(bound.into());
        self
    }

    /// Text for the upper bound. Can contain newlines.
    pub fn upper_bound(mut self, bound: impl Into<Cow<'a, str>>) -> Self {
        self.upper_bound = Some(bound.into());
        self
    }

    /// Fill char for the track.
    pub fn track_char(mut self, bound: impl Into<Cow<'a, str>>) -> Self {
        self.track_char = Some(bound.into());
        self
    }

    /// Text for the horizontal knob. Can contain newlines for
    /// multiline sliders.
    pub fn horizontal_knob(mut self, knob: impl Into<Cow<'a, str>>) -> Self {
        self.horizontal_knob = Some(knob.into());
        self
    }

    /// Text for the vertical knob. Can contain newlines for a
    /// multiline knob.
    pub fn vertical_knob(mut self, knob: impl Into<Cow<'a, str>>) -> Self {
        self.vertical_knob = Some(knob.into());
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }
}

impl<'a, T> Slider<'a, T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    // Creates the default knob text.
    // knob_repeat is either rows/columns, the off direction.
    fn render_knob_str(&'a self, knob_repeat: u16, is_focused: bool) -> Cow<'a, str> {
        fn map_ref<'b>(s0: &'b Option<Cow<'b, str>>, d: Cow<'b, str>) -> Cow<'b, str> {
            s0.as_ref().map(|v| Cow::Borrowed(v.as_ref())).unwrap_or(d)
        }

        if is_focused {
            match (self.direction, knob_repeat) {
                (Direction::Horizontal, 1) => map_ref(&self.horizontal_knob, Cow::Borrowed(" | ")),
                (Direction::Horizontal, 2) => {
                    map_ref(&self.horizontal_knob, Cow::Borrowed(" ╷ \n ╵ "))
                }
                (Direction::Horizontal, 3) => {
                    map_ref(&self.horizontal_knob, Cow::Borrowed(" ╷ \n │ \n ╵ "))
                }
                (Direction::Horizontal, 4) => {
                    map_ref(&self.horizontal_knob, Cow::Borrowed(" ╷ \n │ \n │ \n ╵ "))
                }
                (Direction::Horizontal, 5) => map_ref(
                    &self.horizontal_knob,
                    Cow::Borrowed(" ╷ \n │ \n │ \n │ \n ╵ "),
                ),
                (Direction::Horizontal, n) => {
                    let mut tmp = String::new();
                    tmp.push_str(" ╷ \n");
                    for _ in 0..n - 2 {
                        tmp.push_str(" │ \n");
                    }
                    tmp.push_str(" ╵ ");
                    map_ref(&self.horizontal_knob, Cow::Owned(tmp))
                }

                (Direction::Vertical, 1) => map_ref(&self.vertical_knob, Cow::Borrowed("─")),
                (Direction::Vertical, 2) => map_ref(&self.vertical_knob, Cow::Borrowed("╶╴")),
                (Direction::Vertical, 3) => map_ref(&self.vertical_knob, Cow::Borrowed("╶─╴")),
                (Direction::Vertical, 4) => map_ref(&self.vertical_knob, Cow::Borrowed("╶──╴")),
                (Direction::Vertical, 5) => map_ref(&self.vertical_knob, Cow::Borrowed("╶───╴")),
                (Direction::Vertical, n) => {
                    let mut tmp = String::new();
                    tmp.push('╶');
                    for _ in 0..n - 2 {
                        tmp.push('─');
                    }
                    tmp.push('╴');
                    map_ref(&self.vertical_knob, Cow::Owned(tmp))
                }
            }
        } else {
            match (self.direction, knob_repeat) {
                (Direction::Horizontal, 1) => map_ref(&self.horizontal_knob, Cow::Borrowed("   ")),
                (Direction::Horizontal, 2) => {
                    map_ref(&self.horizontal_knob, Cow::Borrowed("   \n   "))
                }
                (Direction::Horizontal, 3) => {
                    map_ref(&self.horizontal_knob, Cow::Borrowed("   \n   \n   "))
                }
                (Direction::Horizontal, 4) => {
                    map_ref(&self.horizontal_knob, Cow::Borrowed("   \n   \n   \n   "))
                }
                (Direction::Horizontal, 5) => map_ref(
                    &self.horizontal_knob,
                    Cow::Borrowed("   \n   \n   \n   \n   "),
                ),
                (Direction::Horizontal, n) => {
                    let mut tmp = String::new();
                    tmp.push_str("   \n");
                    for _ in 0..n - 2 {
                        tmp.push_str("   \n");
                    }
                    tmp.push_str("   ");
                    map_ref(&self.horizontal_knob, Cow::Owned(tmp))
                }

                (Direction::Vertical, 1) => map_ref(&self.vertical_knob, Cow::Borrowed(" ")),
                (Direction::Vertical, 2) => map_ref(&self.vertical_knob, Cow::Borrowed("  ")),
                (Direction::Vertical, 3) => map_ref(&self.vertical_knob, Cow::Borrowed("   ")),
                (Direction::Vertical, 4) => map_ref(&self.vertical_knob, Cow::Borrowed("    ")),
                (Direction::Vertical, 5) => map_ref(&self.vertical_knob, Cow::Borrowed("     ")),
                (Direction::Vertical, n) => {
                    map_ref(&self.vertical_knob, Cow::Owned(" ".repeat(n as usize)))
                }
            }
        }
    }

    // layout
    fn layout(&self, area: Rect, state: &mut SliderState<T>) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);
        state.direction = self.direction;

        if let Some(range) = self.range {
            state.range = range;
        }
        if let Some(step) = self.step {
            state.step = step;
        }
        if let Some(long_step) = self.long_step {
            state.long_step = Some(long_step);
        }

        let inner = state.inner;

        match self.direction {
            Direction::Horizontal => {
                let lower_width = self
                    .lower_bound
                    .as_ref()
                    .map(|v| v.width() as u16)
                    .unwrap_or_default();
                let upper_width = self
                    .upper_bound
                    .as_ref()
                    .map(|v| v.width() as u16)
                    .unwrap_or_default();

                state.lower_bound = Rect::new(inner.x, inner.y, lower_width, inner.height);
                state.upper_bound = Rect::new(
                    (inner.x + inner.width).saturating_sub(upper_width),
                    inner.y,
                    upper_width,
                    inner.height,
                );

                let track_len = state
                    .upper_bound
                    .x
                    .saturating_sub(state.lower_bound.right());
                state.track =
                    Rect::new(state.lower_bound.right(), inner.y, track_len, inner.height);

                let knob_width = self
                    .render_knob_str(inner.height, false)
                    .split('\n')
                    .next()
                    .expect("one knob")
                    .width() as u16;
                state.scale_len = track_len.saturating_sub(knob_width);

                if let Some(knob_pos) = state.value.map_range(state.range, (0, state.scale_len)) {
                    state.knob =
                        Rect::new(state.track.x + knob_pos, inner.y, knob_width, inner.height)
                } else {
                    state.knob = Rect::new(state.track.x, inner.y, 0, inner.height);
                }
            }
            Direction::Vertical => {
                let lower_height = self
                    .lower_bound
                    .as_ref()
                    .map(|v| v.split('\n').count() as u16)
                    .unwrap_or_default();
                let upper_height = self
                    .upper_bound
                    .as_ref()
                    .map(|v| v.split('\n').count() as u16)
                    .unwrap_or_default();

                state.lower_bound = Rect::new(inner.x, inner.y, inner.width, lower_height);
                state.upper_bound = Rect::new(
                    inner.x,
                    inner.bottom().saturating_sub(upper_height),
                    inner.width,
                    upper_height,
                );

                let track_len = inner.height.saturating_sub(lower_height + upper_height);
                state.track = Rect::new(inner.x, inner.y + lower_height, inner.width, track_len);

                state.scale_len = track_len.saturating_sub(1);

                if let Some(knob_pos) = state.value.map_range(state.range, (0, state.scale_len)) {
                    state.knob = Rect::new(inner.x, state.track.y + knob_pos, inner.width, 1)
                } else {
                    state.knob = Rect::new(inner.x, state.track.y, inner.width, 0)
                }
            }
        }
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a, T> StatefulWidgetRef for Slider<'a, T> {
    type State = SliderState<T>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_slider(self, area, buf, state);
    }
}

impl<T> StatefulWidget for Slider<'_, T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    type State = SliderState<T>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_slider(&self, area, buf, state);
    }
}

fn render_slider<T>(
    widget: &Slider<'_, T>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut SliderState<T>,
) where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    widget.layout(area, state);

    if let Some(block) = widget.block.as_ref() {
        block.render(area, buf);
    } else {
        buf.set_style(area, widget.style);
    }

    let style = if widget.style == Default::default() {
        Style::default().black().on_gray()
    } else {
        widget.style
    };
    let bounds_style = widget.bounds_style.unwrap_or(style);
    let knob_style = if state.is_focused() {
        widget.focus_style.unwrap_or(revert_style(style))
    } else {
        widget.knob_style.unwrap_or(revert_style(style))
    };

    if let Some(lower_bound_str) = widget.lower_bound.as_ref() {
        match widget.direction {
            Direction::Horizontal => {
                buf.set_style(state.lower_bound, bounds_style);

                // need to vertically align manually.
                let lower_height = lower_bound_str.split('\n').count() as u16;
                let y_offset = match widget.text_align {
                    Alignment::Left => 0,
                    Alignment::Center => state.lower_bound.height.saturating_sub(lower_height) / 2,
                    Alignment::Right => state.lower_bound.height.saturating_sub(lower_height),
                };
                let txt_area = Rect::new(
                    state.lower_bound.x,
                    state.lower_bound.y + y_offset,
                    state.lower_bound.width,
                    state.lower_bound.height,
                );

                Text::from(lower_bound_str.as_ref())
                    .alignment(widget.text_align)
                    .render(txt_area, buf);
            }
            Direction::Vertical => {
                Text::from(lower_bound_str.as_ref())
                    .style(bounds_style)
                    .alignment(widget.text_align)
                    .render(state.lower_bound, buf);
            }
        }
    }
    if let Some(upper_bound_str) = widget.upper_bound.as_ref() {
        match widget.direction {
            Direction::Horizontal => {
                buf.set_style(state.upper_bound, bounds_style);

                // need to vertically align manually.
                let upper_height = upper_bound_str.split('\n').count() as u16;
                let y_offset = match widget.text_align {
                    Alignment::Left => 0,
                    Alignment::Center => state.upper_bound.height.saturating_sub(upper_height) / 2,
                    Alignment::Right => state.upper_bound.height.saturating_sub(upper_height),
                };

                let txt_area = Rect::new(
                    state.upper_bound.x,
                    state.upper_bound.y + y_offset,
                    state.upper_bound.width,
                    state.upper_bound.height,
                );

                Text::from(upper_bound_str.as_ref())
                    .alignment(widget.text_align)
                    .render(txt_area, buf);
            }
            Direction::Vertical => {
                Text::from(upper_bound_str.as_ref())
                    .style(bounds_style)
                    .alignment(widget.text_align)
                    .render(state.upper_bound, buf);
            }
        }
    }

    let track_str = widget.track_char.as_ref().unwrap_or(&Cow::Borrowed(" "));
    if " " != track_str.as_ref() {
        for y in state.track.top()..state.track.bottom() {
            for x in state.track.left()..state.track.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(track_str.as_ref());
                }
            }
        }
    }

    match widget.direction {
        Direction::Horizontal => {
            let knob_str = widget.render_knob_str(state.knob.height, state.is_focused());
            Text::from(knob_str.as_ref())
                .style(knob_style)
                .render(state.knob, buf);
        }
        Direction::Vertical => {
            let knob_str = widget.render_knob_str(state.knob.width, state.is_focused());
            Line::from(knob_str)
                .alignment(widget.text_align)
                .style(knob_style)
                .render(state.knob, buf);
        }
    }
}

impl<T> Debug for SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SliderState")
            .field("area", &self.area)
            .field("inner", &self.inner)
            .field("lower_bound", &self.lower_bound)
            .field("upper_bound", &self.upper_bound)
            .field("track", &self.track)
            .field("knob", &self.knob)
            .field("scale_len", &self.scale_len)
            .field("direction", &self.direction)
            .field("range", &self.range)
            .field("step", &self.step)
            .field("long_step", &self.long_step)
            .field("value", &self.value)
            .field("focus", &self.focus)
            .field("mouse", &self.mouse)
            .finish()
    }
}

impl<T> HasFocus for SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
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

impl<T> RelocatableState for SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.inner = relocate_area(self.inner, shift, clip);
        self.lower_bound = relocate_area(self.lower_bound, shift, clip);
        self.upper_bound = relocate_area(self.upper_bound, shift, clip);
        self.track = relocate_area(self.track, shift, clip);
        self.knob = relocate_area(self.knob, shift, clip);
    }
}

macro_rules! slider_new {
    ($tt:ty) => {
        impl Default for SliderState<$tt> {
            fn default() -> Self {
                Self {
                    area: Default::default(),
                    inner: Default::default(),
                    lower_bound: Default::default(),
                    upper_bound: Default::default(),
                    track: Default::default(),
                    knob: Default::default(),
                    scale_len: 0,
                    direction: Default::default(),
                    range: (<$tt>::MIN, <$tt>::MAX),
                    step: 1,
                    long_step: None,
                    value: Default::default(),
                    focus: Default::default(),
                    mouse: Default::default(),
                    non_exhaustive: NonExhaustive,
                }
            }
        }

        impl SliderState<$tt> {
            pub fn new() -> Self {
                Self::new_range((<$tt>::MIN, <$tt>::MAX), 1)
            }
        }
    };
}
macro_rules! slider_new_f {
    ($tt:ty) => {
        impl Default for SliderState<$tt> {
            fn default() -> Self {
                Self {
                    area: Default::default(),
                    inner: Default::default(),
                    lower_bound: Default::default(),
                    upper_bound: Default::default(),
                    track: Default::default(),
                    knob: Default::default(),
                    scale_len: 0,
                    direction: Default::default(),
                    range: (<$tt>::MIN, <$tt>::MAX),
                    step: 1.,
                    long_step: None,
                    value: Default::default(),
                    focus: Default::default(),
                    mouse: Default::default(),
                    non_exhaustive: NonExhaustive,
                }
            }
        }

        impl SliderState<$tt> {
            pub fn new() -> Self {
                Self::new_range((<$tt>::MIN, <$tt>::MAX), 1.)
            }
        }
    };
}

impl<T> Clone for SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            lower_bound: self.lower_bound,
            upper_bound: self.upper_bound,
            track: self.track,
            knob: self.knob,
            scale_len: self.scale_len,
            direction: self.direction,
            range: self.range,
            step: self.step,
            long_step: self.long_step,
            value: self.value,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

slider_new!(u8);
slider_new!(u16);
slider_new!(u32);
slider_new!(u64);
slider_new!(usize);
slider_new!(i8);
slider_new!(i16);
slider_new!(i32);
slider_new!(i64);
slider_new!(isize);
slider_new_f!(f32);
slider_new_f!(f64);

impl<T> SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    /// New state with a given range and step.
    ///
    /// The range will still be overridden when set with the Widget.
    pub fn new_range(range: (T, T), step: T::Step) -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            lower_bound: Default::default(),
            upper_bound: Default::default(),
            track: Default::default(),
            knob: Default::default(),
            scale_len: 0,
            direction: Default::default(),
            range,
            step,
            long_step: None,
            value: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// Change the value.
    pub fn set_value(&mut self, value: T) -> bool {
        let old_value = self.value;
        self.value = value;
        old_value != value
    }

    /// Current value.
    pub fn value(&self) -> T {
        self.value
    }

    /// Set the range.
    pub fn set_range(&mut self, range: (T, T)) {
        self.range = range;
    }

    /// Range.
    pub fn range(&self) -> (T, T) {
        self.range
    }

    /// Minor step size.
    pub fn set_step(&mut self, step: T::Step) {
        self.step = step;
    }

    /// Minor step size.
    pub fn step(&self) -> T::Step {
        self.step
    }

    /// Major step size.
    pub fn set_long_step(&mut self, step: T::Step) {
        self.long_step = Some(step);
    }

    /// Major step size.
    pub fn long_step(&self) -> Option<T::Step> {
        self.long_step
    }

    /// Next value by one step.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        let old_value = self.value;
        self.value = self.value.add_clamp(self.step, self.range);
        old_value != self.value
    }

    /// Previous value by one step.
    pub fn prev(&mut self) -> bool {
        let old_value = self.value;
        self.value = self.value.sub_clamp(self.step, self.range);
        old_value != self.value
    }

    /// Next value by one major step.
    pub fn next_major(&mut self) -> bool {
        let old_value = self.value;
        if let Some(long_step) = self.long_step {
            self.value = self.value.add_clamp(long_step, self.range);
        }
        old_value != self.value
    }

    /// Previous value by one major step.
    pub fn prev_major(&mut self) -> bool {
        let old_value = self.value;
        if let Some(long_step) = self.long_step {
            self.value = self.value.sub_clamp(long_step, self.range);
        }
        old_value != self.value
    }

    /// Clicked in the range or at the boundary.
    /// Transforms the relative screen position to a value.
    pub fn clicked_at(&mut self, x: u16, y: u16) -> bool {
        match self.direction {
            Direction::Horizontal => {
                let x_pos = x.saturating_sub(self.track.x);
                if x_pos >= self.track.width {
                    self.value = self.range.1;
                    true
                } else if let Some(value) = x_pos.map_range((0, self.scale_len), self.range) {
                    self.value = value;
                    true
                } else {
                    false
                }
            }
            Direction::Vertical => {
                let y_pos = y.saturating_sub(self.track.y);
                if y_pos >= self.track.height {
                    self.value = self.range.1;
                    true
                } else if let Some(value) = y_pos.map_range((0, self.scale_len), self.range) {
                    self.value = value;
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl<T> HandleEvent<crossterm::event::Event, Regular, Outcome> for SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        let r = if self.is_focused() {
            match event {
                ct_event!(keycode press CONTROL-Left)
                | ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press Home) => self.set_value(self.range.0).into(),

                ct_event!(keycode press CONTROL-Right)
                | ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press End) => self.set_value(self.range.1).into(),

                ct_event!(keycode press Up)
                | ct_event!(keycode press Left)
                | ct_event!(key press '-') => self.prev().into(),
                ct_event!(keycode press Down)
                | ct_event!(keycode press Right)
                | ct_event!(key press '+') => self.next().into(),

                ct_event!(keycode press PageUp)
                | ct_event!(keycode press ALT-Up)
                | ct_event!(keycode press ALT-Left)
                | ct_event!(key press ALT-'-') => self.prev_major().into(),
                ct_event!(keycode press PageDown)
                | ct_event!(keycode press ALT-Down)
                | ct_event!(keycode press ALT-Right)
                | ct_event!(key press ALT-'+') => self.next_major().into(),
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        if r == Outcome::Continue {
            HandleEvent::handle(self, event, MouseOnly)
        } else {
            r
        }
    }
}

impl<T> HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for SliderState<T>
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse drag Left for x,y) | ct_event!(mouse down Left for x,y) => {
                if self.inner.contains(Position::new(*x, *y)) {
                    self.clicked_at(*x, *y).into()
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(scroll down for x,y) => {
                if self.track.contains(Position::new(*x, *y)) {
                    self.next().into()
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(scroll up for x,y) => {
                if self.track.contains(Position::new(*x, *y)) {
                    self.prev().into()
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(scroll ALT down for x,y) => {
                if self.track.contains(Position::new(*x, *y)) {
                    self.next_major().into()
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(scroll ALT up for x,y) => {
                if self.track.contains(Position::new(*x, *y)) {
                    self.prev_major().into()
                } else {
                    Outcome::Continue
                }
            }
            _ => Outcome::Continue,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events<T>(
    state: &mut SliderState<T>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events<T>(
    state: &mut SliderState<T>,
    event: &crossterm::event::Event,
) -> Outcome
where
    T: RangeOp<Step: Copy + Debug> + MapRange<u16> + Debug + Default + Copy + PartialEq,
    u16: MapRange<T>,
{
    HandleEvent::handle(state, event, MouseOnly)
}
