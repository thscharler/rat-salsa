//!
//! A menubar with sub-menus.
//!
//! Combines [MenuLine] and [PopupMenu] and adds a [MenuStructure] trait
//! to bind all together.
//!
//! Rendering and events are split in base-widget and popup.
//! Use [Menubar] to set all configurations and then call [Menubar::into_widgets].
//! This creates a [MenubarLine] and a [MenubarPopup] which implement
//! the rendering traits. MenubarPopup must render *after* all regular
//! widgets, MenubarLine can render whenever.
//!
//! Event-handling for the popup menu works with the [Popup] qualifier,
//! and must be called before the [Regular] event-handlers to work correctly.
//! Event-handling for the menu line is via the [Regular] event-handler.
//!
use crate::event::MenuOutcome;
use crate::menuline::{MenuLine, MenuLineState};
use crate::popup_menu::{Placement, PopupMenu, PopupMenuState};
use crate::{MenuStructure, MenuStyle};
use rat_event::{flow, HandleEvent, MouseOnly, Popup, Regular};
use rat_focus::{FocusFlag, HasFocusFlag, ZRect};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::{Debug, Formatter};

/// Menubar widget.
/// This handles the configuration only, to get the widgets for rendering
/// call [Menubar::into_widgets] and use both results for rendering.
#[derive(Default, Clone)]
pub struct Menubar<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,

    title: Line<'a>,
    style: Style,
    title_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    highlight_style: Option<Style>,

    popup_width: Option<u16>,
    popup_placement: Placement,
    popup_block: Option<Block<'a>>,
}

/// Menubar line widget.
/// This implements the actual render function.
#[derive(Debug, Default, Clone)]
pub struct MenubarLine<'a> {
    menubar: Menubar<'a>,
}

/// Menubar popup widget.
/// Separate renderer for the popup part of the menubar.
#[derive(Debug, Default, Clone)]
pub struct MenubarPopup<'a> {
    menubar: Menubar<'a>,
}

/// State & event-handling.
#[derive(Debug, Default, Clone)]
pub struct MenuBarState {
    /// Total area for the menubar and any visible popup.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Areas for the menubar.
    /// __readonly__. renewed for each render.
    pub z_areas: [ZRect; 2],

    /// State for the menu.
    pub bar: MenuLineState,
    /// State for the last rendered popup menu.
    pub popup: PopupMenuState,
}

impl<'a> Debug for Menubar<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menubar")
            .field("title", &self.title)
            .field("style", &self.style)
            .field("title_style", &self.title_style)
            .field("select_style", &self.select_style)
            .field("focus_style", &self.focus_style)
            .field("popup_width", &self.popup_width)
            .field("popup_placement", &self.popup_placement)
            .field("popup_block", &self.popup_block)
            .finish()
    }
}

impl<'a> Menubar<'a> {
    pub fn new(structure: &'a dyn MenuStructure<'a>) -> Self {
        Self {
            structure: Some(structure),
            ..Default::default()
        }
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        self.title_style = styles.title;
        self.select_style = styles.select;
        self.focus_style = styles.focus;
        self.highlight_style = styles.highlight;
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

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn popup_width(mut self, width: u16) -> Self {
        self.popup_width = Some(width);
        self
    }

    /// Placement relative to the render-area.
    pub fn popup_placement(mut self, placement: Placement) -> Self {
        self.popup_placement = placement;
        self
    }

    /// Block for borders.
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup_block = Some(block);
        self
    }

    /// Create the widgets for the Menubar. This returns a widget
    /// for the menu-line and for the menu-popup.
    ///
    /// The menu-popup should be rendered, after all widgets
    /// that might be below the popup have been rendered.
    pub fn into_widgets(self) -> (MenubarLine<'a>, MenubarPopup<'a>) {
        (
            MenubarLine {
                menubar: self.clone(),
            },
            MenubarPopup { menubar: self },
        )
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for MenubarLine<'a> {
    type State = MenuBarState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(&self.menubar, area, buf, state);
    }
}

impl<'a> StatefulWidget for MenubarLine<'a> {
    type State = MenuBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(&self.menubar, area, buf, state);
    }
}

fn render_menubar(widget: &Menubar<'_>, area: Rect, buf: &mut Buffer, state: &mut MenuBarState) {
    let mut menu = MenuLine::new()
        .title(widget.title.clone())
        .style(widget.style);
    if let Some(title_style) = widget.title_style {
        menu = menu.title_style(title_style);
    }
    if let Some(select_style) = widget.select_style {
        menu = menu.select_style(select_style);
    }
    if let Some(focus_style) = widget.focus_style {
        menu = menu.focus_style(focus_style);
    }
    if let Some(highlight_style) = widget.highlight_style {
        menu = menu.highlight_style(highlight_style);
    }

    if let Some(structure) = &widget.structure {
        let mut items = Vec::new();
        structure.menus(&mut items);
        for m in items {
            menu = menu.add(m);
        }
    }
    menu.render(area, buf, &mut state.bar);

    // Combined area + each part with a z-index.
    state.area = state.bar.area;
    state.z_areas = [ZRect::from((0, state.bar.area)), ZRect::default()];
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for MenubarPopup<'a> {
    type State = MenuBarState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(&self.menubar, area, buf, state);
    }
}

impl<'a> StatefulWidget for MenubarPopup<'a> {
    type State = MenuBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(&self.menubar, area, buf, state);
    }
}

fn render_menu_popup(
    widget: &Menubar<'_>,
    _area: Rect,
    buf: &mut Buffer,
    state: &mut MenuBarState,
) {
    // Combined area + each part with a z-index.
    state.area = state.bar.area;
    state.z_areas = [ZRect::from((0, state.bar.area)), ZRect::default()];

    let Some(selected) = state.bar.selected() else {
        return;
    };
    let Some(structure) = widget.structure else {
        return;
    };

    if state.popup.is_focused() {
        let mut len = 0;
        let mut popup = PopupMenu::new()
            .placement(widget.popup_placement)
            .style(widget.style);
        if let Some(width) = widget.popup_width {
            popup = popup.width(width);
        }
        if let Some(block) = widget.popup_block.clone() {
            popup = popup.block(block);
        }
        if let Some(focus_style) = widget.focus_style {
            popup = popup.focus_style(focus_style);
        }
        if let Some(highlight_style) = widget.highlight_style {
            popup = popup.highlight_style(highlight_style);
        }
        let mut items = Vec::new();
        structure.submenu(selected, &mut items);
        for item in items {
            popup = popup.add(item);
            len += 1;
        }

        if len > 0 {
            let area = state.bar.item_areas[selected];
            popup.render(area, buf, &mut state.popup);

            // Combined area + each part with a z-index.
            state.area = state.bar.area.union(state.popup.area);
            state.z_areas[1] = ZRect::from((1, state.popup.area));
        }
    } else {
        state.popup = Default::default();
    }
}

impl MenuBarState {
    /// State.
    /// For the specifics use the public fields `menu` and `popup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// New state with a focus name.
    pub fn named(name: &'static str) -> Self {
        Self {
            bar: MenuLineState::named(format!("{}.bar", name).to_string().leak()),
            popup: PopupMenuState::named(format!("{}.popup", name).to_string().leak()),
            ..Default::default()
        }
    }

    /// Submenu visible/active.
    pub fn popup_active(&self) -> bool {
        self.popup.is_focused()
    }

    /// Submenu visible/active.
    pub fn set_popup_active(&mut self, active: bool) {
        self.popup.focus.set(active);
    }
}

impl HasFocusFlag for MenuBarState {
    fn focus(&self) -> FocusFlag {
        self.bar.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn z_areas(&self) -> &[ZRect] {
        &self.z_areas
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for MenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        if let Some(selected) = self.bar.selected() {
            if self.popup_active() {
                match self.popup.handle(event, Popup) {
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => MenuOutcome::MenuActivated(selected, n),
                    r => r,
                }
            } else {
                MenuOutcome::Continue
            }
        } else {
            MenuOutcome::Continue
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, MenuOutcome> for MenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        if self.bar.is_focused() {
            let old_selected = self.bar.selected();
            match self.bar.handle(event, Regular) {
                r @ MenuOutcome::Selected(_) => {
                    if self.bar.selected == old_selected {
                        self.popup.flip_active();
                    } else {
                        self.popup.set_active(true);
                    }
                    r
                }
                r => r,
            }
        } else {
            self.bar.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for MenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        flow!(if let Some(selected) = self.bar.selected() {
            if self.popup_active() {
                match self.popup.handle(event, MouseOnly) {
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => {
                        self.set_popup_active(false);
                        MenuOutcome::MenuActivated(selected, n)
                    }
                    r => r,
                }
            } else {
                MenuOutcome::Continue
            }
        } else {
            MenuOutcome::Continue
        });

        let old_selected = self.bar.selected();
        match self.bar.handle(event, MouseOnly) {
            r @ MenuOutcome::Selected(_) => {
                if self.bar.selected == old_selected {
                    self.popup.flip_active();
                } else {
                    self.popup.set_active(true);
                }
                r
            }
            r => r,
        }
    }
}

/// Handle menu events. Keyboard events are processed if focus is true.
///
/// Attention:
/// For the event-handling of the popup-menus you need to call handle_popup_events().
pub fn handle_events(
    state: &mut MenuBarState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.bar.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle menu events for the popup-menu.
///
/// This one is separate, as it needs to be called before other event-handlers
/// to cope with overlapping regions.
///
/// focus - is the menubar focused?
pub fn handle_popup_events(
    state: &mut MenuBarState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.bar.focus.set(focus);
    state.handle(event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut MenuLineState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, MouseOnly)
}
