//! Tabs embedded in the Block.
//!
//! If no block has been set, this will draw a block at the side
//! of the tabs.

use crate::tabbed::{TabPlacement, TabWidget, Tabbed, TabbedState};
use crate::util;
use crate::util::{block_left, block_right, fill_buf_area, revert_style};
use rat_focus::HasFocus;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::symbols::line;
use ratatui::widgets::{Block, BorderType, Borders, Widget};

/// Embedded tabs in the Block.
///
/// If no block has been set, this will draw a block at the side
/// of the tabs.
///
/// On the left/right side this will just draw a link to the tab-text.
/// On the top/bottom side the tabs will be embedded in the border.
#[derive(Debug)]
pub(super) struct AttachedTabs;

impl AttachedTabs {
    fn get_link(&self, placement: TabPlacement, block: &Block<'_>) -> Option<&str> {
        match placement {
            TabPlacement::Top => unreachable!(),
            TabPlacement::Bottom => unreachable!(),
            TabPlacement::Left => match block_left(block).as_str() {
                line::VERTICAL => Some(line::VERTICAL_LEFT),
                line::DOUBLE_VERTICAL => Some(util::DOUBLE_VERTICAL_SINGLE_LEFT),
                line::THICK_VERTICAL => Some(util::THICK_VERTICAL_SINGLE_LEFT),

                _ => None,
            },
            TabPlacement::Right => match block_right(block).as_str() {
                line::VERTICAL => Some(line::VERTICAL_RIGHT),
                line::DOUBLE_VERTICAL => Some(util::DOUBLE_VERTICAL_SINGLE_RIGHT),
                line::THICK_VERTICAL => Some(util::THICK_VERTICAL_SINGLE_RIGHT),
                _ => None,
            },
        }
    }
}

impl TabWidget for AttachedTabs {
    fn layout(&self, area: Rect, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
        state.area = area;

        let mut block;
        let (block, block_offset) = if let Some(block) = &tabbed.block {
            (block, 1)
        } else {
            block = Block::new();
            block = match tabbed.placement {
                TabPlacement::Top => block.borders(Borders::TOP),
                TabPlacement::Left => block.borders(Borders::LEFT),
                TabPlacement::Right => block.borders(Borders::RIGHT),
                TabPlacement::Bottom => block.borders(Borders::BOTTOM),
            };
            (&block, 0)
        };

        let close_width = if tabbed.closeable { 2 } else { 0 };

        let max_width = tabbed
            .tabs
            .iter()
            .map(|v| v.width())
            .max()
            .unwrap_or_default() as u16;

        match tabbed.placement {
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

        match tabbed.placement {
            TabPlacement::Top | TabPlacement::Bottom => {
                let mut constraints = Vec::new();
                for tab in tabbed.tabs.iter() {
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
                for _tab in tabbed.tabs.iter() {
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

        match tabbed.placement {
            TabPlacement::Top | TabPlacement::Bottom => {
                state.tab_title_close_areas = state
                    .tab_title_areas
                    .iter()
                    .map(|v| {
                        Rect::new(
                            (v.x + v.width)
                                .saturating_sub(close_width)
                                .saturating_sub(1),
                            v.y,
                            if tabbed.closeable { 3 } else { 0 },
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
                            v.x, //
                            v.y,
                            if tabbed.closeable { 3 } else { 0 },
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
                            (v.x + v.width)
                                .saturating_sub(close_width)
                                .saturating_sub(1),
                            v.y,
                            if tabbed.closeable { 3 } else { 0 },
                            1,
                        )
                    })
                    .collect::<Vec<_>>();
            }
        }
    }

    fn render(&self, buf: &mut Buffer, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
        let mut block;
        let block = if let Some(block) = &tabbed.block {
            block
        } else {
            block = Block::new()
                .border_type(BorderType::Plain)
                .style(tabbed.style);
            block = match tabbed.placement {
                TabPlacement::Top => block.borders(Borders::TOP),
                TabPlacement::Bottom => block.borders(Borders::BOTTOM),
                TabPlacement::Left => block.borders(Borders::LEFT),
                TabPlacement::Right => block.borders(Borders::RIGHT),
            };
            &block
        };

        let focus_style = if let Some(focus_style) = tabbed.focus_style {
            focus_style
        } else {
            revert_style(tabbed.style)
        };
        let select_style = if let Some(select_style) = tabbed.select_style {
            if state.is_focused() {
                focus_style
            } else {
                select_style
            }
        } else {
            if state.is_focused() {
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
        let hover_style = if let Some(hover_style) = tabbed.hover_style {
            hover_style
        } else {
            tabbed.style
        };

        // area for the left/right tabs.
        if matches!(tabbed.placement, TabPlacement::Left | TabPlacement::Right) {
            buf.set_style(state.tab_title_area, tabbed.style);
        }
        block.clone().render(state.block_area, buf);

        for (idx, tab_area) in state.tab_title_areas.iter().copied().enumerate() {
            if Some(idx) == state.selected() {
                fill_buf_area(buf, tab_area, " ", tab_style.patch(select_style));
            } else {
                fill_buf_area(buf, tab_area, " ", tab_style);
            }

            let txt_area = match tabbed.placement {
                TabPlacement::Top | TabPlacement::Right | TabPlacement::Bottom => {
                    tab_area.inner(Margin::new(1, 0))
                }
                TabPlacement::Left => {
                    if tabbed.closeable {
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
            tabbed.tabs[idx].clone().render(txt_area, buf);

            // join left/right
            match tabbed.placement {
                TabPlacement::Top => {}
                TabPlacement::Bottom => {}
                TabPlacement::Left => {
                    if let Some(cell) = buf.cell_mut((tab_area.x + tab_area.width, tab_area.y)) {
                        if let Some(sym) = self.get_link(tabbed.placement, block) {
                            cell.set_symbol(sym);
                        }
                    }
                }
                TabPlacement::Right => {
                    if let Some(cell) = buf.cell_mut((tab_area.x - 1, tab_area.y)) {
                        if let Some(sym) = self.get_link(tabbed.placement, block) {
                            cell.set_symbol(sym);
                        }
                    }
                }
            }
        }

        if tabbed.closeable {
            for i in 0..state.tab_title_close_areas.len() {
                if state.mouse.hover.get() == Some(i) {
                    buf.set_style(state.tab_title_close_areas[i], hover_style);
                }
                if let Some(cell) = buf.cell_mut(state.tab_title_close_areas[i].as_position()) {
                    cell.set_symbol(" \u{2A2F} ");
                }
            }
        }
    }
}
