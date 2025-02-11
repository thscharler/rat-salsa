//!
//! A main menu widget.
//!
use crate::_private::NonExhaustive;
use crate::event::MenuOutcome;
use crate::util::{fallback_select_style, revert_style};
use crate::{MenuBuilder, MenuItem, MenuStyle};
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::{Style, Stylize};
use ratatui_core::text::{Line, Span};
// #[cfg(feature = "unstable-widget-ref")]
// use ratatui::widgets::StatefulWidgetRef;
use ratatui_core::widgets::{StatefulWidget, Widget};
use std::fmt::Debug;

/// Main menu widget.
#[derive(Debug, Default, Clone)]
pub struct MenuLine<'a> {
    title: Line<'a>,
    pub(crate) menu: MenuBuilder<'a>,

    style: Style,
    highlight_style: Option<Style>,
    disabled_style: Option<Style>,
    right_style: Option<Style>,
    title_style: Option<Style>,
    // TODO: breaking: remove separate select_style
    select_style: Option<Style>,
    focus_style: Option<Style>,
}

/// State & event handling.
#[derive(Debug)]
pub struct MenuLineState {
    /// Area for the whole widget.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Areas for each item.
    /// __readonly__. renewed for each render.
    pub item_areas: Vec<Rect>,
    /// Hot keys
    /// __readonly__. renewed for each render.
    pub navchar: Vec<Option<char>>,
    /// Disable menu-items.
    /// __readonly__. renewed for each render.
    pub disabled: Vec<bool>,

    // TODO: breaking: remove Option
    /// Selected item.
    /// __read+write__
    pub selected: Option<usize>,

    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Flags for mouse handling.
    /// __used for mouse interaction__
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

    /// Add an item.
    pub fn item(mut self, item: MenuItem<'a>) -> Self {
        self.menu.item(item);
        self
    }

    /// Parse the text.
    ///
    /// __See__
    ///
    /// [MenuItem::new_parsed]
    pub fn item_parsed(mut self, text: &'a str) -> Self {
        self.menu.item_parsed(text);
        self
    }

    /// Add a text-item.
    pub fn item_str(mut self, txt: &'a str) -> Self {
        self.menu.item_str(txt);
        self
    }

    /// Add an owned text as item.
    pub fn item_string(mut self, txt: String) -> Self {
        self.menu.item_string(txt);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        if styles.highlight.is_some() {
            self.highlight_style = styles.highlight;
        }
        if styles.disabled.is_some() {
            self.disabled_style = styles.disabled;
        }
        if styles.right.is_some() {
            self.right_style = styles.right;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.title.is_some() {
            self.title_style = styles.title;
        }
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Shortcut highlight style.
    #[inline]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = Some(style);
        self
    }

    /// Shortcut highlight style.
    #[inline]
    pub fn highlight_style_opt(mut self, style: Option<Style>) -> Self {
        self.highlight_style = style;
        self
    }

    /// Disabled item style.
    #[inline]
    pub fn disabled_style(mut self, style: Style) -> Self {
        self.disabled_style = Some(style);
        self
    }

    /// Disabled item style.
    #[inline]
    pub fn disabled_style_opt(mut self, style: Option<Style>) -> Self {
        self.disabled_style = style;
        self
    }

    /// Style for the hotkey.
    #[inline]
    pub fn right_style(mut self, style: Style) -> Self {
        self.right_style = Some(style);
        self
    }

    /// Style for the hotkey.
    #[inline]
    pub fn right_style_opt(mut self, style: Option<Style>) -> Self {
        self.right_style = style;
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style_opt(mut self, style: Option<Style>) -> Self {
        self.title_style = style;
        self
    }

    /// Selection
    #[inline]
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Selection
    #[inline]
    pub fn select_style_opt(mut self, style: Option<Style>) -> Self {
        self.select_style = style;
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style_opt(mut self, style: Option<Style>) -> Self {
        self.focus_style = style;
        self
    }
}

// #[cfg(feature = "unstable-widget-ref")]
// impl<'a> StatefulWidgetRef for MenuLine<'a> {
//     type State = MenuLineState;
//
//     fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
//         render_ref(self, area, buf, state);
//     }
// }

impl StatefulWidget for MenuLine<'_> {
    type State = MenuLineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &MenuLine<'_>, area: Rect, buf: &mut Buffer, state: &mut MenuLineState) {
    state.area = area;
    state.item_areas.clear();

    if widget.menu.items.is_empty() {
        state.selected = None;
    } else if state.selected.is_none() {
        state.selected = Some(0);
    }

    state.navchar = widget
        .menu
        .items
        .iter()
        .map(|v| v.navchar.map(|w| w.to_ascii_lowercase()))
        .collect();
    state.disabled = widget.menu.items.iter().map(|v| v.disabled).collect();

    let style = widget.style;
    #[allow(clippy::collapsible_else_if)]
    let select_style = if state.is_focused() {
        if let Some(focus_style) = widget.focus_style {
            focus_style
        } else {
            revert_style(style)
        }
    } else {
        if let Some(select_style) = widget.select_style {
            select_style
        } else {
            fallback_select_style(style)
        }
    };
    let title_style = if let Some(title_style) = widget.title_style {
        title_style
    } else {
        style.clone().underlined()
    };
    let highlight_style = if let Some(highlight_style) = widget.highlight_style {
        highlight_style
    } else {
        Style::new().underlined()
    };
    let right_style = if let Some(right_style) = widget.right_style {
        right_style
    } else {
        Style::new().italic()
    };
    let disabled_style = if let Some(disabled_style) = widget.disabled_style {
        disabled_style
    } else {
        widget.style
    };

    buf.set_style(area, style);

    let mut item_area = Rect::new(area.x, area.y, 0, 1);

    if widget.title.width() > 0 {
        item_area.width = widget.title.width() as u16;

        buf.set_style(item_area, title_style);
        widget.title.clone().render(item_area, buf);

        item_area.x += item_area.width + 1;
    }

    for (n, item) in widget.menu.items.iter().enumerate() {
        item_area.width =
            item.item_width() + item.right_width() + if item.right.is_empty() { 0 } else { 2 };
        if item_area.right() >= area.right() {
            item_area = item_area.clamp(area);
        }
        state.item_areas.push(item_area);

        #[allow(clippy::collapsible_else_if)]
        let (style, right_style) = if state.selected == Some(n) {
            if item.disabled {
                (
                    style.patch(disabled_style),
                    style.patch(disabled_style).patch(right_style),
                )
            } else {
                (
                    style.patch(select_style),
                    style.patch(select_style).patch(right_style),
                )
            }
        } else {
            if item.disabled {
                (
                    style.patch(disabled_style),
                    style.patch(disabled_style).patch(right_style),
                )
            } else {
                (style, style.patch(right_style))
            }
        };

        let item_line = if let Some(highlight) = item.highlight.clone() {
            Line::from_iter([
                Span::from(&item.item[..highlight.start - 1]), // account for _
                Span::from(&item.item[highlight.start..highlight.end]).style(highlight_style),
                Span::from(&item.item[highlight.end..]),
                if !item.right.is_empty() {
                    Span::from(format!("({})", item.right)).style(right_style)
                } else {
                    Span::default()
                },
            ])
        } else {
            Line::from_iter([
                Span::from(item.item.as_ref()),
                if !item.right.is_empty() {
                    Span::from(format!("({})", item.right)).style(right_style)
                } else {
                    Span::default()
                },
            ])
        };
        item_line.style(style).render(item_area, buf);

        item_area.x += item_area.width + 1;
    }
}

impl HasFocus for MenuLineState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    /// Focus flag.
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
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

    /// New with a focus name.
    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Default::default()
        }
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

        // before first render or no items:
        if self.disabled.is_empty() {
            return false;
        }

        self.selected = if let Some(start) = old {
            let mut idx = start;
            loop {
                if idx == 0 {
                    idx = start;
                    break;
                }
                idx -= 1;

                if self.disabled.get(idx) == Some(&false) {
                    break;
                }
            }

            Some(idx)
        } else if self.len() > 0 {
            Some(self.len().saturating_sub(1))
        } else {
            None
        };

        old != self.selected
    }

    /// Next item.
    #[inline]
    pub fn next_item(&mut self) -> bool {
        let old = self.selected;

        // before first render or no items:
        if self.disabled.is_empty() {
            return false;
        }

        self.selected = if let Some(start) = old {
            let mut idx = start;
            loop {
                if idx + 1 == self.len() {
                    idx = start;
                    break;
                }
                idx += 1;

                if self.disabled.get(idx) == Some(&false) {
                    break;
                }
            }
            Some(idx)
        } else if self.len() > 0 {
            Some(0)
        } else {
            None
        };

        old != self.selected
    }

    /// Select by hotkey
    #[inline]
    pub fn navigate(&mut self, c: char) -> MenuOutcome {
        // before first render or no items:
        if self.disabled.is_empty() {
            return MenuOutcome::Continue;
        }

        let c = c.to_ascii_lowercase();
        for (i, cc) in self.navchar.iter().enumerate() {
            #[allow(clippy::collapsible_if)]
            if *cc == Some(c) {
                if self.disabled.get(i) == Some(&false) {
                    if self.selected == Some(i) {
                        return MenuOutcome::Activated(i);
                    } else {
                        self.selected = Some(i);
                        return MenuOutcome::Selected(i);
                    }
                }
            }
        }

        MenuOutcome::Continue
    }

    /// Select item at position.
    /// Only reports a change if the selection actually changed.
    /// Reports no change before the first render and if no item was hit.
    #[inline]
    pub fn select_at(&mut self, pos: (u16, u16)) -> bool {
        let old_selected = self.selected;

        // before first render or no items:
        if self.disabled.is_empty() {
            return false;
        }

        if let Some(idx) = self.mouse.item_at(&self.item_areas, pos.0, pos.1) {
            if self.disabled.get(idx) == Some(&false) {
                self.selected = Some(idx);
            }
        }

        self.selected != old_selected
    }

    /// Select item at position.
    /// Reports a change even if the same menu item has been selected.
    /// Reports no change before the first render and if no item was hit.
    #[inline]
    pub fn select_at_always(&mut self, pos: (u16, u16)) -> bool {
        // before first render or no items:
        if self.disabled.is_empty() {
            return false;
        }

        if let Some(idx) = self.mouse.item_at(&self.item_areas, pos.0, pos.1) {
            if self.disabled.get(idx) == Some(&false) {
                self.selected = Some(idx);
                return true;
            }
        }

        false
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        self.mouse.item_at(&self.item_areas, pos.0, pos.1)
    }
}

impl Clone for MenuLineState {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            item_areas: self.item_areas.clone(),
            navchar: self.navchar.clone(),
            disabled: self.disabled.clone(),
            selected: self.selected,
            focus: FocusFlag::named(self.focus.name()),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for MenuLineState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            item_areas: vec![],
            navchar: vec![],
            disabled: vec![],
            selected: None,
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, MenuOutcome> for MenuLineState {
    #[allow(clippy::redundant_closure)]
    fn handle(&mut self, event: &crossterm::event::Event, _: Regular) -> MenuOutcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(key press ' ') => {
                    self
                        .selected//
                        .map_or(MenuOutcome::Continue, |v| MenuOutcome::Selected(v))
                }
                ct_event!(key press ANY-c) => {
                    self.navigate(*c) //
                }
                ct_event!(keycode press Left) => {
                    if self.prev_item() {
                        if let Some(selected) = self.selected {
                            MenuOutcome::Selected(selected)
                        } else {
                            MenuOutcome::Changed
                        }
                    } else {
                        MenuOutcome::Continue
                    }
                }
                ct_event!(keycode press Right) => {
                    if self.next_item() {
                        if let Some(selected) = self.selected {
                            MenuOutcome::Selected(selected)
                        } else {
                            MenuOutcome::Changed
                        }
                    } else {
                        MenuOutcome::Continue
                    }
                }
                ct_event!(keycode press Home) => {
                    if self.select(Some(0)) {
                        if let Some(selected) = self.selected {
                            MenuOutcome::Selected(selected)
                        } else {
                            MenuOutcome::Changed
                        }
                    } else {
                        MenuOutcome::Continue
                    }
                }
                ct_event!(keycode press End) => {
                    if self.select(Some(self.len().saturating_sub(1))) {
                        if let Some(selected) = self.selected {
                            MenuOutcome::Selected(selected)
                        } else {
                            MenuOutcome::Changed
                        }
                    } else {
                        MenuOutcome::Continue
                    }
                }
                ct_event!(keycode press Enter) => {
                    if let Some(select) = self.selected {
                        MenuOutcome::Activated(select)
                    } else {
                        MenuOutcome::Continue
                    }
                }
                _ => MenuOutcome::Continue,
            }
        } else {
            MenuOutcome::Continue
        };

        if res == MenuOutcome::Continue {
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
                        None => MenuOutcome::Continue,
                    }
                } else {
                    MenuOutcome::Continue
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
                    MenuOutcome::Continue
                }
            }
            ct_event!(mouse down Left for col, row) if self.area.contains((*col, *row).into()) => {
                if self.select_at_always((*col, *row)) {
                    MenuOutcome::Selected(self.selected().expect("selected"))
                } else {
                    MenuOutcome::Continue
                }
            }
            _ => MenuOutcome::Continue,
        }
    }
}

/// Handle all events.
/// Key events are only processed if focus is true.
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
