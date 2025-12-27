#![doc = include_str!("../readme.md")]

use ratatui_core::layout::{Position, Rect};

/// Widgets can be rendered to a temporary Buffer that will
/// be dumped to the main render Buffer at a later point.
///
/// This temporary Buffer can have its own coordinate system
/// and output in the main render Buffer can happen anywhere.
///
/// Most rat-widgets store one or more areas during rendering
/// for later mouse interactions. All these areas need to
/// be adjusted whenever such a temporary Buffer is used.
///
/// This trait provides the entry point for such an adjustment.
pub trait RelocatableState {
    /// Relocate the areas in this widgets state.
    /// - shift: positions are moved by (x,y)
    /// - clip: areas must be clipped to the given Rect.
    fn relocate(&mut self, shift: (i16, i16), clip: Rect);

    /// Relocate only popup areas.
    /// As rendering the popups is a separate render,
    /// this has to be separate too.
    #[allow(unused_variables)]
    fn relocate_popup(&mut self, shift: (i16, i16), clip: Rect) {}

    /// Relocate all popup areas to a clip-rect (0,0+0x0).
    fn relocate_popup_hidden(&mut self) {
        self.relocate_popup((0, 0), Rect::default())
    }

    /// Relocate all areas to a clip-rect (0,0+0x0).
    fn relocate_hidden(&mut self) {
        self.relocate((0, 0), Rect::default())
    }
}

/// Create the implementation of RelocatableState for the
/// given list of struct members.
#[macro_export]
macro_rules! impl_relocatable_state {
    ($($n:ident),* for $ty:ty) => {
        impl $crate::RelocatableState for $ty {
            fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
                $(self.$n.relocate(shift, clip);)*
            }
        }
    };
}

impl RelocatableState for Rect {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        *self = relocate_area(*self, shift, clip);
    }
}

impl RelocatableState for [Rect] {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        for rect in self {
            rect.relocate(shift, clip);
        }
    }
}

impl RelocatableState for () {
    fn relocate(&mut self, _shift: (i16, i16), _clip: Rect) {}
}

/// Shift the area by offset and clip it.
pub fn relocate_areas(area: &mut [Rect], shift: (i16, i16), clip: Rect) {
    for a in area {
        *a = relocate_area(*a, shift, clip)
    }
}

/// Shift the area by offset and clip it.
pub fn relocate_area(area: Rect, shift: (i16, i16), clip: Rect) -> Rect {
    let relocated = relocate(area, shift);
    let clipped = clipped(relocated, clip);

    if !clipped.is_empty() {
        clipped
    } else {
        // default on empty
        Rect::default()
    }
}

/// Shift the position by offset and clip it.
/// Clipped positions are replaced with (0,0) for this function.
pub fn relocate_positions(pos: &mut [Position], shift: (i16, i16), clip: Rect) {
    for p in pos {
        *p = relocate_position(*p, shift, clip).unwrap_or_default()
    }
}

/// Shift the position by offset and clip it.
/// Returns None if the position is clipped.
pub fn relocate_position(pos: Position, shift: (i16, i16), clip: Rect) -> Option<Position> {
    let reloc = relocate(Rect::new(pos.x, pos.y, 0, 0), shift);
    let reloc_pos = Position::new(reloc.x, reloc.y);

    if clip.contains(reloc_pos) {
        Some(reloc_pos)
    } else {
        None
    }
}

/// Shift the position by offset and clip it.
/// Returns None if the position is clipped.
pub fn relocate_pos_tuple(pos: (u16, u16), shift: (i16, i16), clip: Rect) -> Option<(u16, u16)> {
    let reloc = relocate(Rect::new(pos.0, pos.1, 0, 0), shift);
    let reloc_pos = Position::new(reloc.x, reloc.y);

    if clip.contains(reloc_pos) {
        Some((reloc_pos.x, reloc_pos.y))
    } else {
        None
    }
}

/// Clipping might introduce another offset by cutting away
/// part of an area that a widgets internal offset refers to.
///
/// This calculates this dark, extra offset.
pub fn relocate_dark_offset(area: Rect, shift: (i16, i16), clip: Rect) -> (u16, u16) {
    let relocated = relocate(area, shift);
    let clipped = clipped(relocated, clip);

    (clipped.x - relocated.x, clipped.y - relocated.y)
}

#[inline]
fn relocate(area: Rect, shift: (i16, i16)) -> Rect {
    let x0 = area.left().saturating_add_signed(shift.0);
    let x1 = area.right().saturating_add_signed(shift.0);
    let y0 = area.top().saturating_add_signed(shift.1);
    let y1 = area.bottom().saturating_add_signed(shift.1);

    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

#[inline]
fn clipped(area: Rect, clip: Rect) -> Rect {
    area.intersection(clip)
}
