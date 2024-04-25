use crate::_private::NonExhaustive;
use crate::{
    ct_event, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling,
    MouseOnly, ScrollParam, ScrolledWidget,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;
use std::fmt::Debug;
use std::hash::Hash;
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Debug, Default, Clone)]
pub struct TreeExt<'a, Identifier> {
    items: Vec<TreeItem<'a, Identifier>>,
    block: Option<Block<'a>>,
    style: Option<Style>,
    highlight_style: Option<Style>,
    highlight_symbol: Option<&'a str>,
    node_closed_symbol: Option<&'a str>,
    node_open_symbol: Option<&'a str>,
    node_no_children_symbol: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct TreeExtState<Identifier> {
    pub area: Rect,
    pub focus: FocusFlag,

    pub max_v_offset: usize,
    pub v_page_len: usize,

    pub widget: TreeState<Identifier>,

    pub non_exhaustive: NonExhaustive,
}

impl<'a, Identifier> TreeExt<'a, Identifier> {
    pub fn new(items: Vec<TreeItem<'a, Identifier>>) -> Self {
        Self {
            items,
            block: None,
            style: None,
            highlight_style: None,
            highlight_symbol: None,
            node_closed_symbol: None,
            node_open_symbol: None,
            node_no_children_symbol: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = Some(style);
        self
    }

    pub fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.highlight_symbol = Some(highlight_symbol);
        self
    }

    pub fn node_closed_symbol(mut self, symbol: &'a str) -> Self {
        self.node_closed_symbol = Some(symbol);
        self
    }

    pub const fn node_open_symbol(mut self, symbol: &'a str) -> Self {
        self.node_open_symbol = Some(symbol);
        self
    }

    pub const fn node_no_children_symbol(mut self, symbol: &'a str) -> Self {
        self.node_no_children_symbol = Some(symbol);
        self
    }
}

impl<'a, Identifier> ScrolledWidget<TreeExtState<Identifier>> for TreeExt<'a, Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn need_scroll(&self, area: Rect, state: &mut TreeExtState<Identifier>) -> ScrollParam {
        let flattened = state.widget.flatten(&self.items);

        let mut has_vscroll = false;
        let mut height = 0;
        for item in &flattened {
            height += item.item.height();
            if height > area.height as usize {
                has_vscroll = true;
                break;
            }
        }

        ScrollParam {
            has_hscroll: false,
            has_vscroll,
        }
    }
}

impl<'a, Identifier> StatefulWidget for TreeExt<'a, Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    type State = TreeExtState<Identifier>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let flattened = state.widget.flatten(&self.items);

        // calculate max offset
        let mut n = 0;
        let mut height = 0;
        for item in flattened.iter().rev() {
            height += item.item.height();
            if height > area.height as usize {
                break;
            }
            n += 1;
        }
        state.max_v_offset = flattened.len() - n;

        // calculate current page
        let ensure_index_in_view = if
        /*state.ensure_selected_in_view_on_next_render &&*/
        !state.widget.selected().is_empty() {
            flattened
                .iter()
                .position(|flattened| flattened.identifier == state.widget.selected())
        } else {
            None
        };

        // Ensure last line is still visible
        let mut start = state
            .widget
            .get_offset()
            .min(flattened.len().saturating_sub(1));
        if let Some(ensure_index_in_view) = ensure_index_in_view {
            start = start.min(ensure_index_in_view);
        }

        let mut n = 0;
        let mut height = 0;
        for item in flattened.iter().skip(start) {
            height += item.item.height();
            if height > area.height as usize {
                break;
            }
            n += 1;
        }
        state.v_page_len = n;

        // render
        let mut tree = Tree::new(self.items).unwrap_or(Tree::new(Vec::new()).expect("tree"));
        if let Some(block) = self.block {
            tree = tree.block(block);
        }
        if let Some(style) = self.style {
            tree = tree.style(style);
        }
        if let Some(highlight_style) = self.highlight_style {
            tree = tree.highlight_style(highlight_style);
        }
        if let Some(highlight_symbol) = self.highlight_symbol {
            tree = tree.highlight_symbol(highlight_symbol);
        }
        if let Some(node_closed_symbol) = self.node_closed_symbol {
            tree = tree.node_closed_symbol(node_closed_symbol);
        }
        if let Some(node_open_symbol) = self.node_open_symbol {
            tree = tree.node_open_symbol(node_open_symbol);
        }
        if let Some(node_no_children_symbol) = self.node_no_children_symbol {
            tree = tree.node_no_children_symbol(node_no_children_symbol);
        }
        tree.render(area, buf, &mut state.widget);
    }
}

impl<Identifier: Default> Default for TreeExtState<Identifier> {
    fn default() -> Self {
        Self {
            area: Default::default(),
            focus: Default::default(),
            max_v_offset: 0,
            v_page_len: 0,
            widget: TreeState::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Identifier> HasScrolling for TreeExtState<Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn max_v_offset(&self) -> usize {
        self.max_v_offset
    }

    fn max_h_offset(&self) -> usize {
        0
    }

    fn v_page_len(&self) -> usize {
        self.v_page_len
    }

    fn h_page_len(&self) -> usize {
        0
    }

    fn v_offset(&self) -> usize {
        self.widget.get_offset()
    }

    fn h_offset(&self) -> usize {
        0
    }

    fn set_v_offset(&mut self, offset: usize) {
        let old_offset = self.v_offset();
        if old_offset < offset {
            self.widget.scroll_down(offset - old_offset);
        } else {
            self.widget.scroll_up(old_offset - offset);
        }
    }

    fn set_h_offset(&mut self, _offset: usize) {
        // noop
    }
}

impl<Identifier> HasFocusFlag for TreeExtState<Identifier> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<Identifier, A, E> HandleCrossterm<ControlUI<A, E>, DefaultKeys> for TreeExtState<Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn handle(&mut self, event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        if self.is_focused() {
            match event {
                ct_event!(keycode press Left) => {
                    self.widget.key_left();
                    ControlUI::Change
                }
                ct_event!(keycode press Right) => {
                    self.widget.key_right();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    // TODO: doesn't work for now.
                    // self.widget.key_up();
                    ControlUI::Change
                }
                ct_event!(keycode press Down) => {
                    // TODO: doesn't work for now.
                    // self.widget.key_down();
                    ControlUI::Change
                }
                _ => ControlUI::Continue,
            }
        } else {
            ControlUI::Continue
        }
    }
}

impl<Identifier, A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TreeExtState<Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> ControlUI<A, E> {
        // TODO: do something
        ControlUI::Continue
    }
}
