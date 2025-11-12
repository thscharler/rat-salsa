//! Render tabs.
//!
//! This widget doesn't render the content.
//! Use [TabbedState::widget_area] to render the selected tab.
//!
//! ```
//! use ratatui::buffer::Buffer;
//! use ratatui::layout::Rect;
//! use ratatui::text::Text;
//! use ratatui::widgets::{Block, StatefulWidget, Widget};
//! use rat_widget::tabbed::{TabPlacement, TabType, Tabbed, TabbedState};
//! # struct State { tabbed: TabbedState }
//! # let mut state = State { tabbed: Default::default() };
//! # let mut buf = Buffer::default();
//! # let buf = &mut buf;
//! # let area = Rect::default();
//!
//! let mut tab = Tabbed::new()
//!     .tab_type(TabType::Attached)
//!     .placement(TabPlacement::Top)
//!     .closeable(true)
//!     .block(
//!          Block::bordered()
//!     )
//!     .tabs(["Issues", "Numbers", "More numbers"])
//!     .render(area, buf, &mut state.tabbed);
//!
//! match state.tabbed.selected() {
//!     Some(0) => {
//!         Text::from("... issues ...")
//!             .render(state.tabbed.widget_area, buf);
//!     }
//!     Some(1) => {
//!         Text::from("... 1,2,3,4 ...")
//!             .render(state.tabbed.widget_area, buf);
//!     }
//!     Some(1) => {
//!         Text::from("... 5,6,7,8 ...")
//!             .render(state.tabbed.widget_area, buf);
//!     }
//!     _ => {}
//! }
//!
//! ```
//!
use crate::_private::NonExhaustive;
use crate::event::TabbedOutcome;
use crate::tabbed::attached::AttachedTabs;
use crate::tabbed::glued::GluedTabs;
use rat_event::util::MouseFlagsN;
use rat_event::{HandleEvent, MouseOnly, Regular, ct_event, flow};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_reloc::{RelocatableState, relocate_area, relocate_areas};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget};
use std::cmp::min;
use std::fmt::Debug;

mod attached;
mod glued;

/// Placement relative to the Rect given to render.
///
/// The popup-menu is always rendered outside the box,
/// and this gives the relative placement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TabPlacement {
    /// On top of the given area. Placed slightly left, so that
    /// the menu text aligns with the left border.
    #[default]
    Top,
    /// Placed left-top of the given area.
    /// For a submenu opening to the left.
    Left,
    /// Placed right-top of the given area.
    /// For a submenu opening to the right.
    Right,
    /// Below the bottom of the given area. Placed slightly left,
    /// so that the menu text aligns with the left border.
    Bottom,
}

/// Rendering style for the tabs.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TabType {
    /// Basic tabs glued to the outside of the widget.
    Glued,

    /// Embedded tabs in the Block.
    ///
    /// If no block has been set, this will draw a block at the side
    /// of the tabs.
    ///
    /// On the left/right side this will just draw a link to the tab-text.
    /// On the top/bottom side the tabs will be embedded in the border.
    #[default]
    Attached,
}

/// A tabbed widget.
///
/// This widget draws the tabs and handles events.
///
/// Use [TabbedState::selected] and [TabbedState::widget_area] to render
/// the actual content of the tab.
///
#[derive(Debug, Default, Clone)]
pub struct Tabbed<'a> {
    tab_type: TabType,
    placement: TabPlacement,
    closeable: bool,
    tabs: Vec<Line<'a>>,
    block: Option<Block<'a>>,

    style: Style,
    tab_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
}

/// Combined Styles
#[derive(Debug, Clone)]
pub struct TabbedStyle {
    pub style: Style,
    pub tab: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,

    pub tab_type: Option<TabType>,
    pub placement: Option<TabPlacement>,
    pub block: Option<Block<'static>>,

    pub non_exhaustive: NonExhaustive,
}

/// State & event-handling.
#[derive(Debug, Default)]
pub struct TabbedState {
    /// Total area.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area for drawing the Block inside the tabs.
    /// __readonly__. renewed for each render.
    pub block_area: Rect,
    /// Area used to render the content of the tab.
    /// Use this area to render the current tab content.
    /// __readonly__. renewed for each render.
    pub widget_area: Rect,

    /// Total area reserved for tabs.
    /// __readonly__. renewed for each render.
    pub tab_title_area: Rect,
    /// Area of each tab.
    /// __readonly__. renewed for each render.
    pub tab_title_areas: Vec<Rect>,
    /// Area for 'Close Tab' interaction.
    /// __readonly__. renewed for each render.
    pub tab_title_close_areas: Vec<Rect>,

    /// Selected Tab, only ever is None if there are no tabs.
    /// Otherwise, set to 0 on render.
    /// __read+write___
    pub selected: Option<usize>,

    /// Focus
    /// __read+write__
    pub focus: FocusFlag,
    /// Mouse flags
    /// __read+write__
    pub mouse: MouseFlagsN,
}

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum TabbedOutcome {
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
        /// Tab selection changed.
        Select(usize),
        /// Selected tab should be closed.
        Close(usize),
    }

    impl ConsumedEvent for TabbedOutcome {
        fn is_consumed(&self) -> bool {
            *self != TabbedOutcome::Continue
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for TabbedOutcome {
        fn from(value: bool) -> Self {
            if value {
                TabbedOutcome::Changed
            } else {
                TabbedOutcome::Unchanged
            }
        }
    }

    impl From<Outcome> for TabbedOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => TabbedOutcome::Continue,
                Outcome::Unchanged => TabbedOutcome::Unchanged,
                Outcome::Changed => TabbedOutcome::Changed,
            }
        }
    }

    impl From<TabbedOutcome> for Outcome {
        fn from(value: TabbedOutcome) -> Self {
            match value {
                TabbedOutcome::Continue => Outcome::Continue,
                TabbedOutcome::Unchanged => Outcome::Unchanged,
                TabbedOutcome::Changed => Outcome::Changed,
                TabbedOutcome::Select(_) => Outcome::Changed,
                TabbedOutcome::Close(_) => Outcome::Changed,
            }
        }
    }
}

impl<'a> Tabbed<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tab type.
    pub fn tab_type(mut self, tab_type: TabType) -> Self {
        self.tab_type = tab_type;
        self
    }

    /// Tab placement.
    pub fn placement(mut self, placement: TabPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Tab-text.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = impl Into<Line<'a>>>) -> Self {
        self.tabs = tabs.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        self
    }

    /// Closeable tabs?
    ///
    /// Renders a close symbol and reacts with [TabbedOutcome::Close].
    pub fn closeable(mut self, closeable: bool) -> Self {
        self.closeable = closeable;
        self
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set combined styles.
    pub fn styles(mut self, styles: TabbedStyle) -> Self {
        self.style = styles.style;
        if styles.tab.is_some() {
            self.tab_style = styles.tab;
        }
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if let Some(tab_type) = styles.tab_type {
            self.tab_type = tab_type;
        }
        if let Some(placement) = styles.placement {
            self.placement = placement
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        self
    }

    /// Base style. Mostly for any background.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for the tab-text.
    pub fn tab_style(mut self, style: Style) -> Self {
        self.tab_style = Some(style);
        self
    }

    /// Style for the selected tab.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Style for a focused tab.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }
}

impl Default for TabbedStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            tab: None,
            select: None,
            focus: None,
            tab_type: None,
            placement: None,
            block: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl StatefulWidget for Tabbed<'_> {
    type State = TabbedState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

impl<'a> StatefulWidget for &Tabbed<'a> {
    type State = TabbedState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

fn render_ref(tabbed: &Tabbed<'_>, area: Rect, buf: &mut Buffer, state: &mut TabbedState) {
    if tabbed.tabs.is_empty() {
        state.selected = None;
    } else {
        if state.selected.is_none() {
            state.selected = Some(0);
        }
    }

    match tabbed.tab_type {
        TabType::Glued => {
            GluedTabs.layout(area, tabbed, state);
            GluedTabs.render(buf, tabbed, state);
        }
        TabType::Attached => {
            AttachedTabs.layout(area, tabbed, state);
            AttachedTabs.render(buf, tabbed, state);
        }
    }
}

impl Clone for TabbedState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            block_area: self.block_area,
            widget_area: self.widget_area,
            tab_title_area: self.tab_title_area,
            tab_title_areas: self.tab_title_areas.clone(),
            tab_title_close_areas: self.tab_title_close_areas.clone(),
            selected: self.selected,
            focus: self.focus.new_instance(),
            mouse: Default::default(),
        }
    }
}

impl HasFocus for TabbedState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }

    fn navigable(&self) -> Navigation {
        Navigation::Leave
    }
}

impl RelocatableState for TabbedState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area = relocate_area(self.area, shift, clip);
        self.block_area = relocate_area(self.block_area, shift, clip);
        self.widget_area = relocate_area(self.widget_area, shift, clip);
        self.tab_title_area = relocate_area(self.tab_title_area, shift, clip);
        relocate_areas(self.tab_title_areas.as_mut(), shift, clip);
        relocate_areas(self.tab_title_close_areas.as_mut(), shift, clip);
    }
}

impl TabbedState {
    /// New initial state.
    pub fn new() -> Self {
        Default::default()
    }

    /// State with a focus name.
    pub fn named(name: &str) -> Self {
        let mut z = Self::default();
        z.focus = z.focus.with_name(name);
        z
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, selected: Option<usize>) {
        self.selected = selected;
    }

    /// Selects the next tab. Stops at the end.
    pub fn next_tab(&mut self) -> bool {
        let old_selected = self.selected;

        if let Some(selected) = self.selected() {
            self.selected = Some(min(
                selected + 1,
                self.tab_title_areas.len().saturating_sub(1),
            ));
        }

        old_selected != self.selected
    }

    /// Selects the previous tab. Stops at the end.
    pub fn prev_tab(&mut self) -> bool {
        let old_selected = self.selected;

        if let Some(selected) = self.selected() {
            if selected > 0 {
                self.selected = Some(selected - 1);
            }
        }

        old_selected != self.selected
    }
}

/// Handle the regular events for Tabbed.
impl HandleEvent<crossterm::event::Event, Regular, TabbedOutcome> for TabbedState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> TabbedOutcome {
        if self.is_focused() {
            flow!(match event {
                ct_event!(keycode press Left) => self.prev_tab().into(),
                ct_event!(keycode press Right) => self.next_tab().into(),
                ct_event!(keycode press Up) => self.prev_tab().into(),
                ct_event!(keycode press Down) => self.next_tab().into(),
                _ => TabbedOutcome::Continue,
            });
        }

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, TabbedOutcome> for TabbedState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> TabbedOutcome {
        match event {
            ct_event!(mouse any for e) if self.mouse.hover(&self.tab_title_close_areas, e) => {
                TabbedOutcome::Changed
            }
            ct_event!(mouse any for e) if self.mouse.drag(&[self.tab_title_area], e) => {
                if let Some(n) = self.mouse.item_at(&self.tab_title_areas, e.column, e.row) {
                    self.select(Some(n));
                    TabbedOutcome::Select(n)
                } else {
                    TabbedOutcome::Unchanged
                }
            }
            ct_event!(mouse down Left for x, y)
                if self.tab_title_area.contains((*x, *y).into()) =>
            {
                if let Some(sel) = self.mouse.item_at(&self.tab_title_close_areas, *x, *y) {
                    TabbedOutcome::Close(sel)
                } else if let Some(sel) = self.mouse.item_at(&self.tab_title_areas, *x, *y) {
                    self.select(Some(sel));
                    TabbedOutcome::Select(sel)
                } else {
                    TabbedOutcome::Continue
                }
            }

            _ => TabbedOutcome::Continue,
        }
    }
}

/// The design space for tabs is too big to capture with a handful of parameters.
///
/// This trait splits off the layout and rendering of the actual tabs from
/// the general properties and behaviour of tabs.
trait TabWidget: Debug {
    /// Calculate the layout for the tabs.
    fn layout(
        &self, //
        area: Rect,
        tabbed: &Tabbed<'_>,
        state: &mut TabbedState,
    );

    /// Render the tabs.
    fn render(
        &self, //
        buf: &mut Buffer,
        tabbed: &Tabbed<'_>,
        state: &mut TabbedState,
    );
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TabbedState,
    focus: bool,
    event: &crossterm::event::Event,
) -> TabbedOutcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut TabbedState,
    event: &crossterm::event::Event,
) -> TabbedOutcome {
    HandleEvent::handle(state, event, MouseOnly)
}
