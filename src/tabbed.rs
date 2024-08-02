use crate::_private::NonExhaustive;
use crate::event::TabbedOutcome;
use crate::tabbed::glued::GluedTabs;
use rat_event::util::item_at_clicked;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};
use std::cell::Cell;
use std::cmp::min;
use std::fmt::Debug;
use std::rc::Rc;

/// The design space for tabs is too big to capture with a handful of parameters.
///
/// This trait splits off the layout and rendering of the actual tabs from
/// the general properties and behaviour of tabs.
pub trait TabType: Debug {
    /// Calculate the layout for the tabs.
    fn layout<'a>(
        &self, //
        area: Rect,
        tabbed: &Tabbed<'a>,
        state: &mut TabbedState,
    );

    /// Render the tabs.
    fn render<'a>(
        &self, //
        buf: &mut Buffer,
        tabbed: &Tabbed<'a>,
        state: &mut TabbedState,
    );
}

/// A tabbed widget.
///
/// This widget draws the tabs and handles events.
///
/// Use [TabbedState::selected] and [TabbedState::inner_area] to render
/// the actual content of the tab.
///
#[derive(Debug)]
pub struct Tabbed<'a> {
    tab_type: Box<dyn TabType + 'a>,

    closeable: bool,
    tabs: Vec<Line<'a>>,
    block: Option<Block<'a>>,

    style: Style,
    tab_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
}

impl<'a> Tabbed<'a> {
    pub fn new() -> Self {
        Self {
            tab_type: Box::new(GluedTabs::new()),
            closeable: false,
            tabs: Default::default(),
            block: None,
            style: Default::default(),
            tab_style: None,
            select_style: None,
            focus_style: None,
        }
    }

    /// Tab-type is a trait that handles layout and rendering for
    /// the tabs.
    ///
    /// See [GluedTabs] and [AttachedTabs].
    pub fn tab_type(mut self, tab_type: impl TabType + 'a) -> Self {
        self.tab_type = Box::new(tab_type);
        self
    }

    /// Tab-text.
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = impl Into<Line<'a>>>) -> Self {
        self.tabs = tabs.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        self
    }

    /// Tab-text.
    pub fn get_tabs(&self) -> &[Line<'a>] {
        &self.tabs
    }

    /// Closeable tabs?
    ///
    /// Renders a close symbol and reacts with [TabbedOutcome::Close].
    pub fn closeable(mut self, closeable: bool) -> Self {
        self.closeable = closeable;
        self
    }

    /// Closeable tabs?
    pub fn is_closeable(&self) -> bool {
        self.closeable
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Block
    pub fn get_block(&self) -> Option<&Block<'a>> {
        self.block.as_ref()
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

    /// Base style.
    pub fn get_style(&self) -> Style {
        self.style
    }

    /// Style for the tab-text.
    pub fn tab_style(mut self, style: Style) -> Self {
        self.tab_style = Some(style);
        self
    }

    /// Style for the tab-text.
    pub fn get_tab_style(&self) -> Option<Style> {
        self.tab_style
    }

    /// Style for the selected tab.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Style for the selected tab.
    pub fn get_select_style(&self) -> Option<Style> {
        self.select_style
    }

    /// Style for a focused tab.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Style for a focused tab.
    pub fn get_focus_style(&self) -> Option<Style> {
        self.focus_style
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
    pub area: Rect,
    /// Area for drawing the Block inside the tabs.
    pub block_area: Rect,
    /// Area used to render the content of the tab.
    /// Use this area to render the current tab content.
    pub widget_area: Rect,

    /// Total area reserved for tabs.
    pub tab_title_area: Rect,
    /// Area of each tab.
    pub tab_title_areas: Vec<Rect>,
    /// Area for 'Close Tab' interaction.
    pub tab_title_close_areas: Vec<Rect>,

    /// Selected Tab, only ever is None if there are no tabs.
    /// Otherwise, set to 0 on render.
    pub selected: Rc<Cell<Option<usize>>>,

    /// Focus
    pub focus: FocusFlag,
}

impl<'a> StatefulWidget for Tabbed<'a> {
    type State = TabbedState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

impl<'a> StatefulWidgetRef for Tabbed<'a> {
    type State = TabbedState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

fn render_ref(tabbed: &Tabbed<'_>, area: Rect, buf: &mut Buffer, state: &mut TabbedState) {
    if tabbed.tabs.is_empty() {
        state.selected.set(None)
    } else {
        if state.selected.get().is_none() {
            state.selected.set(Some(0));
        }
    }

    tabbed.tab_type.layout(area, tabbed, state);
    tabbed.tab_type.render(buf, tabbed, state);
}

impl HasFocusFlag for TabbedState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        Rect::default()
    }

    fn navigable(&self) -> bool {
        false
    }
}

impl TabbedState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected.get()
    }

    pub fn set_selected(&mut self, selected: usize) {
        self.selected.set(Some(selected));
    }

    /// Selects the next tab. Stops at the end.
    pub fn next_tab(&mut self) -> bool {
        let old_selected = self.selected.get();

        if let Some(selected) = self.selected() {
            self.selected.set(Some(min(
                selected + 1,
                self.tab_title_areas.len().saturating_sub(1),
            )));
        }

        old_selected != self.selected.get()
    }

    /// Selects the previous tab. Stops at the end.
    pub fn prev_tab(&mut self) -> bool {
        let old_selected = self.selected.get();

        if let Some(selected) = self.selected() {
            if selected > 0 {
                self.selected.set(Some(selected - 1));
            }
        }

        old_selected != self.selected.get()
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
            ct_event!(mouse down Left for x, y) => {
                if let Some(sel) = item_at_clicked(&self.tab_title_close_areas, *x, *y) {
                    TabbedOutcome::Close(sel)
                } else if let Some(sel) = item_at_clicked(&self.tab_title_areas, *x, *y) {
                    self.set_selected(sel);
                    TabbedOutcome::Changed
                } else {
                    TabbedOutcome::Continue
                }
            }
            _ => TabbedOutcome::Continue,
        }
    }
}

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

/// Simple glued on the side tabs.
pub mod glued {
    use crate::fill::Fill;
    use crate::tabbed::{TabPlacement, TabType, Tabbed, TabbedState};
    use crate::util::revert_style;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
    use ratatui::widgets::{Widget, WidgetRef};

    /// Renders tab at the given placement.
    /// The block is rendered inside the tabs.
    #[derive(Debug)]
    pub struct GluedTabs {
        placement: TabPlacement,
    }

    impl GluedTabs {
        pub fn new() -> Self {
            Self {
                placement: Default::default(),
            }
        }

        /// Where to place the tabs.
        pub fn placement(mut self, placement: TabPlacement) -> Self {
            self.placement = placement;
            self
        }
    }

    impl TabType for GluedTabs {
        fn layout<'a>(&self, area: Rect, tabbed: &Tabbed<'a>, state: &mut TabbedState) {
            state.area = area;

            let block_offset = if tabbed.block.is_some() { 1 } else { 0 };
            let close_width = if tabbed.is_closeable() { 2 } else { 0 };

            let max_width = tabbed
                .get_tabs()
                .iter()
                .map(|v| v.width())
                .max()
                .unwrap_or_default() as u16;

            match self.placement {
                TabPlacement::Top => {
                    state.block_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
                    state.tab_title_area = Rect::new(area.x, area.y, area.width, 1);
                    if let Some(block) = tabbed.get_block() {
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
                    if let Some(block) = tabbed.get_block() {
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
                    if let Some(block) = tabbed.get_block() {
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
                    if let Some(block) = tabbed.get_block() {
                        state.widget_area = block.inner(state.block_area);
                    } else {
                        state.widget_area = state.block_area;
                    }
                }
            }

            match self.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    let mut constraints = Vec::new();
                    for tab in tabbed.get_tabs() {
                        constraints.push(Constraint::Length(tab.width() as u16 + 2 + close_width));
                    }

                    state.tab_title_areas = Vec::from(
                        Layout::horizontal(constraints)
                            .flex(Flex::Start)
                            .spacing(1)
                            .horizontal_margin(block_offset)
                            .split(state.tab_title_area)
                            .as_ref(),
                    );
                }
                TabPlacement::Left | TabPlacement::Right => {
                    let mut constraints = Vec::new();
                    for _tab in tabbed.get_tabs() {
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

            match self.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                (v.x + v.width).saturating_sub(close_width),
                                v.y,
                                if tabbed.is_closeable() { 1 } else { 0 },
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
                                if tabbed.is_closeable() { 1 } else { 0 },
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
                                if tabbed.is_closeable() { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
            }
        }

        fn render<'a>(&self, buf: &mut Buffer, tabbed: &Tabbed<'a>, state: &mut TabbedState) {
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

            Fill::new()
                .style(tabbed.style)
                .render(state.tab_title_area, buf);
            tabbed.block.render_ref(state.block_area, buf);

            for (idx, tab_area) in state.tab_title_areas.iter().copied().enumerate() {
                if Some(idx) == state.selected() {
                    buf.set_style(tab_area, select_style);
                } else {
                    buf.set_style(tab_area, tab_style);
                }

                let txt_area = match self.placement {
                    TabPlacement::Top | TabPlacement::Right | TabPlacement::Bottom => {
                        tab_area.inner(Margin::new(1, 0))
                    }
                    TabPlacement::Left => {
                        if tabbed.is_closeable() {
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
                tabbed.get_tabs()[idx].render_ref(txt_area, buf);
            }
            if tabbed.is_closeable() {
                for i in 0..state.tab_title_close_areas.len() {
                    "\u{2A2F}".render_ref(state.tab_title_close_areas[i], buf);
                }
            }
        }
    }
}

/// Tabs embedded in the Block.
///
/// If no block has been set, this will draw a block at the side
/// of the tabs.
pub mod attached {
    use crate::fill::Fill;
    use crate::tabbed::{TabPlacement, TabType, Tabbed, TabbedState};
    use crate::util::revert_style;
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
    use ratatui::widgets::{Block, BorderType, Borders, Widget, WidgetRef};

    /// Embeds tabs in the Block.
    ///
    /// If no block has been set, this will draw a block at the side
    /// of the tabs.
    ///
    /// On the left/right side this will just draw a join to the tab-text.
    /// On the top/bottom side the tabs will be embedded in the border.
    #[derive(Debug)]
    pub struct AttachedTabs {
        placement: TabPlacement,
        join: Option<BorderType>,
    }

    impl AttachedTabs {
        pub fn new() -> Self {
            Self {
                placement: Default::default(),
                join: None,
            }
        }

        /// Placement of the tabs.
        pub fn placement(mut self, placement: TabPlacement) -> Self {
            self.placement = placement;
            self
        }

        /// Draw joins for this border-type.
        pub fn join(mut self, border_type: BorderType) -> Self {
            self.join = Some(border_type);
            self
        }
    }

    impl TabType for AttachedTabs {
        fn layout<'a>(&self, area: Rect, tabbed: &Tabbed<'a>, state: &mut TabbedState) {
            state.area = area;

            let mut block;
            let (block, block_offset) = if let Some(block) = &tabbed.block {
                (block, 1)
            } else {
                block = Block::new();
                block = match self.placement {
                    TabPlacement::Top => block.borders(Borders::TOP),
                    TabPlacement::Left => block.borders(Borders::LEFT),
                    TabPlacement::Right => block.borders(Borders::RIGHT),
                    TabPlacement::Bottom => block.borders(Borders::BOTTOM),
                };
                (&block, 0)
            };

            let close_width = if tabbed.is_closeable() { 2 } else { 0 };

            let max_width = tabbed
                .get_tabs()
                .iter()
                .map(|v| v.width())
                .max()
                .unwrap_or_default() as u16;

            match self.placement {
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

            match self.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    let mut constraints = Vec::new();
                    for tab in tabbed.get_tabs() {
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
                    for _tab in tabbed.get_tabs() {
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

            match self.placement {
                TabPlacement::Top | TabPlacement::Bottom => {
                    state.tab_title_close_areas = state
                        .tab_title_areas
                        .iter()
                        .map(|v| {
                            Rect::new(
                                (v.x + v.width).saturating_sub(close_width),
                                v.y,
                                if tabbed.is_closeable() { 1 } else { 0 },
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
                                if tabbed.is_closeable() { 1 } else { 0 },
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
                                if tabbed.is_closeable() { 1 } else { 0 },
                                1,
                            )
                        })
                        .collect::<Vec<_>>();
                }
            }
        }

        fn render<'a>(&self, buf: &mut Buffer, tabbed: &Tabbed<'a>, state: &mut TabbedState) {
            let mut block;
            let block = if let Some(block) = &tabbed.block {
                block
            } else {
                block = Block::new();
                block = match self.placement {
                    TabPlacement::Top => block.borders(Borders::TOP).border_type(BorderType::Plain),
                    TabPlacement::Bottom => block
                        .borders(Borders::BOTTOM)
                        .border_type(BorderType::Plain),
                    TabPlacement::Left => {
                        block.borders(Borders::LEFT).border_type(BorderType::Plain)
                    }
                    TabPlacement::Right => {
                        block.borders(Borders::RIGHT).border_type(BorderType::Plain)
                    }
                };
                block = block.style(tabbed.style);
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

            match self.placement {
                TabPlacement::Left | TabPlacement::Right => {
                    Fill::new()
                        .style(tabbed.style)
                        .render(state.tab_title_area, buf);
                }
                TabPlacement::Top | TabPlacement::Bottom => {}
            }
            block.render_ref(state.block_area, buf);

            for (idx, tab_area) in state.tab_title_areas.iter().copied().enumerate() {
                if Some(idx) == state.selected() {
                    Fill::new()
                        .style(select_style)
                        .fill_char(" ")
                        .render(tab_area, buf);
                } else {
                    Fill::new()
                        .style(tab_style)
                        .fill_char(" ")
                        .render(tab_area, buf);
                }

                let txt_area = match self.placement {
                    TabPlacement::Top | TabPlacement::Right | TabPlacement::Bottom => {
                        tab_area.inner(Margin::new(1, 0))
                    }
                    TabPlacement::Left => {
                        if tabbed.is_closeable() {
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
                tabbed.get_tabs()[idx].render_ref(txt_area, buf);

                match self.placement {
                    TabPlacement::Top => {}
                    TabPlacement::Bottom => {}
                    TabPlacement::Left => {
                        let join = match self.join {
                            None => "\u{2524}",
                            Some(BorderType::Plain) => "\u{2524}",
                            Some(BorderType::Rounded) => "\u{2524}",
                            Some(BorderType::Double) => "\u{2562}",
                            Some(BorderType::Thick) => "\u{2528}",
                            Some(BorderType::QuadrantInside) => "\u{2588}",
                            Some(BorderType::QuadrantOutside) => "\u{258C}",
                        };
                        buf.get_mut(tab_area.x + tab_area.width, tab_area.y)
                            .set_symbol(join);
                    }
                    TabPlacement::Right => {
                        let join = match self.join {
                            None => "\u{251C}",
                            Some(BorderType::Plain) => "\u{251C}",
                            Some(BorderType::Rounded) => "\u{251C}",
                            Some(BorderType::Double) => "\u{255F}",
                            Some(BorderType::Thick) => "\u{2520}",
                            Some(BorderType::QuadrantInside) => "\u{2588}",
                            Some(BorderType::QuadrantOutside) => "\u{2590}",
                        };
                        buf.get_mut(tab_area.x - 1, tab_area.y).set_symbol(join);
                    }
                }
            }
            if tabbed.is_closeable() {
                for i in 0..state.tab_title_close_areas.len() {
                    "\u{2A2F}".render_ref(state.tab_title_close_areas[i], buf);
                }
            }
        }
    }
}
