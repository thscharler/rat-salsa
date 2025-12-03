//! This widget will render a menubar and one level of menus.
//!
//! It is not a Widget itself, instead it will [split into](Menubar::into_widgets)
//! a [MenubarLine] and a [MenubarPopup] widget that can be rendered.
//! The MenubarLine can be rendered in its designated area anytime.
//! The MenubarPopup must be rendered at the end of rendering,
//! for it to be able to render above the other widgets.
//!
//! Event handling for the menubar must happen before handling
//! events for the widgets that might be rendered in the background.
//! Otherwise, mouse navigation will not work correctly.
//!
//! The structure of the menubar is defined with the trait
//! [MenuStructure], and there is [StaticMenu](crate::StaticMenu)
//! which can define the structure as static data.
//!
//! [Example](https://github.com/thscharler/rat-salsa/blob/master/rat-widget/examples/menubar1.rs)
//!
#![allow(clippy::uninlined_format_args)]
use crate::_private::NonExhaustive;
use crate::event::MenuOutcome;
use crate::menuline::{MenuLine, MenuLineState};
use crate::popup_menu::{PopupMenu, PopupMenuState};
use crate::{MenuStructure, MenuStyle};
use rat_cursor::HasScreenCursor;
use rat_event::{ConsumedEvent, HandleEvent, MouseOnly, Popup, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_popup::Placement;
use rat_reloc::RelocatableState;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::Debug;

/// Menubar widget.
///
/// This handles the configuration only, to get the widgets for rendering
/// call [Menubar::into_widgets] and use both results for rendering.
#[derive(Debug, Clone)]
pub struct Menubar<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,

    menu: MenuLine<'a>,

    popup_alignment: Alignment,
    popup_placement: Placement,
    popup_offset: Option<(i16, i16)>,
    popup: PopupMenu<'a>,
}

/// Menubar line widget.
///
/// This will render the main menu bar.
#[derive(Debug, Clone)]
pub struct MenubarLine<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,
    menu: MenuLine<'a>,
}

/// Menubar popup widget.
///
/// Separate renderer for the popup part of the menu bar.
#[derive(Debug, Clone)]
pub struct MenubarPopup<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,
    popup_alignment: Alignment,
    popup_placement: Placement,
    popup_offset: Option<(i16, i16)>,
    popup: PopupMenu<'a>,
}

/// State & event-handling.
#[derive(Debug, Clone)]
pub struct MenubarState {
    /// Area for the main-menubar.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// State for the menu.
    pub bar: MenuLineState,
    /// State for the last rendered popup menu.
    pub popup: PopupMenuState,

    /// Rendering is split into base-widget and menu-popup.
    /// Relocate after rendering the popup.
    relocate_popup: bool,

    pub non_exhaustive: NonExhaustive,
}

impl Default for Menubar<'_> {
    fn default() -> Self {
        Self {
            structure: Default::default(),
            menu: Default::default(),
            popup_alignment: Alignment::Left,
            popup_placement: Placement::AboveOrBelow,
            popup_offset: Default::default(),
            popup: Default::default(),
        }
    }
}

impl<'a> Menubar<'a> {
    #[inline]
    pub fn new(structure: &'a dyn MenuStructure<'a>) -> Self {
        Self {
            structure: Some(structure),
            ..Default::default()
        }
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.menu = self.menu.style(style);
        self
    }

    /// Block.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.menu = self.menu.block(block);
        self
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.menu = self.menu.title(title);
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style(mut self, style: Style) -> Self {
        self.menu = self.menu.title_style(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.menu = self.menu.focus_style(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn right_style(mut self, style: Style) -> Self {
        self.menu = self.menu.right_style(style);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    #[inline]
    pub fn popup_width(mut self, width: u16) -> Self {
        self.popup = self.popup.menu_width(width);
        self
    }

    /// Placement relative to the render-area.
    #[inline]
    pub fn popup_alignment(mut self, alignment: Alignment) -> Self {
        self.popup_alignment = alignment;
        self
    }

    /// Placement relative to the render-area.
    #[inline]
    pub fn popup_offset(mut self, offset: (i16, i16)) -> Self {
        self.popup_offset = Some(offset);
        self
    }

    /// Placement relative to the render-area.
    #[inline]
    pub fn popup_placement(mut self, placement: Placement) -> Self {
        self.popup_placement = placement;
        self
    }

    /// Base style for the popup-menu.
    #[inline]
    pub fn popup_style(mut self, style: Style) -> Self {
        self.popup = self.popup.style(style);
        self
    }

    /// Block for the popup-menu.
    #[inline]
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
        self
    }

    /// Selection + Focus for the popup-menu.
    #[inline]
    pub fn popup_focus_style(mut self, style: Style) -> Self {
        self.popup = self.popup.focus_style(style);
        self
    }

    /// Hotkey for the popup-menu.
    #[inline]
    pub fn popup_right_style(mut self, style: Style) -> Self {
        self.popup = self.popup.right_style(style);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.menu = self.menu.styles(styles.clone());
        self.popup = self.popup.styles(styles.clone());

        if let Some(alignment) = styles.popup.alignment {
            self.popup_alignment = alignment;
        }
        if let Some(placement) = styles.popup.placement {
            self.popup_placement = placement;
        }
        if let Some(offset) = styles.popup.offset {
            self.popup_offset = Some(offset);
        }
        self
    }

    /// Create the widgets for the Menubar. This returns a widget
    /// for the menu-line and for the menu-popup.
    ///
    /// The menu-popup should be rendered after all widgets
    /// that might be below the popup have been rendered.
    #[inline]
    pub fn into_widgets(self) -> (MenubarLine<'a>, MenubarPopup<'a>) {
        (
            MenubarLine {
                structure: self.structure,
                menu: self.menu,
            },
            MenubarPopup {
                structure: self.structure,
                popup_alignment: self.popup_alignment,
                popup_placement: self.popup_placement,
                popup_offset: self.popup_offset,
                popup: self.popup,
            },
        )
    }
}

impl<'a> StatefulWidget for Menubar<'a> {
    type State = MenubarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (menu, popup) = self.into_widgets();
        menu.render(area, buf, state);
        popup.render(Rect::default(), buf, state);
        // direct rendering of menubar + popup.
        // relocate() will be called, not relocate_popup()
        state.relocate_popup = false;
    }
}

impl StatefulWidget for MenubarLine<'_> {
    type State = MenubarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(self, area, buf, state);
    }
}

fn render_menubar(
    mut widget: MenubarLine<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut MenubarState,
) {
    if let Some(structure) = &widget.structure {
        structure.menus(&mut widget.menu.menu);
    }
    widget.menu.render(area, buf, &mut state.bar);
    // Area of the main bar.
    state.area = state.bar.area;
    state.relocate_popup = true;
}

impl StatefulWidget for MenubarPopup<'_> {
    type State = MenubarState;

    fn render(self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(self, buf, state);
    }
}

fn render_menu_popup(mut widget: MenubarPopup<'_>, buf: &mut Buffer, state: &mut MenubarState) {
    let Some(selected) = state.bar.selected() else {
        return;
    };
    let Some(structure) = widget.structure else {
        return;
    };

    if state.popup.is_active() {
        let item = state.bar.item_areas[selected];

        let popup_padding = widget.popup.get_block_padding();
        let sub_offset = if let Some(offset) = widget.popup_offset {
            offset
        } else {
            (-(popup_padding.left as i16 + 1), 0)
        };

        widget.popup = widget
            .popup
            .constraint(
                widget
                    .popup_placement
                    .into_constraint(widget.popup_alignment, item),
            )
            .offset(sub_offset);
        structure.submenu(selected, &mut widget.popup.menu);

        if !widget.popup.menu.items.is_empty() {
            let area = state.bar.item_areas[selected];
            widget.popup.render(area, buf, &mut state.popup);
        }
    } else {
        state.popup = Default::default();
    }
}

impl MenubarState {
    /// State.
    /// For the specifics use the public fields `menu` and `popup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// New state with a focus name.
    pub fn named(name: &'static str) -> Self {
        Self {
            bar: MenuLineState::named(format!("{}.bar", name).to_string().leak()),
            popup: PopupMenuState::new(),
            ..Default::default()
        }
    }

    /// Submenu visible/active.
    pub fn popup_active(&self) -> bool {
        self.popup.is_active()
    }

    /// Submenu visible/active.
    pub fn set_popup_active(&mut self, active: bool) {
        self.popup.set_active(active);
    }

    /// Set the z-value for the popup-menu.
    ///
    /// This is the z-index used when adding the menubar to
    /// the focus list.
    pub fn set_popup_z(&mut self, z: u16) {
        self.popup.set_popup_z(z)
    }

    /// The z-index for the popup-menu.
    pub fn popup_z(&self) -> u16 {
        self.popup.popup_z()
    }

    /// Selected as menu/submenu
    pub fn selected(&self) -> (Option<usize>, Option<usize>) {
        (self.bar.selected, self.popup.selected)
    }
}

impl Default for MenubarState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            bar: Default::default(),
            popup: Default::default(),
            relocate_popup: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for MenubarState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget_with_flags(self.focus(), self.area(), self.area_z(), self.navigable());
        builder.widget_with_flags(
            self.focus(),
            self.popup.popup.area,
            self.popup.popup.area_z,
            Navigation::Mouse,
        );
    }

    fn focus(&self) -> FocusFlag {
        self.bar.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HasScreenCursor for MenubarState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl RelocatableState for MenubarState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        if !self.relocate_popup {
            self.area.relocate(shift, clip);
            self.bar.relocate(shift, clip);
            self.popup.relocate(shift, clip);
            self.popup.relocate_popup(shift, clip);
        }
    }

    fn relocate_popup(&mut self, shift: (i16, i16), clip: Rect) {
        if self.relocate_popup {
            self.relocate_popup = false;
            self.area.relocate(shift, clip);
            self.bar.relocate(shift, clip);
            self.popup.relocate(shift, clip);
            self.popup.relocate_popup(shift, clip);
        }
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for MenubarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        handle_menubar(self, event, Popup, Regular)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for MenubarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> MenuOutcome {
        handle_menubar(self, event, MouseOnly, MouseOnly)
    }
}

fn handle_menubar<Q1, Q2>(
    state: &mut MenubarState,
    event: &crossterm::event::Event,
    qualifier1: Q1,
    qualifier2: Q2,
) -> MenuOutcome
where
    PopupMenuState: HandleEvent<crossterm::event::Event, Q1, MenuOutcome>,
    MenuLineState: HandleEvent<crossterm::event::Event, Q2, MenuOutcome>,
    MenuLineState: HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome>,
{
    if !state.is_focused() {
        state.set_popup_active(false);
    }

    if state.bar.is_focused() {
        let mut r = if let Some(selected) = state.bar.selected() {
            if state.popup_active() {
                match state.popup.handle(event, qualifier1) {
                    MenuOutcome::Hide => {
                        // only hide on focus lost. ignore this one.
                        MenuOutcome::Continue
                    }
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => MenuOutcome::MenuActivated(selected, n),
                    r => r,
                }
            } else {
                MenuOutcome::Continue
            }
        } else {
            MenuOutcome::Continue
        };

        r = r.or_else(|| {
            let old_selected = state.bar.selected();
            let r = state.bar.handle(event, qualifier2);
            match r {
                MenuOutcome::Selected(_) => {
                    if state.bar.selected == old_selected {
                        state.popup.flip_active();
                    } else {
                        state.popup.select(None);
                        state.popup.set_active(true);
                    }
                }
                MenuOutcome::Activated(_) => {
                    state.popup.flip_active();
                }
                _ => {}
            }
            r
        });

        r
    } else {
        state.bar.handle(event, MouseOnly)
    }
}

/// Handle menu events for the popup-menu.
///
/// This one is separate, as it needs to be called before other event-handlers
/// to cope with overlapping regions.
///
/// focus - is the menubar focused?
pub fn handle_events(
    state: &mut MenubarState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.bar.focus.set(focus);
    state.handle(event, Popup)
}

/// Handle menu events for the popup-menu.
///
/// This one is separate, as it needs to be called before other event-handlers
/// to cope with overlapping regions.
///
/// focus - is the menubar focused?
pub fn handle_popup_events(
    state: &mut MenubarState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.bar.focus.set(focus);
    state.handle(event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut MenubarState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, MouseOnly)
}
