use crate::splitter::{ResizeConstraint, SPLIT_WIDTH, Split, SplitState, SplitType};
use log::debug;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position, Rect};
use ratatui::prelude::BlockExt;
use std::cmp::min;

/// Calculates the first layout according to the constraints.
/// When a resize is detected, the current widths are used as constraints.
pub(super) fn layout_split<'a>(widget: &Split<'a>, area: Rect, state: &mut SplitState) {
    state.area = area;
    state.inner = widget.block.inner_if_some(area);
    state.relocate_split = true;

    // use only the inner from here on
    let inner = state.inner;

    let layout_change = state.area_length.len() != widget.constraints.len();
    let meta_change = state.direction != widget.direction
        || state.split_type != widget.split_type
        || state.mark_offset != widget.mark_offset;

    let old_len = |v: &Rect| {
        // must use the old direction to get a correct value.
        if state.direction == Direction::Horizontal {
            v.width
        } else {
            v.height
        }
    };
    let new_len = |v: &Rect| {
        // must use the old direction to get a correct value.
        if widget.direction == Direction::Horizontal {
            v.width
        } else {
            v.height
        }
    };

    let new_split_areas = if layout_change {
        // initial
        let new_areas = Layout::new(widget.direction, widget.constraints.clone())
            .flex(Flex::Legacy)
            .split(inner);
        Some(new_areas)
    } else {
        let old_length: u16 = state.area_length.iter().sum();
        if meta_change || old_len(&inner) != old_length {
            // use cached constraints. avoids cumulative errors.
            if state.area_constraint.is_empty() {
                for i in 0..state.area_length.len() {
                    match widget.resize_constraints[i] {
                        ResizeConstraint::Fixed => {
                            state
                                .area_constraint
                                .push(Constraint::Length(state.area_length[i]));
                        }
                        ResizeConstraint::ScaleProportional => {
                            state
                                .area_constraint
                                .push(Constraint::Fill(state.area_length[i]));
                        }
                        ResizeConstraint::ScaleEqual => {
                            state.area_constraint.push(Constraint::Fill(1));
                        }
                    }
                }
            }
            let new_areas =
                Layout::new(widget.direction, state.area_constraint.clone()).split(inner);
            Some(new_areas)
        } else {
            None
        }
    };

    if let Some(new_split_areas) = new_split_areas {
        state.area_length.clear();
        for v in new_split_areas.iter() {
            state.area_length.push(new_len(v));
        }
        while state.hidden_length.len() < state.area_length.len() {
            state.hidden_length.push(0);
        }
        while state.hidden_length.len() > state.area_length.len() {
            state.hidden_length.pop();
        }
    }

    state.direction = widget.direction;
    state.split_type = widget.split_type;
    state.resize = widget.resize;
    state.mark_offset = widget.mark_offset;

    layout_from_widths(widget, state);
}

fn layout_from_widths<'a>(widget: &Split<'a>, state: &mut SplitState) {
    // Areas changed, create areas and splits.
    state.widget_areas.clear();
    state.splitline_areas.clear();
    state.splitline_mark_position.clear();

    let inner = state.inner;

    let mut total = 0;
    for length in state
        .area_length
        .iter()
        .take(state.area_length.len().saturating_sub(1))
        .copied()
    {
        let mut area = if widget.direction == Direction::Horizontal {
            Rect::new(inner.x + total, inner.y, length, inner.height)
        } else {
            Rect::new(inner.x, inner.y + total, inner.width, length)
        };
        let mut split = if widget.direction == Direction::Horizontal {
            Rect::new(
                inner.x + total + length.saturating_sub(SPLIT_WIDTH),
                inner.y,
                min(1, length),
                inner.height,
            )
        } else {
            Rect::new(
                inner.x,
                inner.y + total + length.saturating_sub(SPLIT_WIDTH),
                inner.width,
                min(1, length),
            )
        };
        let mut mark = if widget.direction == Direction::Horizontal {
            Position::new(
                inner.x + total + length.saturating_sub(SPLIT_WIDTH),
                inner.y + widget.mark_offset,
            )
        } else {
            Position::new(
                inner.x + widget.mark_offset,
                inner.y + total + length.saturating_sub(SPLIT_WIDTH),
            )
        };

        adjust_for_split_type(
            widget.direction,
            widget.split_type,
            &mut area,
            &mut split,
            &mut mark,
        );

        state.widget_areas.push(area);
        state.splitline_areas.push(split);
        state.splitline_mark_position.push(mark);

        total += length;
    }
    if let Some(length) = state.area_length.last().copied() {
        let area = if widget.direction == Direction::Horizontal {
            Rect::new(inner.x + total, inner.y, length, inner.height)
        } else {
            Rect::new(inner.x, inner.y + total, inner.width, length)
        };

        state.widget_areas.push(area);
    }

    // Set 2nd dimension too, if necessary.
    if let Some(test) = state.widget_areas.first() {
        if widget.direction == Direction::Horizontal {
            if test.height != state.inner.height {
                for r in &mut state.widget_areas {
                    r.height = state.inner.height;
                }
                for r in &mut state.splitline_areas {
                    r.height = state.inner.height;
                }
            }
        } else {
            if test.width != state.inner.width {
                for r in &mut state.widget_areas {
                    r.width = state.inner.width;
                }
                for r in &mut state.splitline_areas {
                    r.width = state.inner.width;
                }
            }
        }
    }
}

/// Adjust area and split according to the split_type.
fn adjust_for_split_type(
    direction: Direction,
    split_type: SplitType,
    area: &mut Rect,
    split: &mut Rect,
    mark: &mut Position,
) {
    use Direction::*;
    use SplitType::*;

    match (direction, split_type) {
        (
            Horizontal,
            FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
            | FullQuadrantOutside,
        ) => {
            area.width = area.width.saturating_sub(SPLIT_WIDTH);
        }
        (
            Vertical,
            FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
            | FullQuadrantOutside,
        ) => {
            area.height = area.height.saturating_sub(SPLIT_WIDTH);
        }

        (Horizontal, Scroll) => {
            split.y = mark.y;
            split.height = 2;
        }
        (Vertical, Scroll) => {
            split.x = mark.x;
            split.width = 2;
        }

        (Horizontal, Widget) => {}
        (Vertical, Widget) => {}
    }
}
