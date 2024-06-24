#![doc = include_str!("../readme.md")]

mod focus;
mod zrect;

#[allow(unused_imports)]
use log::debug;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};

pub use crate::focus::{handle_focus, handle_mouse_focus, Focus};
pub use crate::zrect::ZRect;

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::{
        crossterm, ct_event, flow, flow_ok, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly,
        Outcome,
    };
}

/// Contains flags for the focus.
/// This struct is embedded in the widget state.
///
/// See [HasFocusFlag], [on_gained!](crate::on_gained!) and
/// [on_lost!](crate::on_lost!).
///
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FocusFlag {
    /// Field name for debugging purposes.
    pub name: Cell<&'static str>,
    /// Focus.
    pub focus: Cell<bool>,
    /// This widget just gained the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_gained!](crate::on_gained!)
    pub gained: Cell<bool>,
    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    pub lost: Cell<bool>,
}

/// Trait for a widget that has a focus flag.
pub trait HasFocusFlag {
    /// Access to the flag for the rest.
    fn focus(&self) -> &FocusFlag;

    /// Access the area for mouse focus.
    fn area(&self) -> Rect;

    /// The widget might have several disjointed areas.
    /// This is the case if it is showing a popup, but there
    /// might be other causes.
    ///
    /// This is seen as a higher resolution image of the
    /// area given with area(). That means the result of
    /// area() is the union of all areas given here.
    fn z_areas(&self) -> &[ZRect] {
        &[]
    }

    /// The widget is focusable, but doesn't want to partake
    /// in keyboard navigation.
    fn navigable(&self) -> bool {
        true
    }

    /// Focused?
    fn is_focused(&self) -> bool {
        self.focus().get()
    }

    /// Just lost focus.
    fn lost_focus(&self) -> bool {
        self.focus().lost()
    }

    /// Just gained focus.
    fn gained_focus(&self) -> bool {
        self.focus().gained()
    }
}

/// Is this a container widget of sorts.
pub trait HasFocus {
    /// Returns a Focus struct.
    fn focus(&self) -> Focus<'_>;
}

impl Debug for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusFlag")
            .field("name", &self.name.get())
            .field("focus", &self.focus.get())
            .field("gained", &self.gained.get())
            .field("lost", &self.lost.get())
            .finish()
    }
}

impl FocusFlag {
    /// Has the focus.
    #[inline]
    pub fn get(&self) -> bool {
        self.focus.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self) {
        self.focus.set(true);
    }

    /// Set the field-name.
    #[inline]
    pub fn set_name(&self, name: &'static str) {
        self.name.set(name);
    }

    /// Get the field-name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name.get()
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.lost.get()
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.gained.get()
    }

    /// Reset all flags to false.
    #[inline]
    pub fn clear(&self) {
        self.focus.set(false);
        self.lost.set(false);
        self.gained.set(false);
    }
}

/// Does a match on the state struct of a widget. If `widget_state.lost_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocusFlag],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_lost;
///
/// on_lost!(
///     state.field1 => {
///         // do checks
///     },
///     state.field2 => {
///         // do checks
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_lost {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocusFlag;
        $(if $field.lost_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If `widget_state.gained_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocusFlag],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_gained;
///
/// on_gained!(
///     state.field1 => {
///         // do prep
///     },
///     state.field2 => {
///         // do prep
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_gained {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocusFlag;
        $(if $field.gained_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If
/// `widget_state.is_focused()` is true the block is executed.
/// There is a `_` branch too, that is evaluated if none of the
/// given widget-states has the focus.
///
/// This requires that `widget_state` implements [HasFocusFlag],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::match_focus;
///
/// let res = match_focus!(
///     state.field1 => {
///         // do this
///         true
///     },
///     state.field2 => {
///         // do that
///         true
///     },
///     _ => {
///         false
///     }
/// );
///
/// if res {
///     // react
/// }
/// ```
///
#[macro_export]
macro_rules! match_focus {
    ($($field:expr => $block:expr),* $(, _ => $final:expr)?) => {{
        use $crate::HasFocusFlag;
        if false {
            unreachable!();
        }
        $(else if $field.is_focused() { $block })*
        $(else { $final })?
    }};
}
