//!
//! A popup-menu.
//!
//! It diverges from other widgets as it doesn't draw
//! *inside* the given area but aims to stay *outside* of it.
//!
//! You can give a [PopupConstraint] where the popup-menu should appear
//! relative to the given area.
//!
//! If you want it to appear at a mouse-click position, use a
//! `Rect::new(mouse_x, mouse_y, 0,0)` area.
//! If you want it to appear next to a given widget, use
//! the widgets drawing area.
//!
//! If no special boundary is set, the widget tries to stay
//! inside the `buffer.area`.

use crate::_private::NonExhaustive;
use crate::event::MenuOutcome;
use crate::util::revert_style;
use crate::{MenuBuilder, MenuItem, MenuStyle, Separator};
use rat_event::util::MouseFlags;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Popup};
use rat_popup::event::PopupOutcome;
pub use rat_popup::PopupConstraint;
use rat_popup::{PopupCore, PopupCoreState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::StatefulWidget;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, Padding, Widget};
use std::cmp::max;
use std::mem;
use unicode_segmentation::UnicodeSegmentation;

/// Popup menu.
#[derive(Debug, Default, Clone)]
pub struct PopupMenu<'a> {
    pub(crate) menu: MenuBuilder<'a>,

    width: Option<u16>,
    popup: PopupCore<'a>,

    style: Style,
    highlight_style: Option<Style>,
    disabled_style: Option<Style>,
    right_style: Option<Style>,
    focus_style: Option<Style>,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct PopupMenuState {
    /// Popup data.
    pub popup: PopupCoreState,
    /// Areas for each item.
    /// __readonly__. renewed for each render.
    pub item_areas: Vec<Rect>,
    /// Area for the separator after each item.
    /// The area has height 0 if there is no separator.
    /// __readonly__. renewed for each render.
    pub sep_areas: Vec<Rect>,
    /// Letter navigation
    /// __readonly__. renewed for each render.
    pub navchar: Vec<Option<char>>,
    /// Disabled menu-items.
    pub disabled: Vec<bool>,

    /// Selected item.
    /// __read+write__
    pub selected: Option<usize>,

    /// Mouse flags
    /// __used for mouse interaction__
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for PopupMenuState {
    fn default() -> Self {
        Self {
            popup: Default::default(),
            item_areas: vec![],
            sep_areas: vec![],
            navchar: vec![],
            disabled: vec![],
            selected: None,
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> PopupMenu<'a> {
    fn size(&self) -> Size {
        let width = if let Some(width) = self.width {
            width
        } else {
            let text_width = self
                .menu
                .items
                .iter()
                .map(|v| (v.item_width() * 3) / 2 + v.right_width())
                .max();
            text_width.unwrap_or(10)
        };
        let height = self.menu.items.iter().map(MenuItem::height).sum::<u16>();

        let block = self.popup.get_block_size();

        #[allow(clippy::if_same_then_else)]
        let vertical_padding = if block.height == 0 { 2 } else { 0 };
        let horizontal_padding = 2;

        Size::new(
            width + horizontal_padding + block.width,
            height + vertical_padding + block.height,
        )
    }

    fn layout(&self, area: Rect, inner: Rect, state: &mut PopupMenuState) {
        let block = Size::new(area.width - inner.width, area.height - inner.height);

        // add text padding.
        #[allow(clippy::if_same_then_else)]
        let vert_offset = if block.height == 0 { 1 } else { 0 };
        let horiz_offset = 1;
        let horiz_offset_sep = 0;

        state.item_areas.clear();
        state.sep_areas.clear();

        let mut row = 0;

        for item in &self.menu.items {
            state.item_areas.push(Rect::new(
                inner.x + horiz_offset,
                inner.y + row + vert_offset,
                inner.width.saturating_sub(2 * horiz_offset),
                1,
            ));
            state.sep_areas.push(Rect::new(
                inner.x + horiz_offset_sep,
                inner.y + row + 1 + vert_offset,
                inner.width.saturating_sub(2 * horiz_offset_sep),
                if item.separator.is_some() { 1 } else { 0 },
            ));

            row += item.height();
        }
    }
}

impl<'a> PopupMenu<'a> {
    /// New, empty.
    pub fn new() -> Self {
        Default::default()
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

    /// Sets the separator for the last item added.
    /// If there is none adds this as an empty menu-item.
    pub fn separator(mut self, separator: Separator) -> Self {
        self.menu.separator(separator);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn width_opt(mut self, width: Option<u16>) -> Self {
        self.width = width;
        self
    }

    /// Set relative placement.
    pub fn constraint(mut self, placement: PopupConstraint) -> Self {
        self.popup = self.popup.constraint(placement);
        self
    }

    /// Adds an extra offset.
    pub fn offset(mut self, offset: (i16, i16)) -> Self {
        self.popup = self.popup.offset(offset);
        self
    }

    /// Adds an extra x offset.
    pub fn x_offset(mut self, offset: i16) -> Self {
        self.popup = self.popup.x_offset(offset);
        self
    }

    /// Adds an extra y offset.
    pub fn y_offset(mut self, offset: i16) -> Self {
        self.popup = self.popup.y_offset(offset);
        self
    }

    /// Set outer bounds for the popup-menu.
    /// If not set, the [Buffer::area] is used as outer bounds.
    pub fn boundary(mut self, boundary: Rect) -> Self {
        self.popup = self.popup.boundary(boundary);
        self
    }

    /// Set a style-set.
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        self.popup = self.popup.styles(styles.popup);
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
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.popup = self.popup.style(style);
        self.style = style;
        self
    }

    /// Highlight style.
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = Some(style);
        self
    }

    /// Highlight style.
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

    /// Focus/Selection style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Focus/Selection style.
    pub fn focus_style_opt(mut self, style: Option<Style>) -> Self {
        self.focus_style = style;
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
        self
    }

    /// Block for borders.
    pub fn block_opt(mut self, block: Option<Block<'a>>) -> Self {
        self.popup = self.popup.block_opt(block);
        self
    }

    /// Get the padding the block imposes as a Size.
    pub fn get_block_size(&self) -> Size {
        self.popup.get_block_size()
    }

    /// Get the padding the block imposes as a Size.
    pub fn get_block_padding(&self) -> Padding {
        self.popup.get_block_padding()
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for PopupMenu<'a> {
    type State = PopupMenuState;

    fn render_ref(&self, mut area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.is_active() {
            state.clear_areas();
            return;
        }

        state.navchar = self.menu.items.iter().map(|v| v.navchar).collect();
        state.disabled = self.menu.items.iter().map(|v| v.disabled).collect();

        let size = self.size();
        area.width = size.width;
        area.height = size.height;

        self.popup.render_ref(area, buf, &mut state.popup);
        self.layout(state.popup.area, state.popup.widget_area, state);
        render_items(&self, buf, state)
    }
}

impl<'a> StatefulWidget for PopupMenu<'a> {
    type State = PopupMenuState;

    fn render(mut self, _area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !state.is_active() {
            state.clear_areas();
            return;
        }

        state.navchar = self.menu.items.iter().map(|v| v.navchar).collect();
        state.disabled = self.menu.items.iter().map(|v| v.disabled).collect();

        let size = self.size();
        let area = Rect::new(0, 0, size.width, size.height);

        mem::take(&mut self.popup).render(area, buf, &mut state.popup);
        self.layout(state.popup.area, state.popup.widget_area, state);
        render_items(&self, buf, state);
    }
}

fn render_items(widget: &PopupMenu<'_>, buf: &mut Buffer, state: &mut PopupMenuState) {
    let focus_style = if let Some(focus) = widget.focus_style {
        focus
    } else {
        revert_style(widget.style)
    };
    let highlight_style = if let Some(highlight_style) = widget.highlight_style {
        highlight_style
    } else {
        Style::new().underlined()
    };
    let right_style = if let Some(right_style) = widget.right_style {
        right_style
    } else {
        Style::default().italic()
    };
    let disabled_style = if let Some(disabled_style) = widget.disabled_style {
        disabled_style
    } else {
        widget.style
    };

    for (n, item) in widget.menu.items.iter().enumerate() {
        let mut item_area = state.item_areas[n];

        #[allow(clippy::collapsible_else_if)]
        let (style, right_style) = if state.selected == Some(n) {
            if item.disabled {
                (disabled_style, disabled_style.patch(right_style))
            } else {
                (focus_style, focus_style.patch(right_style))
            }
        } else {
            if item.disabled {
                (disabled_style, disabled_style.patch(right_style))
            } else {
                (widget.style, widget.style.patch(right_style))
            }
        };

        let item_line = if let Some(highlight) = item.highlight.clone() {
            Line::from_iter([
                Span::from(&item.item[..highlight.start - 1]), // account for _
                Span::from(&item.item[highlight.start..highlight.end]).style(highlight_style),
                Span::from(&item.item[highlight.end..]),
            ])
        } else {
            Line::from(item.item.as_ref())
        };
        item_line.style(style).render(item_area, buf);

        if !item.right.is_empty() {
            let right_width = item.right.graphemes(true).count() as u16;
            if right_width < item_area.width {
                let delta = item_area.width.saturating_sub(right_width);
                item_area.x += delta;
                item_area.width -= delta;
            }
            Span::from(item.right.as_ref())
                .style(right_style)
                .render(item_area, buf);
        }

        if let Some(separator) = item.separator {
            let sep_area = state.sep_areas[n];
            let sym = match separator {
                Separator::Empty => " ",
                Separator::Plain => "\u{2500}",
                Separator::Thick => "\u{2501}",
                Separator::Double => "\u{2550}",
                Separator::Dashed => "\u{2212}",
                Separator::Dotted => "\u{2508}",
            };
            for x in 0..sep_area.width {
                if let Some(cell) = buf.cell_mut((sep_area.x + x, sep_area.y)) {
                    cell.set_symbol(sym);
                }
            }
        }
    }
}

impl PopupMenuState {
    /// New
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Show the popup.
    pub fn flip_active(&mut self) {
        self.popup.flip_active();
    }

    /// Show the popup.
    pub fn is_active(&self) -> bool {
        self.popup.is_active()
    }

    /// Show the popup.
    pub fn set_active(&mut self, active: bool) {
        self.popup.set_active(active);
        if !active {
            self.clear_areas();
        }
    }

    /// Clear the areas.
    pub fn clear_areas(&mut self) {
        self.popup.clear_areas();
        self.sep_areas.clear();
        self.navchar.clear();
        self.item_areas.clear();
        self.disabled.clear();
    }

    /// Number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.item_areas.len()
    }

    /// Any items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.item_areas.is_empty()
    }

    /// Selected item.
    #[inline]
    pub fn select(&mut self, select: Option<usize>) -> bool {
        let old = self.selected;
        if let Some(select) = select {
            if self.disabled.get(select) == Some(&true) {
                return false;
            }
        }
        self.selected = select;
        old != self.selected
    }

    /// Selected item.
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Select the previous item.
    #[inline]
    pub fn prev_item(&mut self) -> bool {
        let old = self.selected;

        let idx = if let Some(start) = old {
            let mut idx = start;
            loop {
                if idx == 0 {
                    idx = start;
                    break;
                }
                idx -= 1;

                if !self.disabled[idx] {
                    break;
                }
            }
            idx
        } else {
            self.len().saturating_sub(1)
        };

        self.selected = Some(idx);
        old != self.selected
    }

    /// Select the next item.
    #[inline]
    pub fn next_item(&mut self) -> bool {
        let old = self.selected;

        let idx = if let Some(start) = old {
            let mut idx = start;
            loop {
                if idx + 1 == self.len() {
                    idx = start;
                    break;
                }
                idx += 1;

                if !self.disabled[idx] {
                    break;
                }
            }
            idx
        } else {
            0
        };

        self.selected = Some(idx);
        old != self.selected
    }

    /// Select by navigation key.
    #[inline]
    pub fn navigate(&mut self, c: char) -> MenuOutcome {
        let c = c.to_ascii_lowercase();
        for (i, cc) in self.navchar.iter().enumerate() {
            #[allow(clippy::collapsible_if)]
            if *cc == Some(c) {
                if !self.disabled[i] {
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
    #[inline]
    pub fn select_at(&mut self, pos: (u16, u16)) -> bool {
        let old_selected = self.selected;
        if let Some(idx) = self.mouse.item_at(&self.item_areas, pos.0, pos.1) {
            if !self.disabled[idx] {
                self.selected = Some(idx);
                self.selected != old_selected
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        self.mouse.item_at(&self.item_areas, pos.0, pos.1)
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for PopupMenuState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        let r0 = match self.popup.handle(event, Popup) {
            PopupOutcome::Hide => MenuOutcome::Hide,
            r => r.into(),
        };

        let r1 = if self.is_active() {
            match event {
                ct_event!(key press ANY-c) => {
                    let r = self.navigate(*c);
                    if matches!(r, MenuOutcome::Activated(_)) {
                        self.set_active(false);
                    }
                    r
                }
                ct_event!(keycode press Up) => {
                    if self.prev_item() {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Down) => {
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
                ct_event!(keycode press Esc) => {
                    self.set_active(false);
                    MenuOutcome::Changed
                }
                ct_event!(keycode press Enter) => {
                    if let Some(select) = self.selected {
                        self.set_active(false);
                        MenuOutcome::Activated(select)
                    } else {
                        MenuOutcome::Continue
                    }
                }

                ct_event!(key release _)
                | ct_event!(keycode release Up)
                | ct_event!(keycode release Down)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release Esc)
                | ct_event!(keycode release Enter) => MenuOutcome::Unchanged,

                _ => MenuOutcome::Continue,
            }
        } else {
            MenuOutcome::Continue
        };

        let r = max(r0, r1);

        if !r.is_consumed() {
            self.handle(event, MouseOnly)
        } else {
            r
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for PopupMenuState {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> MenuOutcome {
        if self.is_active() {
            match event {
                ct_event!(mouse moved for col, row)
                    if self.popup.widget_area.contains((*col, *row).into()) =>
                {
                    if self.select_at((*col, *row)) {
                        MenuOutcome::Selected(self.selected().expect("selection"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(mouse down Left for col, row)
                    if self.popup.widget_area.contains((*col, *row).into()) =>
                {
                    if self.item_at((*col, *row)).is_some() {
                        self.set_active(false);
                        MenuOutcome::Activated(self.selected().expect("selection"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                _ => MenuOutcome::Continue,
            }
        } else {
            MenuOutcome::Continue
        }
    }
}

/// Handle all events.
/// The assumption is, the popup-menu is focused or it is hidden.
/// This state must be handled outside of this widget.
pub fn handle_popup_events(
    state: &mut PopupMenuState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut PopupMenuState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, MouseOnly)
}
