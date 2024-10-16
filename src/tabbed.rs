//!
//! Tabs.
//!
use crate::_private::NonExhaustive;
use crate::event::TabbedOutcome;
use crate::tabbed::attached::AttachedTabs;
use crate::tabbed::glued::GluedTabs;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocus, Navigation};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget};
use std::cmp::min;
use std::fmt::Debug;

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
#[derive(Debug, Default)]
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
        self.tab_style = styles.tab_style;
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
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

/// Combined Styles
#[derive(Debug, Clone)]
pub struct TabbedStyle {
    pub style: Style,
    pub tab_style: Option<Style>,
    pub select_style: Option<Style>,
    pub focus_style: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for TabbedStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            tab_style: None,
            select_style: None,
            focus_style: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

/// State & event-handling.
#[derive(Debug, Default, Clone)]
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

impl<'a> StatefulWidget for Tabbed<'a> {
    type State = TabbedState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for Tabbed<'a> {
    type State = TabbedState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

impl HasFocus for TabbedState {
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

impl TabbedState {
    /// New initial state.
    pub fn new() -> Self {
        Default::default()
    }

    /// State with a focus name.
    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Default::default()
        }
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
                ct_event!(keycode press Right) => self.next_tab().into(),
                ct_event!(keycode press Left) => self.prev_tab().into(),
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
            ct_event!(mouse down Left for x, y) => {
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

/// Simple glued on the side tabs.
mod glued {
    use crate::tabbed::{TabPlacement, TabWidget, Tabbed, TabbedState};
    use crate::util::revert_style;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
    use ratatui::widgets::Widget;

    /// Renders simple tabs at the given placement and renders
    /// the block inside the tabs.
    #[derive(Debug, Default)]
    pub(super) struct GluedTabs;

    impl TabWidget for GluedTabs {
        fn layout(&self, area: Rect, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
            state.area = area;

            let margin_offset = 1 + if tabbed.block.is_some() { 1 } else { 0 };
            let close_width = if tabbed.closeable { 2 } else { 0 };

            let max_width = tabbed
                .tabs
                .iter()
                .map(|v| v.width())
                .max()
                .unwrap_or_default() as u16;

            match tabbed.placement {
                TabPlacement::Top => {
                    state.block_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
                    state.tab_title_area = Rect::new(area.x, area.y, area.width, 1);
                    if let Some(block) = &tabbed.block {
                        state.widget_area = block.inner(state.block_area);
                    } else {
                        state.widget_area = state.block_area;
                    }
                }
                TabPlacement::Bottom => {
                    state.block_area =
                        Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
                    state.tab_title_area = Rect::new(
                        area.x,
                        area.y + area.height.saturating_sub(1),
                        area.width,
                        1,
                    );
                    if let Some(block) = &tabbed.block {
                        state.widget_area = block.inner(state.block_area);
                    } else {
                        state.widget_area = state.block_area;
                    }
                }
                TabPlacement::Left => {
                    state.block_area = Rect::new(
                        area.x + max_width + 2 + close_width,
                        area.y,
                        area.width - (max_width + 2 + close_width),
                        area.height,
                    );
                    state.tab_title_area =
                        Rect::new(area.x, area.y, max_width + 2 + close_width, area.height);
                    if let Some(block) = &tabbed.block {
                        state.widget_area = block.inner(state.block_area);
                    } else {
                        state.widget_area = state.block_area;
                    }
                }
                TabPlacement::Right => {
                    state.block_area = Rect::new(
                        area.x,
                        area.y,
                        area.width - (max_width + 2 + close_width),
                        area.height,
                    );
                    state.tab_title_area = Rect::new(
                        area.x + area.width - (max_width + 2 + close_width),
                        area.y,
                        max_width + 2 + close_width,
                        area.height,
                    );
                    if let Some(block) = &tabbed.block {
                        state.widget_area = block.inner(state.block_area);
                    } else {
                        state.widget_area = state.block_area;
                    }
                }
            }

            match tabbed.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    let mut constraints = Vec::new();
                    for tab in tabbed.tabs.iter() {
                        constraints.push(Constraint::Length(tab.width() as u16 + 2 + close_width));
                    }

                    state.tab_title_areas = Vec::from(
                        Layout::horizontal(constraints)
                            .flex(Flex::Start)
                            .spacing(1)
                            .horizontal_margin(margin_offset)
                            .split(state.tab_title_area)
                            .as_ref(),
                    );
                }
                TabPlacement::Left | TabPlacement::Right => {
                    let mut constraints = Vec::new();
                    for _tab in tabbed.tabs.iter() {
                        constraints.push(Constraint::Length(1));
                    }

                    state.tab_title_areas = Vec::from(
                        Layout::vertical(constraints)
                            .flex(Flex::Start)
                            .vertical_margin(margin_offset)
                            .split(state.tab_title_area)
                            .as_ref(),
                    );
                }
            }

            match tabbed.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                (v.x + v.width).saturating_sub(close_width),
                                v.y,
                                if tabbed.closeable { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
                TabPlacement::Left => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                v.x + 1, //
                                v.y,
                                if tabbed.closeable { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
                TabPlacement::Right => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                (v.x + v.width).saturating_sub(close_width),
                                v.y,
                                if tabbed.closeable { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
            }
        }

        fn render(&self, buf: &mut Buffer, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
            let focus_style = if let Some(focus_style) = tabbed.focus_style {
                focus_style
            } else {
                revert_style(tabbed.style)
            };
            let select_style = if let Some(select_style) = tabbed.select_style {
                if state.focus.get() {
                    focus_style
                } else {
                    select_style
                }
            } else {
                if state.focus.get() {
                    focus_style
                } else {
                    revert_style(tabbed.style)
                }
            };
            let tab_style = if let Some(tab_style) = tabbed.tab_style {
                tab_style
            } else {
                tabbed.style
            };

            buf.set_style(state.tab_title_area, tabbed.style);
            tabbed.block.clone().render(state.block_area, buf);

            for (idx, tab_area) in state.tab_title_areas.iter().copied().enumerate() {
                if Some(idx) == state.selected() {
                    buf.set_style(tab_area, select_style);
                } else {
                    buf.set_style(tab_area, tab_style);
                }

                let txt_area = match tabbed.placement {
                    TabPlacement::Top | TabPlacement::Right | TabPlacement::Bottom => {
                        tab_area.inner(Margin::new(1, 0))
                    }
                    TabPlacement::Left => {
                        if tabbed.closeable {
                            Rect::new(
                                tab_area.x + 3,
                                tab_area.y,
                                tab_area.width - 4,
                                tab_area.height,
                            )
                        } else {
                            tab_area.inner(Margin::new(1, 0))
                        }
                    }
                };
                tabbed.tabs[idx].clone().render(txt_area, buf);
            }
            if tabbed.closeable {
                for i in 0..state.tab_title_close_areas.len() {
                    if state.mouse.hover.get() == Some(i) {
                        buf.set_style(state.tab_title_close_areas[i], revert_style(tab_style));
                    }
                    if let Some(cell) = buf.cell_mut(state.tab_title_close_areas[i].as_position()) {
                        cell.set_symbol("\u{2A2F}");
                    }
                }
            }
        }
    }
}

/// Tabs embedded in the Block.
///
/// If no block has been set, this will draw a block at the side
/// of the tabs.
mod attached {
    use crate::tabbed::{TabPlacement, TabWidget, Tabbed, TabbedState};
    use crate::util;
    use crate::util::{block_left, block_right, fill_buf_area, revert_style};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
    use ratatui::symbols::line;
    use ratatui::widgets::{Block, BorderType, Borders, Widget};

    /// Embedded tabs in the Block.
    ///
    /// If no block has been set, this will draw a block at the side
    /// of the tabs.
    ///
    /// On the left/right side this will just draw a link to the tab-text.
    /// On the top/bottom side the tabs will be embedded in the border.
    #[derive(Debug)]
    pub(super) struct AttachedTabs;

    impl AttachedTabs {
        fn get_link(&self, placement: TabPlacement, block: &Block<'_>) -> Option<&str> {
            match placement {
                TabPlacement::Top => unreachable!(),
                TabPlacement::Bottom => unreachable!(),
                TabPlacement::Left => match block_left(block).as_str() {
                    line::VERTICAL => Some(line::VERTICAL_LEFT),
                    line::DOUBLE_VERTICAL => Some(util::DOUBLE_VERTICAL_SINGLE_LEFT),
                    line::THICK_VERTICAL => Some(util::THICK_VERTICAL_SINGLE_LEFT),

                    _ => None,
                },
                TabPlacement::Right => match block_right(block).as_str() {
                    line::VERTICAL => Some(line::VERTICAL_RIGHT),
                    line::DOUBLE_VERTICAL => Some(util::DOUBLE_VERTICAL_SINGLE_RIGHT),
                    line::THICK_VERTICAL => Some(util::THICK_VERTICAL_SINGLE_RIGHT),
                    _ => None,
                },
            }
        }
    }

    impl TabWidget for AttachedTabs {
        fn layout(&self, area: Rect, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
            state.area = area;

            let mut block;
            let (block, block_offset) = if let Some(block) = &tabbed.block {
                (block, 1)
            } else {
                block = Block::new();
                block = match tabbed.placement {
                    TabPlacement::Top => block.borders(Borders::TOP),
                    TabPlacement::Left => block.borders(Borders::LEFT),
                    TabPlacement::Right => block.borders(Borders::RIGHT),
                    TabPlacement::Bottom => block.borders(Borders::BOTTOM),
                };
                (&block, 0)
            };

            let close_width = if tabbed.closeable { 2 } else { 0 };

            let max_width = tabbed
                .tabs
                .iter()
                .map(|v| v.width())
                .max()
                .unwrap_or_default() as u16;

            match tabbed.placement {
                TabPlacement::Top => {
                    state.block_area = Rect::new(area.x, area.y, area.width, area.height);
                    state.tab_title_area = Rect::new(area.x, area.y, area.width, 1);
                    state.widget_area = block.inner(state.block_area);
                }
                TabPlacement::Bottom => {
                    state.block_area = Rect::new(area.x, area.y, area.width, area.height);
                    state.tab_title_area = Rect::new(
                        area.x,
                        area.y + area.height.saturating_sub(1),
                        area.width,
                        1,
                    );
                    state.widget_area = block.inner(state.block_area);
                }
                TabPlacement::Left => {
                    state.block_area = Rect::new(
                        area.x + max_width + 2 + close_width,
                        area.y,
                        area.width - (max_width + 2 + close_width),
                        area.height,
                    );
                    state.tab_title_area =
                        Rect::new(area.x, area.y, max_width + 2 + close_width, area.height);
                    state.widget_area = block.inner(state.block_area);
                }
                TabPlacement::Right => {
                    state.block_area = Rect::new(
                        area.x,
                        area.y,
                        area.width - (max_width + 2 + close_width),
                        area.height,
                    );
                    state.tab_title_area = Rect::new(
                        (area.x + area.width).saturating_sub(max_width + 2 + close_width),
                        area.y,
                        max_width + 2 + close_width,
                        area.height,
                    );
                    state.widget_area = block.inner(state.block_area);
                }
            }

            match tabbed.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    let mut constraints = Vec::new();
                    for tab in tabbed.tabs.iter() {
                        constraints.push(Constraint::Length(tab.width() as u16 + 2 + close_width));
                    }

                    state.tab_title_areas = Vec::from(
                        Layout::horizontal(constraints)
                            .flex(Flex::Start)
                            .spacing(1)
                            .horizontal_margin(block_offset + 1)
                            .split(state.tab_title_area)
                            .as_ref(),
                    );
                }
                TabPlacement::Left | TabPlacement::Right => {
                    let mut constraints = Vec::new();
                    for _tab in tabbed.tabs.iter() {
                        constraints.push(Constraint::Length(1));
                    }

                    state.tab_title_areas = Vec::from(
                        Layout::vertical(constraints)
                            .flex(Flex::Start)
                            .vertical_margin(block_offset)
                            .split(state.tab_title_area)
                            .as_ref(),
                    );
                }
            }

            match tabbed.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                (v.x + v.width).saturating_sub(close_width),
                                v.y,
                                if tabbed.closeable { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
                TabPlacement::Left => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                v.x + 1, //
                                v.y,
                                if tabbed.closeable { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
                TabPlacement::Right => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                (v.x + v.width).saturating_sub(close_width),
                                v.y,
                                if tabbed.closeable { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
            }
        }

        fn render(&self, buf: &mut Buffer, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
            let mut block;
            let block = if let Some(block) = &tabbed.block {
                block
            } else {
                block = Block::new()
                    .border_type(BorderType::Plain)
                    .style(tabbed.style);
                block = match tabbed.placement {
                    TabPlacement::Top => block.borders(Borders::TOP),
                    TabPlacement::Bottom => block.borders(Borders::BOTTOM),
                    TabPlacement::Left => block.borders(Borders::LEFT),
                    TabPlacement::Right => block.borders(Borders::RIGHT),
                };
                &block
            };

            let focus_style = if let Some(focus_style) = tabbed.focus_style {
                focus_style
            } else {
                revert_style(tabbed.style)
            };
            let select_style = if let Some(select_style) = tabbed.select_style {
                if state.focus.get() {
                    focus_style
                } else {
                    select_style
                }
            } else {
                if state.focus.get() {
                    focus_style
                } else {
                    revert_style(tabbed.style)
                }
            };
            let tab_style = if let Some(tab_style) = tabbed.tab_style {
                tab_style
            } else {
                tabbed.style
            };

            // area for the left/right tabs.
            if matches!(tabbed.placement, TabPlacement::Left | TabPlacement::Right) {
                buf.set_style(state.tab_title_area, tabbed.style);
            }
            block.clone().render(state.block_area, buf);

            for (idx, tab_area) in state.tab_title_areas.iter().copied().enumerate() {
                if Some(idx) == state.selected() {
                    fill_buf_area(buf, tab_area, " ", select_style);
                } else {
                    fill_buf_area(buf, tab_area, " ", tab_style);
                }

                let txt_area = match tabbed.placement {
                    TabPlacement::Top | TabPlacement::Right | TabPlacement::Bottom => {
                        tab_area.inner(Margin::new(1, 0))
                    }
                    TabPlacement::Left => {
                        if tabbed.closeable {
                            Rect::new(
                                tab_area.x + 3,
                                tab_area.y,
                                tab_area.width - 4,
                                tab_area.height,
                            )
                        } else {
                            tab_area.inner(Margin::new(1, 0))
                        }
                    }
                };
                tabbed.tabs[idx].clone().render(txt_area, buf);

                // join left/right
                match tabbed.placement {
                    TabPlacement::Top => {}
                    TabPlacement::Bottom => {}
                    TabPlacement::Left => {
                        if let Some(cell) = buf.cell_mut((tab_area.x + tab_area.width, tab_area.y))
                        {
                            if let Some(sym) = self.get_link(tabbed.placement, block) {
                                cell.set_symbol(sym);
                            }
                        }
                    }
                    TabPlacement::Right => {
                        if let Some(cell) = buf.cell_mut((tab_area.x - 1, tab_area.y)) {
                            if let Some(sym) = self.get_link(tabbed.placement, block) {
                                cell.set_symbol(sym);
                            }
                        }
                    }
                }
            }

            if tabbed.closeable {
                for i in 0..state.tab_title_close_areas.len() {
                    if state.mouse.hover.get() == Some(i) {
                        buf.set_style(state.tab_title_close_areas[i], revert_style(tab_style));
                    }
                    if let Some(cell) = buf.cell_mut(state.tab_title_close_areas[i].as_position()) {
                        cell.set_symbol("\u{2A2F}");
                    }
                }
            }
        }
    }
}
