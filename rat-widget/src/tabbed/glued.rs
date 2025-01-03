//! Simple glued on the side tabs.

use crate::tabbed::{TabPlacement, TabWidget, Tabbed, TabbedState};
use crate::util::revert_style;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Margin, Rect};
use ratatui::widgets::Widget;

/// Renders simple tabs at the given placement and renders
/// the block inside the tabs.
#[derive(Debug, Default)]
pub(super) struct GluedTabs;

impl TabWidget for GluedTabs {
    fn layout(&self, area: Rect, tabbed: &Tabbed<'_>, state: &mut TabbedState) {
        state.area = area;

        let margin_offset = 1 + if tabbed.block.is_some() { 1 } else { 0 };
        let close_width = if tabbed.closeable { 2 } else { 0 };

        let max_width = tabbed
            .tabs
            .iter()
            .map(|v| v.width())
            .max()
            .unwrap_or_default() as u16;

        match tabbed.placement {
            TabPlacement::Top => {
                state.block_area = Rect::new(area.x, area.y + 1, area.width, area.height - 1);
                state.tab_title_area = Rect::new(area.x, area.y, area.width, 1);
                if let Some(block) = &tabbed.block {
                    state.widget_area = block.inner(state.block_area);
                } else {
                    state.widget_area = state.block_area;
                }
            }
            TabPlacement::Bottom => {
                state.block_area =
                    Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
                state.tab_title_area = Rect::new(
                    area.x,
                    area.y + area.height.saturating_sub(1),
                    area.width,
                    1,
                );
                if let Some(block) = &tabbed.block {
                    state.widget_area = block.inner(state.block_area);
                } else {
                    state.widget_area = state.block_area;
                }
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
                if let Some(block) = &tabbed.block {
                    state.widget_area = block.inner(state.block_area);
                } else {
                    state.widget_area = state.block_area;
                }
            }
            TabPlacement::Right => {
                state.block_area = Rect::new(
                    area.x,
                    area.y,
                    area.width - (max_width + 2 + close_width),
                    area.height,
                );
                state.tab_title_area = Rect::new(
                    area.x + area.width - (max_width + 2 + close_width),
                    area.y,
                    max_width + 2 + close_width,
                    area.height,
                );
                if let Some(block) = &tabbed.block {
                    state.widget_area = block.inner(state.block_area);
                } else {
                    state.widget_area = state.block_area;
                }
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
                        .horizontal_margin(margin_offset)
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
                        .vertical_margin(margin_offset)
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
        let focus_style = if let Some(focus_style) = tabbed.focus_style {
            focus_style
        } else {
            revert_style(tabbed.style)
        };
        let select_style = if let Some(select_style) = tabbed.select_style {
            if state.focus.get() {
                focus_style
            } else {
                select_style
            }
        } else {
            if state.focus.get() {
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

        buf.set_style(state.tab_title_area, tabbed.style);
        tabbed.block.clone().render(state.block_area, buf);

        for (idx, tab_area) in state.tab_title_areas.iter().copied().enumerate() {
            if Some(idx) == state.selected() {
                buf.set_style(tab_area, select_style);
            } else {
                buf.set_style(tab_area, tab_style);
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
        }
        if tabbed.closeable {
            for i in 0..state.tab_title_close_areas.len() {
                if state.mouse.hover.get() == Some(i) {
                    buf.set_style(state.tab_title_close_areas[i], revert_style(tab_style));
                }
                if let Some(cell) = buf.cell_mut(state.tab_title_close_areas[i].as_position()) {
                    cell.set_symbol(" \u{2A2F} ");
                }
            }
        }
    }
}
