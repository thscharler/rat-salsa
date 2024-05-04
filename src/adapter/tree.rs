use crate::_private::NonExhaustive;
use crate::events::{DefaultKeys, HandleEvent, MouseOnly, Outcome};
use crate::{ct_event, ScrollingOutcome, ScrollingState, ScrollingWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::widgets::Block;
use std::fmt::Debug;
use std::hash::Hash;
use tui_tree_widget::{TreeItem, TreeState};

/// This is currently dysfunctional.
/// Waiting for upstream changes.
#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct TreeS<'a, Identifier> {
    items: Vec<TreeItem<'a, Identifier>>,
    block: Option<Block<'a>>,
    style: Option<Style>,
    highlight_style: Option<Style>,
    highlight_symbol: Option<&'a str>,
    node_closed_symbol: Option<&'a str>,
    node_open_symbol: Option<&'a str>,
    node_no_children_symbol: Option<&'a str>,
}

#[derive(Debug)]
pub struct TreeSState<Identifier> {
    pub area: Rect,

    pub max_v_offset: usize,
    pub v_page_len: usize,

    pub widget: TreeState<Identifier>,

    pub non_exhaustive: NonExhaustive,
}

impl<'a, Identifier> TreeS<'a, Identifier> {
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

impl<'a, Identifier> ScrollingWidget<TreeSState<Identifier>> for TreeS<'a, Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn need_scroll(&self, area: Rect, state: &mut TreeSState<Identifier>) -> (bool, bool) {
        let height = state.widget.total_required_height(&self.items);
        let v_scroll = height > area.height as usize;
        (false, v_scroll)
    }
}

impl<'a, Identifier> StatefulWidget for TreeS<'a, Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    type State = TreeSState<Identifier>;

    fn render(self, _area: Rect, _buf: &mut Buffer, _state: &mut Self::State) {
        // state.area = area;
        //
        // let flattened = state.widget.flatten(&self.items);
        //
        // // calculate max offset
        // let mut n = 0;
        // let mut height = 0;
        // for item in flattened.iter().rev() {
        //     height += item.item.height();
        //     if height > area.height as usize {
        //         break;
        //     }
        //     n += 1;
        // }
        // state.max_v_offset = flattened.len() - n;
        //
        // // calculate current page
        // let ensure_index_in_view = if !state.widget.selected().is_empty() {
        //     flattened
        //         .iter()
        //         .position(|flattened| flattened.identifier == state.widget.selected())
        // } else {
        //     None
        // };
        //
        // // Ensure last line is still visible
        // let mut start = state
        //     .widget
        //     .get_offset()
        //     .min(flattened.len().saturating_sub(1));
        // if let Some(ensure_index_in_view) = ensure_index_in_view {
        //     start = start.min(ensure_index_in_view);
        // }
        //
        // let mut n = 0;
        // let mut height = 0;
        // for item in flattened.iter().skip(start) {
        //     height += item.item.height();
        //     if height > area.height as usize {
        //         break;
        //     }
        //     n += 1;
        // }
        // state.v_page_len = n;
        //
        // // render
        // let mut tree = Tree::new(self.items).unwrap_or(Tree::new(Vec::new()).expect("tree"));
        // if let Some(block) = self.block {
        //     tree = tree.block(block);
        // }
        // if let Some(style) = self.style {
        //     tree = tree.style(style);
        // }
        // if let Some(highlight_style) = self.highlight_style {
        //     tree = tree.highlight_style(highlight_style);
        // }
        // if let Some(highlight_symbol) = self.highlight_symbol {
        //     tree = tree.highlight_symbol(highlight_symbol);
        // }
        // if let Some(node_closed_symbol) = self.node_closed_symbol {
        //     tree = tree.node_closed_symbol(node_closed_symbol);
        // }
        // if let Some(node_open_symbol) = self.node_open_symbol {
        //     tree = tree.node_open_symbol(node_open_symbol);
        // }
        // if let Some(node_no_children_symbol) = self.node_no_children_symbol {
        //     tree = tree.node_no_children_symbol(node_no_children_symbol);
        // }
        // tree.render(area, buf, &mut state.widget);
    }
}

impl<Identifier: Default> Default for TreeSState<Identifier> {
    fn default() -> Self {
        Self {
            area: Default::default(),
            max_v_offset: 0,
            v_page_len: 0,
            widget: TreeState::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Identifier> ScrollingState for TreeSState<Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn vertical_max_offset(&self) -> usize {
        self.max_v_offset
    }

    fn vertical_offset(&self) -> usize {
        self.widget.get_offset()
    }

    fn vertical_page(&self) -> usize {
        self.v_page_len
    }

    fn vertical_scroll(&self) -> usize {
        self.v_page_len / 10
    }

    fn horizontal_max_offset(&self) -> usize {
        0
    }

    fn horizontal_offset(&self) -> usize {
        0
    }

    fn horizontal_page(&self) -> usize {
        0
    }

    fn horizontal_scroll(&self) -> usize {
        0
    }

    fn set_vertical_offset(&mut self, offset: usize) -> ScrollingOutcome {
        let old_offset = self.vertical_offset();

        if old_offset < offset {
            self.widget.scroll_down(offset - old_offset);
            ScrollingOutcome::Unknown
        } else {
            if self.widget.scroll_up(old_offset - offset) {
                ScrollingOutcome::Scrolled
            } else {
                ScrollingOutcome::Denied
            }
        }
    }

    fn set_horizontal_offset(&mut self, _offset: usize) -> ScrollingOutcome {
        ScrollingOutcome::Denied
    }
}

impl<Identifier> HandleEvent<crossterm::event::Event, DefaultKeys, Outcome>
    for TreeSState<Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        focus: bool,
        _keymap: DefaultKeys,
    ) -> Outcome {
        if focus {
            match event {
                ct_event!(keycode press Home) => {
                    self.widget.select_first();
                    Outcome::Changed
                }
                ct_event!(keycode press End) => {
                    self.widget.select_last();
                    Outcome::Changed
                }
                ct_event!(keycode press Left) => {
                    self.widget.key_left();
                    Outcome::Changed
                }
                ct_event!(keycode press Right) => {
                    self.widget.key_right();
                    Outcome::Changed
                }
                ct_event!(keycode press Up) => {
                    // TODO: doesn't work for now.
                    self.widget.key_up();
                    Outcome::Changed
                }
                ct_event!(keycode press Down) => {
                    // TODO: doesn't work for now.
                    self.widget.key_down();
                    Outcome::Changed
                }
                ct_event!(key press ' ') => {
                    self.widget.toggle_selected();
                    Outcome::Changed
                }
                _ => Outcome::NotUsed,
            }
        } else {
            Outcome::NotUsed
        }
    }
}

impl<Identifier> HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TreeSState<Identifier>
where
    Identifier: Debug + Clone + PartialEq + Eq + Hash,
{
    fn handle(
        &mut self,
        _event: &crossterm::event::Event,
        _focus: bool,
        _keymap: MouseOnly,
    ) -> Outcome {
        Outcome::NotUsed
    }
}
