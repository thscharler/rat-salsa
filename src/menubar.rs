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
use crate::popup_menu::{PopupMenu, PopupMenuState};
use crate::{MenuStructure, MenuStyle};
use rat_event::{flow, HandleEvent, MouseOnly, Popup, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_popup::Placement;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::Debug;

/// Menubar widget.
/// This handles the configuration only, to get the widgets for rendering
/// call [Menubar::into_widgets] and use both results for rendering.
#[derive(Debug, Clone)]
pub struct Menubar<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,

    title: Line<'a>,
    style: Style,
    title_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    highlight_style: Option<Style>,
    disabled_style: Option<Style>,
    right_style: Option<Style>,

    popup_placement: Placement,
    popup: PopupMenu<'a>,
}

/// Menubar line widget.
/// This implements the actual render function.
#[derive(Debug, Clone)]
pub struct MenubarLine<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,

    title: Line<'a>,
    style: Style,
    title_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    highlight_style: Option<Style>,
    disabled_style: Option<Style>,
    right_style: Option<Style>,
}

/// Menubar popup widget.
/// Separate renderer for the popup part of the menubar.
#[derive(Debug, Clone)]
pub struct MenubarPopup<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,

    style: Style,
    focus_style: Option<Style>,
    highlight_style: Option<Style>,
    disabled_style: Option<Style>,
    right_style: Option<Style>,

    popup_placement: Placement,
    popup: PopupMenu<'a>,
}

/// State & event-handling.
#[derive(Debug, Default, Clone)]
pub struct MenubarState {
    /// Area for the menubar.
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// State for the menu.
    pub bar: MenuLineState,
    /// State for the last rendered popup menu.
    pub popup: PopupMenuState,
}

impl<'a> Default for Menubar<'a> {
    fn default() -> Self {
        Self {
            structure: None,
            title: Default::default(),
            style: Default::default(),
            title_style: None,
            select_style: None,
            focus_style: None,
            highlight_style: None,
            disabled_style: None,
            right_style: None,
            popup_placement: Placement::AboveOrBelow,
            popup: Default::default(),
        }
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
        self.popup = self.popup.styles(styles.clone());

        self.style = styles.style;
        if styles.highlight.is_some() {
            self.highlight_style = styles.highlight;
        }
        if styles.disabled.is_some() {
            self.disabled_style = styles.disabled;
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
        if styles.right.is_some() {
            self.right_style = styles.right;
        }
        if let Some(placement) = styles.popup.placement {
            self.popup_placement = placement;
        }
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

    /// Selection + Focus
    #[inline]
    pub fn right_style(mut self, style: Style) -> Self {
        self.right_style = Some(style);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn popup_width(mut self, width: u16) -> Self {
        self.popup = self.popup.width(width);
        self
    }

    /// Placement relative to the render-area.
    pub fn popup_placement(mut self, placement: Placement) -> Self {
        self.popup_placement = placement;
        self
    }

    /// Block for borders.
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
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
                structure: self.structure,
                title: self.title,
                style: self.style,
                title_style: self.title_style,
                select_style: self.select_style,
                focus_style: self.focus_style,
                highlight_style: self.highlight_style,
                disabled_style: self.disabled_style,
                right_style: self.right_style,
            },
            MenubarPopup {
                structure: self.structure,
                style: self.style,
                focus_style: self.focus_style,
                highlight_style: self.highlight_style,
                disabled_style: self.disabled_style,
                right_style: self.right_style,
                popup_placement: self.popup_placement,
                popup: self.popup,
            },
        )
    }
}

impl<'a> StatefulWidget for MenubarLine<'a> {
    type State = MenubarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(&self, area, buf, state);
    }
}

fn render_menubar(
    widget: &MenubarLine<'_>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut MenubarState,
) {
    let mut menu = MenuLine::new()
        .title(widget.title.clone())
        .style(widget.style)
        .title_style_opt(widget.title_style)
        .select_style_opt(widget.select_style)
        .focus_style_opt(widget.focus_style)
        .highlight_style_opt(widget.highlight_style)
        .disabled_style_opt(widget.disabled_style)
        .right_style_opt(widget.right_style);

    if let Some(structure) = &widget.structure {
        structure.menus(&mut menu.menu);
    }
    menu.render(area, buf, &mut state.bar);

    // Combined area + each part with a z-index.
    state.area = state.bar.area;
}

impl<'a> StatefulWidget for MenubarPopup<'a> {
    type State = MenubarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(self, area, buf, state);
    }
}

fn render_menu_popup(
    widget: MenubarPopup<'_>,
    _area: Rect,
    buf: &mut Buffer,
    state: &mut MenubarState,
) {
    // Combined area + each part with a z-index.
    state.area = state.bar.area;

    let Some(selected) = state.bar.selected() else {
        return;
    };
    let Some(structure) = widget.structure else {
        return;
    };

    if state.popup.is_active() {
        let item = state.bar.item_areas[selected];

        let popup_padding = widget.popup.get_block_padding();
        let sub_offset = (-(popup_padding.left as i16 + 1), 0);

        let mut popup = widget
            .popup
            .constraint(widget.popup_placement.into_constraint(item))
            .offset(sub_offset)
            .style(widget.style)
            .focus_style_opt(widget.focus_style)
            .highlight_style_opt(widget.highlight_style)
            .disabled_style_opt(widget.disabled_style)
            .right_style_opt(widget.right_style);

        structure.submenu(selected, &mut popup.menu);

        if !popup.menu.items.is_empty() {
            let area = state.bar.item_areas[selected];
            popup.render(area, buf, &mut state.popup);

            // Combined area + each part with a z-index.
            state.area = state.bar.area.union(state.popup.popup.area);
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

    /// Selected as menu/submenu
    pub fn selected(&self) -> (Option<usize>, Option<usize>) {
        (self.bar.selected, self.popup.selected)
    }
}

impl HasFocus for MenubarState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.add_widget(self.focus(), self.area(), self.area_z(), self.navigable());
        builder.add_widget(self.focus(), self.popup.popup.area, 1, Navigation::Mouse);
    }

    fn focus(&self) -> FocusFlag {
        self.bar.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for MenubarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        if let Some(selected) = self.bar.selected() {
            if self.popup_active() {
                match self.popup.handle(event, Popup) {
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
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, MenuOutcome> for MenubarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> MenuOutcome {
        // todo: too spooky?
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
                        self.popup.select(None);
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

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for MenubarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        flow!(if let Some(selected) = self.bar.selected() {
            if self.popup_active() {
                match self.popup.handle(event, MouseOnly) {
                    MenuOutcome::Hide => {
                        self.set_popup_active(false);
                        MenuOutcome::Changed
                    }
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
    state: &mut MenubarState,
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
    state: &mut MenubarState,
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
