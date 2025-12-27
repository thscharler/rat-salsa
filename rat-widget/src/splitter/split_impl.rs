use crate::splitter::{Split, SplitState};
use ratatui_core::layout::{Direction, Position, Rect};
use ratatui_widgets::borders::BorderType;

pub(super) fn get_mark_0<'a>(widget: &Split<'a>) -> &'a str {
    if let Some(mark) = widget.mark_0_char {
        mark
    } else if widget.direction == Direction::Horizontal {
        "<"
    } else {
        "^"
    }
}

pub(super) fn get_mark_1<'a>(widget: &Split<'a>) -> &'a str {
    if let Some(mark) = widget.mark_1_char {
        mark
    } else if widget.direction == Direction::Horizontal {
        ">"
    } else {
        "v"
    }
}

pub(super) fn get_fill_char<'a>(widget: &Split<'a>) -> Option<&'a str> {
    use super::SplitType::*;
    use Direction::*;

    match (widget.direction, widget.split_type) {
        (Horizontal, FullEmpty) => Some(" "),
        (Vertical, FullEmpty) => Some(" "),
        (Horizontal, FullPlain) => Some("\u{2502}"),
        (Vertical, FullPlain) => Some("\u{2500}"),
        (Horizontal, FullDouble) => Some("\u{2551}"),
        (Vertical, FullDouble) => Some("\u{2550}"),
        (Horizontal, FullThick) => Some("\u{2503}"),
        (Vertical, FullThick) => Some("\u{2501}"),
        (Horizontal, FullQuadrantInside) => Some("\u{258C}"),
        (Vertical, FullQuadrantInside) => Some("\u{2580}"),
        (Horizontal, FullQuadrantOutside) => Some("\u{2590}"),
        (Vertical, FullQuadrantOutside) => Some("\u{2584}"),
        (_, Scroll) => None,
        (_, Widget) => None,
    }
}

#[allow(unreachable_patterns)]
pub(super) fn get_join_0<'a>(
    widget: &Split<'a>,
    split_area: Rect,
    state: &SplitState,
) -> Option<(Position, &'a str)> {
    use super::SplitType::*;
    use Direction::*;

    let s: Option<&str> = if let Some(join_0) = widget.join_0 {
        match (widget.direction, join_0, widget.split_type) {
            (
                Horizontal,
                BorderType::Plain | BorderType::Rounded,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{252C}"),
            (
                Vertical,
                BorderType::Plain | BorderType::Rounded,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{251C}"),
            (
                Horizontal,
                BorderType::Plain | BorderType::Rounded | BorderType::Thick,
                FullDouble,
            ) => Some("\u{2565}"),
            (Vertical, BorderType::Plain | BorderType::Rounded | BorderType::Thick, FullDouble) => {
                Some("\u{255E}")
            }
            (Horizontal, BorderType::Plain | BorderType::Rounded, FullThick) => Some("\u{2530}"),
            (Vertical, BorderType::Plain | BorderType::Rounded, FullThick) => Some("\u{251D}"),

            (
                Horizontal,
                BorderType::Double,
                FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                | Scroll,
            ) => Some("\u{2564}"),
            (
                Vertical,
                BorderType::Double,
                FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                | Scroll,
            ) => Some("\u{255F}"),
            (Horizontal, BorderType::Double, FullDouble) => Some("\u{2566}"),
            (Vertical, BorderType::Double, FullDouble) => Some("\u{2560}"),

            (
                Horizontal,
                BorderType::Thick,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{252F}"),
            (
                Vertical,
                BorderType::Thick,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{2520}"),
            (Horizontal, BorderType::Thick, FullThick) => Some("\u{2533}"),
            (Vertical, BorderType::Thick, FullThick) => Some("\u{2523}"),

            (Horizontal, BorderType::QuadrantOutside, FullEmpty) => Some("\u{2588}"),
            (Vertical, BorderType::QuadrantOutside, FullEmpty) => Some("\u{2588}"),

            (_, BorderType::QuadrantInside, _) => None,
            (_, BorderType::QuadrantOutside, _) => None,

            (_, _, Widget) => None,
            (_, _, _) => None,
        }
    } else {
        None
    };

    s.map(|s| {
        (
            match widget.direction {
                Horizontal => Position::new(split_area.x, state.area.y),
                Vertical => Position::new(state.area.x, split_area.y),
            },
            s,
        )
    })
}

#[allow(unreachable_patterns)]
pub(super) fn get_join_1<'a>(
    widget: &Split<'a>,
    split_area: Rect,
    state: &SplitState,
) -> Option<(Position, &'a str)> {
    use super::SplitType::*;
    use Direction::*;

    let s: Option<&str> = if let Some(join_1) = widget.join_1 {
        match (widget.direction, join_1, widget.split_type) {
            (
                Horizontal,
                BorderType::Plain | BorderType::Rounded,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{2534}"),
            (
                Vertical,
                BorderType::Plain | BorderType::Rounded,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{2524}"),
            (
                Horizontal,
                BorderType::Plain | BorderType::Rounded | BorderType::Thick,
                FullDouble,
            ) => Some("\u{2568}"),
            (Vertical, BorderType::Plain | BorderType::Rounded | BorderType::Thick, FullDouble) => {
                Some("\u{2561}")
            }
            (Horizontal, BorderType::Plain | BorderType::Rounded, FullThick) => Some("\u{2538}"),
            (Vertical, BorderType::Plain | BorderType::Rounded, FullThick) => Some("\u{2525}"),

            (
                Horizontal,
                BorderType::Double,
                FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                | Scroll,
            ) => Some("\u{2567}"),
            (
                Vertical,
                BorderType::Double,
                FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                | Scroll,
            ) => Some("\u{2562}"),
            (Horizontal, BorderType::Double, FullDouble) => Some("\u{2569}"),
            (Vertical, BorderType::Double, FullDouble) => Some("\u{2563}"),

            (
                Horizontal,
                BorderType::Thick,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{2537}"),
            (
                Vertical,
                BorderType::Thick,
                FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll,
            ) => Some("\u{2528}"),
            (Horizontal, BorderType::Thick, FullThick) => Some("\u{253B}"),
            (Vertical, BorderType::Thick, FullThick) => Some("\u{252B}"),

            (Horizontal, BorderType::QuadrantOutside, FullEmpty) => Some("\u{2588}"),
            (Vertical, BorderType::QuadrantOutside, FullEmpty) => Some("\u{2588}"),

            (_, BorderType::QuadrantInside, _) => None,
            (_, BorderType::QuadrantOutside, _) => None,

            (_, _, Widget) => None,
            (_, _, _) => None,
        }
    } else {
        None
    };

    s.map(|s| {
        (
            match widget.direction {
                Horizontal => Position::new(split_area.x, state.area.y + state.area.height - 1),
                Vertical => Position::new(state.area.x + state.area.width - 1, split_area.y),
            },
            s,
        )
    })
}
