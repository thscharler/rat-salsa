//!
//! A simple line menu.
//!
//! If the render area has more than one line, this will
//! linebreak if needed.
//!
//! ## Navigation keys
//! If you give plain-text strings as items, the underscore
//! designates a navigation key. If you hit the key, the matching
//! item is selected. On the second hit, the matching item is
//! activated.
//!
use crate::_private::NonExhaustive;
use crate::util::{menu_str, next_opt, prev_opt, revert_style};
#[allow(unused_imports)]
use log::debug;
use rat_event::util::{item_at_clicked, MouseFlags};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{StatefulWidget, StatefulWidgetRef, Widget, WidgetRef};
use std::fmt::Debug;

/// One line menu widget.
#[derive(Debug, Default, Clone)]
pub struct MenuLine<'a> {
    title: Line<'a>,
    items: Vec<Line<'a>>,
    navchar: Vec<Option<char>>,

    style: Style,
    title_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
}

/// Combined styles.
#[derive(Debug, Clone)]
pub struct MenuStyle {
    pub style: Style,
    pub title: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub non_exhaustive: NonExhaustive,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct MenuLineState {
    /// Current focus state.
    pub focus: FocusFlag,
    /// Area for the whole widget.
    pub area: Rect,
    /// Areas for each item.
    pub item_areas: Vec<Rect>,
    /// Hot keys
    pub navchar: Vec<Option<char>>,

    /// Selected item.
    pub selected: Option<usize>,

    /// Flags for mouse handling.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> MenuLine<'a> {
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a formatted item.
    /// The navchar is optional, any markup for it is your problem.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: Line<'a>, navchar: Option<char>) -> Self {
        self.items.push(item);
        self.navchar.push(navchar);
        self
    }

    /// Add item.
    #[inline]
    pub fn add_str(mut self, txt: &'a str) -> Self {
        let (item, navchar) = menu_str(txt);
        self.items.push(item);
        self.navchar.push(navchar);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        self.title_style = styles.title;
        self.select_style = styles.select;
        self.focus_style = styles.focus;
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Selection
    #[inline]
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }
}

impl<'a> StatefulWidgetRef for MenuLine<'a> {
    type State = MenuLineState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for MenuLine<'a> {
    type State = MenuLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &MenuLine<'_>, area: Rect, buf: &mut Buffer, state: &mut MenuLineState) {
    state.area = area;
    state.item_areas.clear();
    state.navchar = widget.navchar.clone();

    let select_style = if state.is_focused() {
        if let Some(focus_style) = widget.focus_style {
            focus_style
        } else {
            revert_style(widget.style)
        }
    } else {
        if let Some(select_style) = widget.select_style {
            select_style
        } else {
            revert_style(widget.style)
        }
    };
    let title_style = if let Some(title_style) = widget.title_style {
        title_style
    } else {
        widget.style.underlined()
    };

    buf.set_style(area, widget.style);

    let mut item_area = Rect::new(area.x, area.y, 0, 1);
    if widget.title.width() > 0 {
        item_area.width = widget.title.width() as u16;

        buf.set_style(item_area, title_style);
        widget.title.render_ref(item_area, buf);

        item_area.x += item_area.width + 1;
    }

    'f: {
        for (n, item) in widget.items.iter().enumerate() {
            item_area.width = item.width() as u16;

            // line breaks
            if item_area.right() >= area.right() {
                if item_area.bottom() + 1 >= area.bottom() {
                    break 'f;
                }
                item_area.y += 1;
                item_area.x = area.x;
            }

            state.item_areas.push(item_area);

            if state.selected == Some(n) {
                buf.set_style(item_area, select_style);
            }
            item.render(item_area, buf);

            item_area.x += item_area.width + 1;
        }
    }
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            title: None,
            select: None,
            focus: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for MenuLineState {
    /// Focus flag.
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    /// Focus area.
    fn area(&self) -> Rect {
        self.area
    }
}

#[allow(clippy::len_without_is_empty)]
impl MenuLineState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.item_areas.len()
    }

    /// Any items.
    pub fn is_empty(&self) -> bool {
        self.item_areas.is_empty()
    }

    /// Select
    #[inline]
    pub fn select(&mut self, select: Option<usize>) -> bool {
        let old = self.selected;
        self.selected = select;
        old != self.selected
    }

    /// Selected index
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Previous item.
    #[inline]
    pub fn prev_item(&mut self) -> bool {
        let old = self.selected;
        self.selected = prev_opt(self.selected, 1, self.len());
        old != self.selected
    }

    /// Next item.
    #[inline]
    pub fn next_item(&mut self) -> bool {
        let old = self.selected;
        self.selected = next_opt(self.selected, 1, self.len());
        old != self.selected
    }

    /// Select by hotkey
    #[inline]
    pub fn navigate(&mut self, c: char) -> MenuOutcome {
        let c = c.to_ascii_lowercase();
        for (i, cc) in self.navchar.iter().enumerate() {
            if *cc == Some(c) {
                if self.selected == Some(i) {
                    return MenuOutcome::Activated(i);
                } else {
                    self.selected = Some(i);
                    return MenuOutcome::Selected(i);
                }
            }
        }
        MenuOutcome::NotUsed
    }

    /// Select item at position
    #[inline]
    pub fn select_at(&mut self, pos: (u16, u16)) -> bool {
        if let Some(idx) = item_at_clicked(&self.item_areas, pos.0, pos.1) {
            self.selected = Some(idx);
            true
        } else {
            false
        }
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        item_at_clicked(&self.item_areas, pos.0, pos.1)
    }
}

impl Default for MenuLineState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            navchar: Default::default(),
            mouse: Default::default(),
            selected: Default::default(),
            item_areas: Default::default(),
            area: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Outcome for menuline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuOutcome {
    /// The given event was not handled at all.
    NotUsed,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
    /// The menuitem was selected.
    Selected(usize),
    /// The menuitem was selected and activated.
    Activated(usize),
    /// Selected popup-menu.
    MenuSelected(usize, usize),
    /// Activated popup-menu.
    MenuActivated(usize, usize),
}

impl ConsumedEvent for MenuOutcome {
    fn is_consumed(&self) -> bool {
        *self != MenuOutcome::NotUsed
    }
}

impl From<MenuOutcome> for Outcome {
    fn from(value: MenuOutcome) -> Self {
        match value {
            MenuOutcome::NotUsed => Outcome::Continue,
            MenuOutcome::Unchanged => Outcome::Unchanged,
            MenuOutcome::Changed => Outcome::Changed,
            MenuOutcome::Selected(_) => Outcome::Changed,
            MenuOutcome::Activated(_) => Outcome::Changed,
            MenuOutcome::MenuSelected(_, _) => Outcome::Changed,
            MenuOutcome::MenuActivated(_, _) => Outcome::Changed,
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, MenuOutcome> for MenuLineState {
    #[allow(clippy::redundant_closure)]
    fn handle(&mut self, event: &crossterm::event::Event, _: Regular) -> MenuOutcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(key press ' ') => self
                    .selected
                    .map_or(MenuOutcome::Unchanged, |v| MenuOutcome::Selected(v)),
                ct_event!(key press ANY-c) => self.navigate(*c),
                ct_event!(keycode press Left) => {
                    if self.prev_item() {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Right) => {
                    if self.next_item() {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Home) => {
                    if self.select(Some(0)) {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press End) => {
                    if self.select(Some(self.len().saturating_sub(1))) {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Enter) => {
                    if let Some(select) = self.selected {
                        MenuOutcome::Activated(select)
                    } else {
                        MenuOutcome::NotUsed
                    }
                }

                ct_event!(key release _)
                | ct_event!(keycode release Left)
                | ct_event!(keycode release Right)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release Enter) => MenuOutcome::Unchanged,

                _ => MenuOutcome::NotUsed,
            }
        } else {
            MenuOutcome::NotUsed
        };

        if res == MenuOutcome::NotUsed {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for MenuLineState {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> MenuOutcome {
        match event {
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.area, m) => {
                let idx = self.item_at(self.mouse.pos_of(m));
                if self.selected() == idx {
                    match self.selected {
                        Some(a) => MenuOutcome::Activated(a),
                        None => MenuOutcome::NotUsed,
                    }
                } else {
                    MenuOutcome::NotUsed
                }
            }
            ct_event!(mouse any for m) if self.mouse.drag(self.area, m) => {
                let old = self.selected;
                if self.select_at(self.mouse.pos_of(m)) {
                    if old != self.selected {
                        MenuOutcome::Selected(self.selected().expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                } else {
                    MenuOutcome::NotUsed
                }
            }
            ct_event!(mouse down Left for col, row) if self.area.contains((*col, *row).into()) => {
                if self.select_at((*col, *row)) {
                    MenuOutcome::Selected(self.selected().expect("selected"))
                } else {
                    MenuOutcome::NotUsed
                }
            }
            _ => MenuOutcome::NotUsed,
        }
    }
}

/// Handle all events.
/// Text events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut MenuLineState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut MenuLineState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, MouseOnly)
}
