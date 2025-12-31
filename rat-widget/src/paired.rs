//!
//! Render two widgets in one area.
//!
//! This is nice when you have your layout figured out and
//! then there is the special case where you have to fit
//! two widgets in one layout-area.
//!
//! ```
//! use ratatui_core::buffer::Buffer;
//! use ratatui_core::layout::Rect;
//! use ratatui_core::text::Line;
//! use ratatui_core::widgets::StatefulWidget;
//! use rat_widget::paired::{PairSplit, Paired, PairedState, PairedWidget};
//! use rat_widget::slider::{Slider, SliderState};
//!
//! let value = "2024";
//! # let area = Rect::new(10, 10, 30, 1);
//! # let mut buf = Buffer::empty(area);
//! # let buf = &mut buf;
//! # let mut slider_state = SliderState::new_range((2015u32, 2024u32), 3u32);
//!
//! Paired::new(
//!     Slider::new()
//!         .range((2015u32, 2024u32))
//!         .step(3u32),
//!     PairedWidget::new(Line::from(value)),
//! )
//! .split(PairSplit::Fix1(18))
//! .render(area, buf, &mut PairedState::new(
//!     &mut slider_state,
//!     &mut ()
//! ));
//!
//! ```
//!
//! This example also uses `PairedWidget` to convert a Widget to
//! a StatefulWidget. Otherwise, you can only combine two Widgets
//! or two StatefulWidgets.
//!
use rat_reloc::RelocatableState;
use rat_text::HasScreenCursor;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Constraint, Flex, Layout, Rect};
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use std::marker::PhantomData;
use std::rc::Rc;

/// How to split the area for the two widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairSplit {
    /// Both widgets have a preferred size.
    Fix(u16, u16),
    /// The first widget has a preferred size.
    /// The second gets the rest.
    Fix1(u16),
    /// The second widget has a preferred size.
    /// The first gets the rest.
    Fix2(u16),
    /// Always split the area in the given ratio.
    Ratio(u16, u16),
    /// Use the given Constraints
    Constrain(Constraint, Constraint),
}

/// Renders 2 widgets side by side.
#[derive(Debug)]
pub struct Paired<'a, T, U> {
    first: T,
    second: U,
    split: PairSplit,
    spacing: u16,
    flex: Flex,
    phantom: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct PairedState<'a, TS, US> {
    pub first: &'a mut TS,
    pub second: &'a mut US,
}

/// New-type for Span without Widget trait.
/// Used for [new_labeled].
pub struct NSpan<'a> {
    pub span: Span<'a>,
}

impl<'a, U> Paired<'a, NSpan<'a>, U> {
    /// Create a pair of a label + a stateful widget.
    pub fn new_labeled(label: impl Into<Span<'a>>, second: U) -> Self {
        let label = label.into();
        let width = label.width();
        Self {
            first: NSpan { span: label },
            second,
            split: PairSplit::Fix1(width as u16),
            spacing: 1,
            flex: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, T, U> Paired<'a, T, U> {
    pub fn new(first: T, second: U) -> Self {
        Self {
            first,
            second,
            split: PairSplit::Ratio(1, 1),
            spacing: 1,
            flex: Default::default(),
            phantom: Default::default(),
        }
    }

    pub fn split(mut self, split: PairSplit) -> Self {
        self.split = split;
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }
}

impl<T, U> Paired<'_, T, U> {
    fn layout(&self, area: Rect) -> Rc<[Rect]> {
        match self.split {
            PairSplit::Fix(a, b) => {
                Layout::horizontal([Constraint::Length(a), Constraint::Length(b)])
                    .spacing(self.spacing)
                    .flex(self.flex)
                    .split(area) //
            }
            PairSplit::Fix1(a) => {
                Layout::horizontal([Constraint::Length(a), Constraint::Fill(1)])
                    .spacing(self.spacing)
                    .flex(self.flex)
                    .split(area) //
            }
            PairSplit::Fix2(b) => {
                Layout::horizontal([Constraint::Fill(1), Constraint::Length(b)])
                    .spacing(self.spacing)
                    .flex(self.flex)
                    .split(area) //
            }
            PairSplit::Ratio(a, b) => {
                Layout::horizontal([Constraint::Fill(a), Constraint::Fill(b)])
                    .spacing(self.spacing)
                    .flex(self.flex)
                    .split(area) //
            }
            PairSplit::Constrain(a, b) => {
                Layout::horizontal([a, b])
                    .spacing(self.spacing)
                    .flex(self.flex)
                    .split(area) //
            }
        }
    }
}

impl<'a, U, US> StatefulWidget for Paired<'a, NSpan<'a>, U>
where
    U: StatefulWidget<State = US>,
    US: 'a,
{
    type State = US;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let l = self.layout(area);
        self.first.span.render(l[0], buf);
        self.second.render(l[1], buf, state);
    }
}

impl<'a, T, U, TS, US> StatefulWidget for Paired<'a, T, U>
where
    T: StatefulWidget<State = TS>,
    U: StatefulWidget<State = US>,
    TS: 'a,
    US: 'a,
{
    type State = PairedState<'a, TS, US>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let l = self.layout(area);
        self.first.render(l[0], buf, state.first);
        self.second.render(l[1], buf, state.second);
    }
}

impl<'a, U> Widget for Paired<'a, NSpan<'a>, U>
where
    U: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let l = self.layout(area);
        self.first.span.render(l[0], buf);
        self.second.render(l[1], buf);
    }
}

impl<T, U> Widget for Paired<'_, T, U>
where
    T: Widget,
    U: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let l = self.layout(area);
        self.first.render(l[0], buf);
        self.second.render(l[1], buf);
    }
}

impl<TS, US> HasScreenCursor for PairedState<'_, TS, US>
where
    TS: HasScreenCursor,
    US: HasScreenCursor,
{
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.first.screen_cursor().or(self.second.screen_cursor())
    }
}

impl<TS, US> RelocatableState for PairedState<'_, TS, US>
where
    TS: RelocatableState,
    US: RelocatableState,
{
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.first.relocate(shift, clip);
        self.second.relocate(shift, clip);
    }
}

impl<'a, TS, US> PairedState<'a, TS, US> {
    pub fn new(first: &'a mut TS, second: &'a mut US) -> Self {
        Self { first, second }
    }
}

/// If you want to pair up a StatefulWidget and a Widget you
/// need this adapter for the widget.
#[derive(Debug)]
pub struct PairedWidget<'a, T> {
    widget: T,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T> PairedWidget<'a, T> {
    pub fn new(widget: T) -> Self {
        Self {
            widget,
            phantom: Default::default(),
        }
    }
}

impl<'a, T> StatefulWidget for PairedWidget<'a, T>
where
    T: Widget,
{
    type State = ();

    fn render(self, area: Rect, buf: &mut Buffer, _: &mut Self::State) {
        self.widget.render(area, buf);
    }
}

impl<'a, T> HasScreenCursor for PairedWidget<'a, T> {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}
