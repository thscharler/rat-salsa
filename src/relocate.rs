//! Trait for a shift+clip operation that converts Rects
//! stored in the widgets state to screen coordinates.
//!
//! With this the render() fn can use the area parameter as
//! is and deal with screen coordinates later.
//!
//! Widgets that can do conversions from an internal coordinate system
//! to screen coordinates will need to store this information too.
//!

use ratatui::layout::{Position, Rect};

// --- sample code for a container implementer ---
// pub struct Rect32;
//
// pub struct RelocatedRender;
//
// impl RelocatedRender {
//     fn render<W, S>(widget: W, area: Rect32, state: &mut S)
//     where
//         W: StatefulWidget<State = S>,
//         S: RelocatableState,
//     {
//         // remap area
//         let area = Rect::default();
//         // inner buffer
//         let mut buf = Buffer::default();
//
//         widget.render(area, &mut buf, state);
//
//         // find shift and clip
//         let shift = (-1, -1);
//         let clip = Rect::default();
//
//         state.relocate(shift, clip);
//     }
// }

/// Widgets can be rendered to a temporary buffer using its own coordinate system.
///
/// To adjust the Rects derived during rendering/layout to the actual
/// screen coordinates a widget can implement this trait.
///
/// Container widgets that support this will call relocate() after rendering
/// the widgets.
pub trait RelocatableState {
    /// Relocate the areas in this widgets state.
    fn relocate(&mut self, shift: (i16, i16), clip: Rect);
}

/// Shift the area by offset and clip it.
pub fn relocate_area(area: Rect, shift: (i16, i16), clip: Rect) -> Rect {
    let x0 = area.left().saturating_add_signed(shift.0);
    let x1 = area.right().saturating_add_signed(shift.0);
    let y0 = area.top().saturating_add_signed(shift.1);
    let y1 = area.bottom().saturating_add_signed(shift.1);

    let tgt = Rect::new(x0, y0, x1 - x0, y1 - y0);

    tgt.intersection(clip)
}

/// Shift the position by offset and clip it.
pub fn relocate_pos(pos: Position, shift: (i16, i16), clip: Rect) -> Option<Position> {
    let x0 = pos.x.saturating_add_signed(shift.0);
    let y0 = pos.y.saturating_add_signed(shift.0);
    let pos0 = Position::new(x0, y0);

    if clip.contains(pos0) {
        Some(pos0)
    } else {
        None
    }
}
