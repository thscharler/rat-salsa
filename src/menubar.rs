//!
//! A menubar widget with sub-menus.
//!
//! Combines [RMenuLine] and [PopupMenu] and adds a [MenuStructure] trait
//! to bind all together.
//!
//! Rendering is split in two widgets [RMenuBar] and [RMenuPopup].
//! This should help with front/back rendering.
//!
//! Event-handling for the popup menu is split via the [Popup] qualifier.
//! All `Popup` event-handling should be called before the regular
//! `FocusKeys` handling.
//!
use crate::event::{flow, FocusKeys, HandleEvent, MouseOnly};
use crate::menuline::{RMenuLine, RMenuLineState};
use rat_focus::{FocusFlag, HasFocusFlag, ZRect};
use rat_input::event::Popup;
use rat_input::menuline::{MenuOutcome, MenuStyle};
use rat_input::popup_menu::{Placement, PopupMenu, PopupMenuState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, StatefulWidget, Style};
use ratatui::widgets::{Block, StatefulWidgetRef};
use std::fmt::{Debug, Formatter};

pub use rat_input::menubar::{MenuStructure, StaticMenu};

/// MenuBar widget.
///
/// This is only half of the widget. For popup rendering there is the separate
/// [RMenuPopup].
#[derive(Debug, Default, Clone)]
pub struct RMenuBar<'a> {
    menu: RMenuLine<'a>,
}

/// Menubar widget.
///
/// Separate renderer for the popup part of the menubar.
#[derive(Default, Clone)]
pub struct RMenuPopup<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,
    popup: PopupMenu<'a>,
}

/// State for the menubar.
#[derive(Debug, Default, Clone)]
pub struct RMenuBarState {
    pub menu: RMenuLineState,
    pub popup_active: bool,
    pub popup: PopupMenuState,

    pub area: Rect,
    pub z_areas: [ZRect; 2],
}

impl<'a> RMenuBar<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.menu = self.menu.title(title);
        self
    }

    /// Menu-Structure
    pub fn menu(mut self, structure: &'a dyn MenuStructure<'a>) -> Self {
        for (m, n) in structure.menus() {
            self.menu = self.menu.add(m, n);
        }
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.menu = self.menu.styles(styles.clone());
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.menu = self.menu.style(style);
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style(mut self, style: Style) -> Self {
        self.menu = self.menu.title_style(style);
        self
    }

    /// Selection
    #[inline]
    pub fn select_style(mut self, style: Style) -> Self {
        self.menu = self.menu.select_style(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.menu = self.menu.focus_style(style);
        self
    }
}

impl<'a> Debug for RMenuPopup<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RMenuPopup")
            .field("popup", &self.popup)
            .finish()
    }
}

impl<'a> RMenuPopup<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Menu.
    pub fn menu(mut self, structure: &'a dyn MenuStructure<'a>) -> Self {
        self.structure = Some(structure);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn width(mut self, width: u16) -> Self {
        self.popup = self.popup.width(width);
        self
    }

    /// Placement relative to the render-area.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.popup = self.popup.placement(placement);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.popup = self.popup.styles(styles.clone());
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.popup = self.popup.style(style);
        self
    }

    /// Focus/Selection style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.popup = self.popup.focus_style(style);
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
        self
    }
}

impl<'a> StatefulWidgetRef for RMenuBar<'a> {
    type State = RMenuBarState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for RMenuBar<'a> {
    type State = RMenuBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(&self, area, buf, state);
    }
}

fn render_menubar(widget: &RMenuBar<'_>, area: Rect, buf: &mut Buffer, state: &mut RMenuBarState) {
    widget.menu.render_ref(area, buf, &mut state.menu);

    // Combined area + each part with a z-index.
    state.area = state.menu.widget.area;
    state.z_areas = [ZRect::from((0, state.menu.widget.area)), ZRect::default()];
}

impl<'a> StatefulWidgetRef for RMenuPopup<'a> {
    type State = RMenuBarState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for RMenuPopup<'a> {
    type State = RMenuBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(&self, area, buf, state);
    }
}

fn render_menu_popup(
    widget: &RMenuPopup<'_>,
    _area: Rect,
    buf: &mut Buffer,
    state: &mut RMenuBarState,
) {
    // Combined area + each part with a z-index.
    state.area = state.menu.widget.area;
    state.z_areas = [ZRect::from((0, state.menu.widget.area)), ZRect::default()];

    let Some(selected) = state.menu.selected() else {
        return;
    };
    let Some(structure) = widget.structure else {
        return;
    };

    if state.is_focused() && state.popup_active {
        let mut len = 0;
        let mut popup = widget.popup.clone(); // TODO: ???
        for (item, navchar) in structure.submenu(selected) {
            popup = popup.add(item, navchar);
            len += 1;
        }

        if len > 0 {
            let area = state.menu.widget.item_areas[selected];
            popup.render(area, buf, &mut state.popup);

            // Combined area + each part with a z-index.
            state.area = state.menu.widget.area.union(state.popup.area);
            state.z_areas[1] = ZRect::from((1, state.popup.area));
        }
    } else {
        state.popup = Default::default();
    }
}

impl RMenuBarState {
    /// State.
    /// For the specifics use the public fields `menu` and `popup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submenu visible/active.
    pub fn popup_active(&self) -> bool {
        self.popup_active
    }

    /// Submenu visible/active.
    pub fn set_popup_active(&mut self, active: bool) {
        self.popup_active = active;
    }
}

impl HasFocusFlag for RMenuBarState {
    fn focus(&self) -> &FocusFlag {
        &self.menu.widget.focus
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn z_areas(&self) -> &[ZRect] {
        &self.z_areas
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for RMenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        if let Some(selected) = self.menu.selected() {
            if self.popup_active() {
                match self.popup.handle(event, Popup) {
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => MenuOutcome::MenuActivated(selected, n),
                    r => r,
                }
            } else {
                MenuOutcome::NotUsed
            }
        } else {
            MenuOutcome::NotUsed
        }
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, MenuOutcome> for RMenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: FocusKeys) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        let old_selected = self.menu.selected();
        let r = if self.menu.is_focused() {
            self.menu.handle(event, FocusKeys)
        } else {
            self.menu.handle(event, MouseOnly)
        };
        match r {
            MenuOutcome::Selected(n) => {
                if old_selected == Some(n) {
                    self.set_popup_active(!self.popup_active());
                }
            }
            _ => {}
        };

        r
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for RMenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        flow!(if let Some(selected) = self.menu.selected() {
            if self.popup_active() {
                match self.popup.handle(event, MouseOnly) {
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => MenuOutcome::MenuActivated(selected, n),
                    r => r,
                }
            } else {
                MenuOutcome::NotUsed
            }
        } else {
            MenuOutcome::NotUsed
        });

        let old_selected = self.menu.selected();
        let r = self.menu.handle(event, MouseOnly);
        match r {
            MenuOutcome::Selected(n) => {
                if old_selected == Some(n) {
                    self.set_popup_active(!self.popup_active());
                }
            }
            _ => {}
        };

        r
    }
}
