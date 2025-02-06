//! Widget that helps with rendering a layout using GenericForm.
//!
//!
//! ```
//! # use ratatui::buffer::Buffer;
//! # use ratatui::layout::{Flex, Rect};
//! # use ratatui::text::Span;
//! # use ratatui::widgets::{Padding, Widget, StatefulWidget};
//! # use rat_focus::{FocusFlag, HasFocus};
//! # use rat_text::text_input::{TextInput, TextInputState};
//! # use rat_widget::layout::{FormLabel, FormWidget, GenericLayout, LayoutForm};
//! use rat_widget::pager::{Form, FormState};
//! #
//! # struct State {
//! #     form: FormState<FocusFlag>,
//! #     text1: TextInputState,
//! #     text2: TextInputState,
//! #     text3: TextInputState,
//! # }
//! #
//! # let mut state = State {form: Default::default(),text1: Default::default(),text2: Default::default(),text3: Default::default()};
//! # let area = Rect::default();
//! # let mut buf = Buffer::empty(Rect::default());
//! # let buf = &mut buf;
//!
//! if !state.form.valid_layout(area.as_size()) {
//!     let mut form_layout = LayoutForm::new()
//!             .spacing(1)
//!             .flex(Flex::Legacy)
//!             .line_spacing(1)
//!             .min_label(10);
//!
//!     form_layout.widget(
//!         state.text1.focus(),
//!         FormLabel::Str("Text 1"),
//!         // single row
//!         FormWidget::Width(22)
//!     );
//!     form_layout.widget(
//!         state.text2.focus(),
//!         FormLabel::Str("Text 2"),
//!         // stretch to the form-width, preferred with 15, 1 row high.
//!         FormWidget::StretchX(15, 1)
//!     );
//!     form_layout.widget(
//!         state.text3.focus(),
//!         FormLabel::Str("Text 3"),
//!         // stretch to the form-width and fill vertically.
//!         // preferred width is 15 3 rows high.
//!         FormWidget::StretchXY(15, 3)
//!     );
//!
//!     // calculate the layout and set it.
//!     state.form.set_layout(form_layout.paged(area.as_size(), Padding::default()));
//!  }
//!
//!  let mut form = Form::new()
//!     .into_buffer(area, buf, &mut state.form);
//!
//!  form.render(state.text1.focus(),
//!     || TextInput::new(),
//!     &mut state.text1
//!  );
//!  form.render(state.text2.focus(),
//!     || TextInput::new(),
//!     &mut state.text2
//!  );
//!  form.render(state.text3.focus(),
//!     || TextInput::new(),
//!     &mut state.text3
//!  );
//!
//! ```

use crate::_private::NonExhaustive;
use crate::layout::GenericLayout;
use crate::pager::{Pager, PagerBuffer, PagerStyle};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

/// A widget that helps with rendering a Form.
/// Uses a GenericLayout for its layout information.
/// Does no scrolling/paging whatsoever and any widgets
/// out of view are simply not rendered.
#[derive(Debug, Clone)]
pub struct Form<'a, W>
where
    W: Eq + Hash + Clone,
{
    style: Style,
    block: Option<Block<'a>>,
    layout: Option<GenericLayout<W>>,
    pager: Pager<W>,
    phantom: PhantomData<&'a ()>,
}

/// Renders directly to the frame buffer.
///
/// * It maps your widget area from layout coordinates
///   to screen coordinates before rendering.
/// * It helps with cleanup of the widget state if your
///   widget is currently invisible.
#[derive(Debug)]
pub struct FormBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    pager: PagerBuffer<'a, W>,
}

/// Widget state.
#[derive(Debug, Clone)]
pub struct FormState<W>
where
    W: Eq + Hash + Clone,
{
    /// Page layout
    /// __read+write__ renewed with each render.
    pub layout: Rc<RefCell<GenericLayout<W>>>,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

impl<W> Default for Form<'_, W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            style: Default::default(),
            block: Default::default(),
            layout: Default::default(),
            pager: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<'a, W> Form<'a, W>
where
    W: Eq + Hash + Clone,
{
    /// New SinglePage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the layout. If no layout is set here the layout is
    /// taken from the state.
    pub fn layout(mut self, layout: GenericLayout<W>) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self.pager = self.pager.style(style);
        self
    }

    /// Style for text labels.
    pub fn label_style(mut self, style: Style) -> Self {
        self.pager = self.pager.label_style(style);
        self
    }

    /// Alignment for text labels.
    pub fn label_alignment(mut self, alignment: Alignment) -> Self {
        self.pager = self.pager.label_alignment(alignment);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: PagerStyle) -> Self {
        self.style = styles.style;
        self.block = self.block.map(|v| v.style(styles.style));
        self.pager = self.pager.styles(styles.clone());
        self
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block.style(self.style));
        self
    }

    /// Calculate the layout page size.
    pub fn layout_size(&self, area: Rect) -> Size {
        self.block.inner_if_some(area).as_size()
    }

    // Calculate the view area for all columns.
    pub fn inner(&self, area: Rect) -> Rect {
        self.block.inner_if_some(area)
    }

    /// Render the page navigation and create the SinglePagerBuffer
    /// that will do the actual widget rendering.
    pub fn into_buffer(
        self,
        area: Rect,
        buf: &'a mut Buffer,
        state: &'a mut FormState<W>,
    ) -> FormBuffer<'a, W> {
        // render border
        self.block.render(area, buf);
        // set layout
        if let Some(layout) = self.layout {
            state.layout = Rc::new(RefCell::new(layout));
        }

        FormBuffer {
            pager: self
                .pager //
                .layout(state.layout.clone())
                .page(0)
                .into_buffer(area, Rc::new(RefCell::new(buf))),
        }
    }
}

impl<'a, W> FormBuffer<'a, W>
where
    W: Eq + Hash + Clone,
{
    /// Is the given area visible?
    pub fn is_visible(&self, widget: W) -> bool {
        if let Some(idx) = self.pager.widget_idx(widget) {
            self.pager.is_visible(idx)
        } else {
            false
        }
    }

    /// Render all blocks for the current page.
    pub fn render_block(&mut self) {
        self.pager.render_block()
    }

    /// Render a manual label.
    #[inline(always)]
    pub fn render_label<FN, WW>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce(&Option<Cow<'static, str>>) -> WW,
        WW: Widget,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_label(idx, render_fn)
    }

    /// Render a stateless widget and its label, if any.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_auto_label(idx);
        self.pager.render_widget(idx, render_fn)
    }

    /// Render an optional stateful widget and its label, if any.
    #[inline(always)]
    pub fn render_opt<FN, WW, SS>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> Option<WW>,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_auto_label(idx);
        if !self.pager.render_opt(idx, render_fn, state) {
            self.hidden(state);
            false
        } else {
            true
        }
    }

    /// Render a stateful widget and its label, if any.
    #[inline(always)]
    pub fn render<FN, WW, SS>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> WW,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return false;
        };
        self.pager.render_auto_label(idx);
        if !self.pager.render(idx, render_fn, state) {
            self.hidden(state);
            false
        } else {
            true
        }
    }

    /// Render a stateful widget and its label, if any.
    /// The closure can return a second value, which will be forwarded
    /// if the widget is visible.
    #[inline(always)]
    #[allow(clippy::question_mark)]
    pub fn render2<FN, WW, SS, R>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> Option<R>
    where
        FN: FnOnce() -> (WW, R),
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.render_auto_label(idx);
        if let Some(remainder) = self.pager.render2(idx, render_fn, state) {
            Some(remainder)
        } else {
            self.hidden(state);
            None
        }
    }

    /// Calculate the necessary shift from view to screen.
    /// This does nothing as pager always places the widgets
    /// in screen coordinates.
    ///
    /// Just to keep the api in sync with [Clipper](crate::clipper::Clipper).
    pub fn shift(&self) -> (i16, i16) {
        (0, 0)
    }

    /// Relocate the widget area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    #[allow(clippy::question_mark)]
    pub fn locate_widget(&self, widget: W) -> Option<Rect> {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.locate_widget(idx)
    }

    /// Relocate the label area to screen coordinates.
    /// Returns None if the widget is not visible.
    /// This clips the area to page_area.
    #[allow(clippy::question_mark)]
    pub fn locate_label(&self, widget: W) -> Option<Rect> {
        let Some(idx) = self.pager.widget_idx(widget) else {
            return None;
        };
        self.pager.locate_label(idx)
    }

    /// Relocate an area from layout coordinates to screen coordinates.
    /// A result None indicates that the area is invisible.
    ///
    /// This will clip the area to the page_area.
    pub fn locate_area(&self, area: Rect) -> Option<Rect> {
        self.pager.locate_area(area)
    }

    /// Does nothing for pager.
    /// Just to keep the api in sync with [Clipper](crate::clipper::Clipper).
    pub fn relocate<S>(&self, _state: &mut S)
    where
        S: RelocatableState,
    {
    }

    /// Clear the areas in the widget-state.
    /// This is called by render_xx whenever a widget is invisible.
    pub fn hidden<S>(&self, state: &mut S)
    where
        S: RelocatableState,
    {
        state.relocate((0, 0), Rect::default())
    }

    /// Get access to the buffer during rendering a page.
    pub fn buffer<'b>(&'b mut self) -> RefMut<'b, &'a mut Buffer> {
        self.pager.buffer()
    }
}

impl<W> Default for FormState<W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W> FormState<W>
where
    W: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Layout needs to change?
    pub fn valid_layout(&self, size: Size) -> bool {
        let layout = self.layout.borrow();
        !layout.size_changed(size) && !layout.is_empty()
    }

    /// Set the layout.
    pub fn set_layout(&mut self, layout: GenericLayout<W>) {
        self.layout = Rc::new(RefCell::new(layout));
    }

    /// Layout.
    pub fn layout(&self) -> Ref<'_, GenericLayout<W>> {
        self.layout.borrow()
    }

    /// Clear the layout data and reset the page/page-count.
    pub fn clear(&mut self) {
        self.layout.borrow_mut().clear();
    }
}
