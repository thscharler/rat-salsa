//! Render widgets based on a [GenericLayout].
//!
//! This is useful, if you have more than 3 input widgets and
//! their accompanying labels in a row.
//! See [LayoutForm](crate::layout::LayoutForm) for details on
//! the layout.
//!
//! If the layout is split into multiple pages this also
//! renders the page-navigation and handles scrolling
//! through the pages.
//!
//! ```
//! # use ratatui::buffer::Buffer;
//! # use ratatui::layout::{Flex, Rect};
//! # use ratatui::text::Span;
//! # use ratatui::widgets::{Padding, Widget, StatefulWidget, Block};
//! # use rat_focus::{FocusFlag, HasFocus};
//! # use rat_text::text_input::{TextInput, TextInputState};
//! # use rat_widget::layout::{FormLabel, FormWidget, GenericLayout, LayoutForm};
//! use rat_widget::form::{Form, FormState};
//! #
//! # struct State {
//! #     form: FormState<usize>,
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
//! // create + configure the form.
//! let form = Form::new()
//!     .block(Block::bordered());
//!
//! let layout_size = form.layout_size(area);
//! if !state.form.valid_layout(layout_size) {
//!     // define the layout
//!     let mut form_layout = LayoutForm::new()
//!             .spacing(1)
//!             .flex(Flex::Legacy)
//!             .line_spacing(1)
//!             .min_label(10);
//!
//!     use rat_widget::layout::{FormLabel as L, FormWidget as W};
//!
//!     // single row
//!     form_layout.widget(state.text1.id(), L::Str("Text 1"), W::Width(22));
//!     // stretch to the form-width, preferred with 15, 1 row high.
//!     form_layout.widget(state.text2.id(), L::Str("Text 2"), W::StretchX(15, 1));
//!     // stretch to the form-width and fill vertically.
//!     // preferred width is 15 3 rows high.
//!     form_layout.widget(state.text3.id(), L::Str("Text 3"), W::StretchXY(15, 3));
//!
//!     // calculate the layout and set it.
//!     state.form.set_layout(form_layout.build_paged(area.as_size()));
//!  }
//!
//!  // create a FormBuffer from the parameters that will render
//!  // the individual widgets.
//!  let mut form = form
//!     .into_buffer(area, buf, &mut state.form);
//!
//!  form.render(state.text1.id(),
//!     || TextInput::new(),
//!     &mut state.text1
//!  );
//!  form.render(state.text2.id(),
//!     || TextInput::new(),
//!     &mut state.text2
//!  );
//!  form.render(state.text3.id(),
//!     || TextInput::new(),
//!     &mut state.text3
//!  );
//!
//! ```
use crate::_private::NonExhaustive;
use crate::layout::GenericLayout;
use crate::util::revert_style;
use event::FormOutcome;
use rat_event::util::MouseFlagsN;
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::cmp::min;
use std::hash::Hash;
use std::rc::Rc;
use unicode_display_width::width as unicode_width;

/// Renders other widgets using a [GenericLayout].
/// It doesn't scroll, instead it uses pages.
///
/// `Form` is the first stage and defines the layout and styling.
/// At the end call [into_buffer](Form::into_buffer) to create the
/// [FormBuffer] that allows you to render your widgets.
#[derive(Debug, Clone)]
pub struct Form<'a, W = usize>
where
    W: Eq + Hash + Clone,
{
    layout: Option<GenericLayout<W>>,

    style: Style,
    block: Option<Block<'a>>,
    nav_style: Option<Style>,
    title_style: Option<Style>,
    navigation: bool,
    next_page: &'a str,
    prev_page: &'a str,
    first_page: &'a str,
    last_page: &'a str,

    auto_label: bool,
    label_style: Option<Style>,
    label_alignment: Option<Alignment>,
}

/// Second stage of form rendering.
///
/// This can render the widgets that make up the form content.
///
#[derive(Debug)]
#[must_use]
pub struct FormBuffer<'b, W>
where
    W: Eq + Hash + Clone,
{
    layout: Rc<GenericLayout<W>>,

    page_area: Rect,
    widget_area: Rect,
    buffer: &'b mut Buffer,

    auto_label: bool,
    label_style: Option<Style>,
    label_alignment: Option<Alignment>,
}

/// All styles for a form.
#[derive(Debug, Clone)]
pub struct FormStyle {
    /// base style
    pub style: Style,
    /// label style.
    pub label_style: Option<Style>,
    /// label alignment.
    pub label_alignment: Option<Alignment>,
    /// navigation style.
    pub navigation: Option<Style>,
    /// show navigation
    pub show_navigation: Option<bool>,
    /// title style.
    pub title: Option<Style>,
    /// Block.
    pub block: Option<Block<'static>>,
    /// Navigation icon.
    pub next_page_mark: Option<&'static str>,
    /// Navigation icon.
    pub prev_page_mark: Option<&'static str>,
    /// Navigation icon.
    pub first_page_mark: Option<&'static str>,
    /// Navigation icon.
    pub last_page_mark: Option<&'static str>,

    pub non_exhaustive: NonExhaustive,
}

/// Widget state.
#[derive(Debug, Clone)]
pub struct FormState<W = usize>
where
    W: Eq + Hash + Clone,
{
    /// Page layout
    /// __read+write__ might be overwritten from widget.
    pub layout: Rc<GenericLayout<W>>,
    /// Full area for the widget.
    /// __read only__ renewed for each render.
    pub area: Rect,
    /// Area for the content.
    /// __read only__ renewed for each render.
    pub widget_area: Rect,
    /// Area for prev-page indicator.
    /// __read only__ renewed with each render.
    pub prev_area: Rect,
    /// Area for next-page indicator.
    /// __read only__ renewed with each render.
    pub next_area: Rect,

    pub page: usize,

    /// This widget has no focus of its own, but this flag
    /// can be used to set a container state.
    pub container: FocusFlag,

    /// Mouse
    pub mouse: MouseFlagsN,

    /// Only construct with `..Default::default()`.
    pub non_exhaustive: NonExhaustive,
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum FormOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Displayed page changed.
        Page,
    }

    impl ConsumedEvent for FormOutcome {
        fn is_consumed(&self) -> bool {
            *self != FormOutcome::Continue
        }
    }

    impl From<Outcome> for FormOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => FormOutcome::Continue,
                Outcome::Unchanged => FormOutcome::Unchanged,
                Outcome::Changed => FormOutcome::Changed,
            }
        }
    }

    impl From<FormOutcome> for Outcome {
        fn from(value: FormOutcome) -> Self {
            match value {
                FormOutcome::Continue => Outcome::Continue,
                FormOutcome::Unchanged => Outcome::Unchanged,
                FormOutcome::Changed => Outcome::Changed,
                FormOutcome::Page => Outcome::Changed,
            }
        }
    }
}

impl<W> Default for Form<'_, W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            style: Default::default(),
            block: Default::default(),
            nav_style: Default::default(),
            title_style: Default::default(),
            navigation: true,
            next_page: ">>>",
            prev_page: "<<<",
            first_page: " [ ",
            last_page: " ] ",
            auto_label: true,
            label_style: Default::default(),
            label_alignment: Default::default(),
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

    /// Render the label automatically when rendering the widget.
    ///
    /// Default: true
    pub fn auto_label(mut self, auto: bool) -> Self {
        self.auto_label = auto;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self.block = self.block.map(|v| v.style(style));
        self
    }

    /// Style for navigation.
    pub fn nav_style(mut self, nav_style: Style) -> Self {
        self.nav_style = Some(nav_style);
        self
    }

    /// Show navigation?
    pub fn show_navigation(mut self, show: bool) -> Self {
        self.navigation = show;
        self
    }

    /// Style for the title.
    pub fn title_style(mut self, title_style: Style) -> Self {
        self.title_style = Some(title_style);
        self
    }

    /// Block for border
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block.style(self.style));
        self
    }

    pub fn next_page_mark(mut self, txt: &'a str) -> Self {
        self.next_page = txt;
        self
    }

    pub fn prev_page_mark(mut self, txt: &'a str) -> Self {
        self.prev_page = txt;
        self
    }

    pub fn first_page_mark(mut self, txt: &'a str) -> Self {
        self.first_page = txt;
        self
    }

    pub fn last_page_mark(mut self, txt: &'a str) -> Self {
        self.last_page = txt;
        self
    }

    /// Style for auto-labels.
    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = Some(style);
        self
    }

    /// Alignment for auto-labels.
    pub fn label_alignment(mut self, alignment: Alignment) -> Self {
        self.label_alignment = Some(alignment);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: FormStyle) -> Self {
        self.style = styles.style;
        if let Some(nav) = styles.navigation {
            self.nav_style = Some(nav);
        }
        if let Some(navigation) = styles.show_navigation {
            self.navigation = navigation;
        }
        if let Some(title) = styles.title {
            self.title_style = Some(title);
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        if let Some(txt) = styles.next_page_mark {
            self.next_page = txt;
        }
        if let Some(txt) = styles.prev_page_mark {
            self.prev_page = txt;
        }
        if let Some(txt) = styles.first_page_mark {
            self.first_page = txt;
        }
        if let Some(txt) = styles.last_page_mark {
            self.last_page = txt;
        }
        self.block = self.block.map(|v| v.style(styles.style));

        if let Some(label) = styles.label_style {
            self.label_style = Some(label);
        }
        if let Some(alignment) = styles.label_alignment {
            self.label_alignment = Some(alignment);
        }

        self
    }

    /// Calculate the layout page size.
    pub fn layout_size(&self, area: Rect) -> Size {
        self.block.inner_if_some(area).as_size()
    }

    // Calculate the view area for all columns.
    pub fn layout_area(&self, area: Rect) -> Rect {
        if let Some(block) = &self.block
            && self.navigation
        {
            block.inner(area)
        } else {
            area
        }
    }

    /// Render the page navigation and create the FormBuffer
    /// that will do the actual rendering.
    #[allow(clippy::needless_lifetimes)]
    pub fn into_buffer<'b, 's>(
        mut self,
        area: Rect,
        buf: &'b mut Buffer,
        state: &'s mut FormState<W>,
    ) -> FormBuffer<'b, W> {
        state.area = area;
        state.widget_area = self.layout_area(area);

        if let Some(layout) = self.layout.take() {
            state.layout = Rc::new(layout);
        }

        let page_size = state.layout.page_size();
        assert!(page_size.height < u16::MAX || page_size.height == u16::MAX && state.page == 0);
        let page_area = Rect::new(
            0,
            (state.page as u16).saturating_mul(page_size.height),
            page_size.width,
            page_size.height,
        );

        if self.navigation {
            self.render_navigation(area, buf, state);
        } else {
            buf.set_style(area, self.style);
        }

        let mut form_buf = FormBuffer {
            layout: state.layout.clone(),
            page_area,
            widget_area: state.widget_area,
            buffer: buf,

            auto_label: true,
            label_style: self.label_style,
            label_alignment: self.label_alignment,
        };
        form_buf.render_block();
        form_buf
    }

    fn render_navigation(&self, area: Rect, buf: &mut Buffer, state: &mut FormState<W>) {
        let page_count = state.layout.page_count();

        if !state.layout.is_endless() {
            if state.page > 0 {
                state.prev_area =
                    Rect::new(area.x, area.y, unicode_width(self.prev_page) as u16, 1);
            } else {
                state.prev_area =
                    Rect::new(area.x, area.y, unicode_width(self.first_page) as u16, 1);
            }
            if (state.page + 1) < page_count {
                let p = unicode_width(self.next_page) as u16;
                state.next_area = Rect::new(area.x + area.width.saturating_sub(p), area.y, p, 1);
            } else {
                let p = unicode_width(self.last_page) as u16;
                state.next_area = Rect::new(area.x + area.width.saturating_sub(p), area.y, p, 1);
            }
        } else {
            state.prev_area = Default::default();
            state.next_area = Default::default();
        }

        let block = if page_count > 1 {
            let title = format!(" {}/{} ", state.page + 1, page_count);
            let block = self
                .block
                .clone()
                .unwrap_or_else(|| Block::new().style(self.style))
                .title_bottom(title)
                .title_alignment(Alignment::Right);
            if let Some(title_style) = self.title_style {
                block.title_style(title_style)
            } else {
                block
            }
        } else {
            self.block
                .clone()
                .unwrap_or_else(|| Block::new().style(self.style))
        };
        block.render(area, buf);

        if !state.layout.is_endless() {
            // active areas
            let nav_style = self.nav_style.unwrap_or(self.style);
            if matches!(state.mouse.hover.get(), Some(0)) {
                buf.set_style(state.prev_area, revert_style(nav_style));
            } else {
                buf.set_style(state.prev_area, nav_style);
            }
            if state.page > 0 {
                Span::from(self.prev_page).render(state.prev_area, buf);
            } else {
                Span::from(self.first_page).render(state.prev_area, buf);
            }
            if matches!(state.mouse.hover.get(), Some(1)) {
                buf.set_style(state.next_area, revert_style(nav_style));
            } else {
                buf.set_style(state.next_area, nav_style);
            }
            if (state.page + 1) < page_count {
                Span::from(self.next_page).render(state.next_area, buf);
            } else {
                Span::from(self.last_page).render(state.next_area, buf);
            }
        }
    }
}

impl<'b, W> FormBuffer<'b, W>
where
    W: Eq + Hash + Clone,
{
    /// Is the given area visible?
    pub fn is_visible(&self, widget: W) -> bool {
        if let Some(idx) = self.layout.try_index_of(widget) {
            self.locate_area(self.layout.widget(idx)).is_some()
        } else {
            false
        }
    }

    /// Render all blocks for the current page.
    fn render_block(&mut self) {
        for (idx, block_area) in self.layout.block_area_iter().enumerate() {
            if let Some(block_area) = self.locate_area(*block_area) {
                if let Some(block) = self.layout.block(idx) {
                    block.render(block_area, self.buffer);
                }
            }
        }
    }

    /// Render a manual label.
    #[inline(always)]
    pub fn render_label<FN>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce(&Cow<'static, str>, Rect, &mut Buffer),
    {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return false;
        };
        let Some(label_area) = self.locate_area(self.layout.label(idx)) else {
            return false;
        };
        if let Some(label_str) = self.layout.try_label_str(idx) {
            render_fn(label_str, label_area, self.buffer);
        } else {
            render_fn(&Cow::default(), label_area, self.buffer);
        }
        true
    }

    /// Render a stateless widget and its label, if any.
    #[inline(always)]
    pub fn render_widget<FN, WW>(&mut self, widget: W, render_fn: FN) -> bool
    where
        FN: FnOnce() -> WW,
        WW: Widget,
    {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return false;
        };
        if self.auto_label {
            self.render_auto_label(idx);
        }

        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            return false;
        };
        render_fn().render(widget_area, self.buffer);
        true
    }

    /// Render an optional stateful widget and its label, if any.
    #[inline(always)]
    pub fn render_opt<FN, WW, SS>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> bool
    where
        FN: FnOnce() -> Option<WW>,
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return false;
        };
        if self.auto_label {
            self.render_auto_label(idx);
        }
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            state.relocate_hidden();
            return false;
        };
        let widget = render_fn();
        if let Some(widget) = widget {
            widget.render(widget_area, self.buffer, state);
            true
        } else {
            state.relocate_hidden();
            false
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
        let Some(idx) = self.layout.try_index_of(widget) else {
            return false;
        };
        if self.auto_label {
            self.render_auto_label(idx);
        }
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            state.relocate_hidden();
            return false;
        };
        let widget = render_fn();
        widget.render(widget_area, self.buffer, state);
        true
    }

    /// Render a stateful widget and its label, if any.
    /// The closure can return a second value, which will be returned
    /// if the widget is visible.
    #[inline(always)]
    #[allow(clippy::question_mark)]
    pub fn render2<FN, WW, SS, R>(&mut self, widget: W, render_fn: FN, state: &mut SS) -> Option<R>
    where
        FN: FnOnce() -> (WW, R),
        WW: StatefulWidget<State = SS>,
        SS: RelocatableState,
    {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return None;
        };
        if self.auto_label {
            self.render_auto_label(idx);
        }
        let Some(widget_area) = self.locate_area(self.layout.widget(idx)) else {
            state.relocate_hidden();
            return None;
        };
        let (widget, remainder) = render_fn();
        widget.render(widget_area, self.buffer, state);

        Some(remainder)
    }

    /// Get access to the buffer during rendering a page.
    pub fn buffer(&mut self) -> &mut Buffer {
        self.buffer
    }

    /// Render the label with the set style and alignment.
    #[inline(always)]
    fn render_auto_label(&mut self, idx: usize) -> bool {
        let Some(label_area) = self.locate_area(self.layout.label(idx)) else {
            return false;
        };
        let Some(label_str) = self.layout.try_label_str(idx) else {
            return false;
        };
        let mut label = Line::from(label_str.as_ref());
        if let Some(style) = self.label_style {
            label = label.style(style)
        };
        if let Some(align) = self.label_alignment {
            label = label.alignment(align);
        }
        label.render(label_area, self.buffer);

        true
    }

    /// Get the area for the given widget.
    pub fn locate_widget(&self, widget: W) -> Option<Rect> {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return None;
        };
        self.locate_area(self.layout.widget(idx))
    }

    /// Get the area for the label of the given widget.
    pub fn locate_label(&self, widget: W) -> Option<Rect> {
        let Some(idx) = self.layout.try_index_of(widget) else {
            return None;
        };
        self.locate_area(self.layout.label(idx))
    }

    /// This will clip the area to the page_area.
    #[inline]
    pub fn locate_area(&self, area: Rect) -> Option<Rect> {
        // clip to page
        let area = self.page_area.intersection(area);
        if self.page_area.intersects(area) {
            let located = Rect::new(
                area.x - self.page_area.x + self.widget_area.x,
                area.y - self.page_area.y + self.widget_area.y,
                area.width,
                area.height,
            );
            // clip to render area
            let located = self.widget_area.intersection(located);
            if self.widget_area.intersects(located) {
                Some(located)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for FormStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            label_style: None,
            label_alignment: None,
            navigation: None,
            show_navigation: None,
            title: None,
            block: None,
            next_page_mark: None,
            prev_page_mark: None,
            first_page_mark: None,
            last_page_mark: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W> Default for FormState<W>
where
    W: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            layout: Default::default(),
            area: Default::default(),
            widget_area: Default::default(),
            prev_area: Default::default(),
            next_area: Default::default(),
            page: 0,
            container: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<W> HasFocus for FormState<W>
where
    W: Eq + Hash + Clone,
{
    fn build(&self, _builder: &mut FocusBuilder) {
        // no content.
    }

    fn focus(&self) -> FocusFlag {
        self.container.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<W> FormState<W>
where
    W: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.container = FocusFlag::named(name);
        z
    }

    /// Clear the layout data and reset the page/page-count.
    pub fn clear(&mut self) {
        self.layout = Default::default();
        self.page = 0;
    }

    /// Layout needs to change?
    pub fn valid_layout(&self, size: Size) -> bool {
        !self.layout.size_changed(size) && !self.layout.is_empty()
    }

    /// Set the layout.
    pub fn set_layout(&mut self, layout: GenericLayout<W>) {
        self.layout = Rc::new(layout);
    }

    /// Layout.
    pub fn layout(&self) -> Rc<GenericLayout<W>> {
        self.layout.clone()
    }

    /// Show the page for this widget.
    /// If there is no widget for the given identifier, this
    /// will set the page to 0.
    pub fn show(&mut self, widget: W) {
        let page = self.layout.page_of(widget).unwrap_or_default();
        self.set_page(page);
    }

    /// Number of form pages.
    pub fn page_count(&self) -> usize {
        self.layout.page_count()
    }

    /// Returns the first widget for the given page.
    pub fn first(&self, page: usize) -> Option<W> {
        self.layout.first(page)
    }

    /// Calculates the page of the widget.
    pub fn page_of(&self, widget: W) -> Option<usize> {
        self.layout.page_of(widget)
    }

    /// Set the visible page.
    pub fn set_page(&mut self, page: usize) -> bool {
        let old_page = self.page;
        self.page = min(page, self.page_count().saturating_sub(1));
        old_page != self.page
    }

    /// Visible page
    pub fn page(&self) -> usize {
        self.page
    }

    /// Select next page. Keeps the page in bounds.
    pub fn next_page(&mut self) -> bool {
        let old_page = self.page;

        if self.page + 1 == self.page_count() {
            // don't change
        } else if self.page + 1 > self.page_count() {
            self.page = self.page_count().saturating_sub(1);
        } else {
            self.page += 1;
        }

        old_page != self.page
    }

    /// Select prev page.
    pub fn prev_page(&mut self) -> bool {
        if self.page >= 1 {
            self.page -= 1;
            true
        } else if self.page > 0 {
            self.page = 0;
            true
        } else {
            false
        }
    }
}

impl FormState<usize> {
    /// Focus the first widget on the active page.
    /// This assumes the usize-key is a widget id.
    pub fn focus_first(&self, focus: &Focus) -> bool {
        if let Some(w) = self.first(self.page) {
            focus.by_widget_id(w);
            true
        } else {
            false
        }
    }

    /// Show the page with the focused widget.
    /// This assumes the usize-key is a widget id.
    /// Does nothing if none of the widgets has the focus.
    pub fn show_focused(&mut self, focus: &Focus) -> bool {
        let Some(focused) = focus.focused() else {
            return false;
        };
        let focused = focused.widget_id();
        let page = self.layout.page_of(focused);
        if let Some(page) = page {
            self.set_page(page);
            true
        } else {
            false
        }
    }
}

impl FormState<FocusFlag> {
    /// Focus the first widget on the active page.
    pub fn focus_first(&self, focus: &Focus) -> bool {
        if let Some(w) = self.first(self.page) {
            focus.focus(&w);
            true
        } else {
            false
        }
    }

    /// Show the page with the focused widget.
    /// Does nothing if none of the widgets has the focus.
    pub fn show_focused(&mut self, focus: &Focus) -> bool {
        let Some(focused) = focus.focused() else {
            return false;
        };
        let page = self.layout.page_of(focused);
        if let Some(page) = page {
            self.set_page(page);
            true
        } else {
            false
        }
    }
}

impl<W> HandleEvent<crossterm::event::Event, Regular, FormOutcome> for FormState<W>
where
    W: Eq + Hash + Clone,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> FormOutcome {
        let r = if self.container.is_focused() && !self.layout.is_endless() {
            match event {
                ct_event!(keycode press ALT-PageUp) => {
                    if self.prev_page() {
                        FormOutcome::Page
                    } else {
                        FormOutcome::Continue
                    }
                }
                ct_event!(keycode press ALT-PageDown) => {
                    if self.next_page() {
                        FormOutcome::Page
                    } else {
                        FormOutcome::Continue
                    }
                }
                _ => FormOutcome::Continue,
            }
        } else {
            FormOutcome::Continue
        };

        r.or_else(|| self.handle(event, MouseOnly))
    }
}

impl<W> HandleEvent<crossterm::event::Event, MouseOnly, FormOutcome> for FormState<W>
where
    W: Eq + Hash + Clone,
{
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> FormOutcome {
        if !self.layout.is_endless() {
            match event {
                ct_event!(mouse down Left for x,y) if self.prev_area.contains((*x, *y).into()) => {
                    if self.prev_page() {
                        FormOutcome::Page
                    } else {
                        FormOutcome::Unchanged
                    }
                }
                ct_event!(mouse down Left for x,y) if self.next_area.contains((*x, *y).into()) => {
                    if self.next_page() {
                        FormOutcome::Page
                    } else {
                        FormOutcome::Unchanged
                    }
                }
                ct_event!(scroll down for x,y) => {
                    if self.area.contains((*x, *y).into()) {
                        if self.next_page() {
                            FormOutcome::Page
                        } else {
                            FormOutcome::Continue
                        }
                    } else {
                        FormOutcome::Continue
                    }
                }
                ct_event!(scroll up for x,y) => {
                    if self.area.contains((*x, *y).into()) {
                        if self.prev_page() {
                            FormOutcome::Page
                        } else {
                            FormOutcome::Continue
                        }
                    } else {
                        FormOutcome::Continue
                    }
                }
                ct_event!(mouse any for m)
                    if self.mouse.hover(&[self.prev_area, self.next_area], m) =>
                {
                    FormOutcome::Changed
                }
                _ => FormOutcome::Continue,
            }
        } else {
            FormOutcome::Continue
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events<W>(
    state: &mut FormState<W>,
    _focus: bool,
    event: &crossterm::event::Event,
) -> FormOutcome
where
    W: Eq + Clone + Hash,
{
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events<W>(
    state: &mut FormState<W>,
    event: &crossterm::event::Event,
) -> FormOutcome
where
    W: Eq + Clone + Hash,
{
    HandleEvent::handle(state, event, MouseOnly)
}
