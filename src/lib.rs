//! Current status: BETA
//!
#![doc = include_str!("../readme.md")]

mod focus;
mod zrect;

#[allow(unused_imports)]
use log::debug;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub use crate::focus::{handle_focus, Focus, FocusBuilder};
pub use crate::zrect::ZRect;

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::{
        crossterm, ct_event, flow, try_flow, util, ConsumedEvent, HandleEvent, MouseOnly, Outcome,
        Regular,
    };
}

/// Holds the flags for the focus.
///
/// Add this to the widget state.
///
/// This struct is intended to be cloned and uses a Rc internally
/// to share the state.
///
///
///
/// __Attention__
/// Equality for FocusFlag means pointer-equality of the underlying
/// Rc using Rc::ptr_eq.
///
/// __See__
/// [HasFocusFlag], [on_gained!](crate::on_gained!) and
/// [on_lost!](crate::on_lost!).
///
/// __See__
/// [FocusAdapter] to use with widgets that don't have
/// their own focus flag.
#[derive(Clone, Default)]
pub struct FocusFlag(Rc<FocusFlagCore>);

/// The same as FocusFlag, but distinct to mark the focus for
/// a container.
///
/// This serves the purpose of
///
/// * summarizing the focus for the container. If any of the
///     widgets of the container has the focus, the container
///     itself has the focus.
/// * identifying the container for some functions on Focus.
#[derive(Clone, Default)]
pub struct ContainerFlag(Rc<FocusFlagCore>);

/// Equality for FocusFlag means pointer equality of the underlying
/// Rc using Rc::ptr_eq.
impl PartialEq for FocusFlag {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for FocusFlag {}

impl Display for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "|{}|", self.0.name)
    }
}

impl HasFocusFlag for FocusFlag {
    fn focus(&self) -> FocusFlag {
        self.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

/// Equality for ContainerFlag means pointer equality of the underlying
/// Rc using Rc::ptr_eq.
impl PartialEq for ContainerFlag {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ContainerFlag {}

impl Display for ContainerFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "|{}|", self.0.name)
    }
}

impl HasFocus for ContainerFlag {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.start(self.clone(), Rect::default());
        builder.end(self.clone());
    }

    fn container(&self) -> Option<ContainerFlag> {
        Some(self.clone())
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

// not Clone, always Rc<>
#[derive(Default)]
struct FocusFlagCore {
    /// Field name for debugging purposes.
    name: Box<str>,
    /// Focus.
    focus: Cell<bool>,
    /// This widget just gained the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_gained!](crate::on_gained!)
    gained: Cell<bool>,
    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    lost: Cell<bool>,
}

/// Focus navigation for widgets.
///
/// The effects that hinder focus-change (`Reach*`, `Lock`) only work
/// when navigation changes via next()/prev()/focus_at().
/// Programmatic focus changes are always possible.
///
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Navigation {
    /// Widget is not reachable with normal keyboard or mouse navigation.
    None,
    /// Focus is locked to stay with this widget. No mouse or keyboard navigation
    /// can change that.
    Lock,
    /// Widget is not reachable with keyboard navigation, but can be focused with the mouse.
    Mouse,
    /// Widget cannot be reached with normal keyboard navigation, but can be left.
    /// (e.g. Tabs, Split-Divider)
    Leave,
    /// Widget can be reached with normal keyboard navigation, but not left.
    /// (e.g. TextArea)
    Reach,
    /// Widget can be reached with normal keyboard navigation, but only be left with
    /// backward navigation.
    /// (e.g. some widget with internal structure)
    ReachLeaveFront,
    /// Widget can be reached with normal keyboard navigation, but only be left with
    /// forward navigation.
    /// (e.g. some widget with internal structure)
    ReachLeaveBack,
    /// Widget can be reached and left with normal keyboard navigation.
    #[default]
    Regular,
}

/// Trait for a widget that has a focus flag.
pub trait HasFocusFlag {
    /// Access to the flag for the rest.
    fn focus(&self) -> FocusFlag;

    /// Area for mouse focus.
    ///
    /// This area shouldn't overlap with areas returned by other widgets.
    /// If it does, the widget should use `z_areas()` for clarification.
    /// Otherwise, the areas are searched in order of addition.
    fn area(&self) -> Rect;

    /// The widget might have several disjointed/overlapping areas.
    /// This is the case if it is showing a popup, but there might be other causes.
    ///
    /// If `z_areas()` returns an empty slice it defaults to `area()`+z-index 0.
    ///
    /// `z_areas()` are a higher resolution image of the widgets areas.
    /// If a widget returns anything but an empty slice here:
    /// - `area()` must be the union of all z_areas. Hit detection will fail
    ///   for anything outside `area()`
    /// - z_areas may overlap other areas. The area with the higher z-index
    ///   will win the hit-test. If there are still overlapping areas after
    ///   that, they will be used in the order of addition.
    fn z_areas(&self) -> &[ZRect] {
        &[]
    }

    /// Declares how the widget interacts with focus.
    ///
    /// Default is [Navigation::Regular].
    fn navigable(&self) -> Navigation {
        Navigation::Regular
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

/// Shortcut function for building the focus for a container
/// that implements [HasFocus]().
///
/// This creates a fresh Focus.
///
/// __See__
/// Use [rebuild_focus()] if you want to ensure that widgets
/// that are no longer in the widget structure have their
/// focus flag reset properly. If you don't have
/// some logic to conditionally add widgets to the focus,
/// this function is probably fine.
pub fn build_focus(container: &dyn HasFocus) -> Focus {
    let mut b = FocusBuilder::new(None);
    container.build(&mut b);
    b.build()
}

/// Shortcut function for building the focus for a container
/// that implements [HasFocus]()
///
/// This takes the old Focus and reuses most of its allocations.
/// It also ensures that any widgets no longer in the widget structure
/// have their focus-flags reset.
pub fn rebuild_focus(container: &dyn HasFocus, old: Option<Focus>) -> Focus {
    let mut b = FocusBuilder::new(old);
    container.build(&mut b);
    b.build()
}

/// Is this a container widget.
pub trait HasFocus {
    /// Build the focus-structure for the container.
    fn build(&self, builder: &mut FocusBuilder);

    /// Returns the container-flag, if any.
    fn container(&self) -> Option<ContainerFlag> {
        None
    }

    /// Area of the container.
    /// TODO: make this independent of container?
    fn area(&self) -> Rect {
        Rect::default()
    }

    /// Focused?
    fn is_focused(&self) -> bool {
        if let Some(flag) = self.container() {
            flag.get()
        } else {
            false
        }
    }

    /// Just lost focus.
    fn lost_focus(&self) -> bool {
        if let Some(flag) = self.container() {
            flag.lost()
        } else {
            false
        }
    }

    /// Just gained focus.
    fn gained_focus(&self) -> bool {
        if let Some(flag) = self.container() {
            flag.gained()
        } else {
            false
        }
    }
}

impl Debug for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusFlag")
            .field("name", &self.name())
            .field("focus", &self.get())
            .field("gained", &self.gained())
            .field("lost", &self.lost())
            .finish()
    }
}

impl Debug for ContainerFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerFlag")
            .field("name", &self.name())
            .field("focus", &self.get())
            .field("gained", &self.gained())
            .field("lost", &self.lost())
            .finish()
    }
}

impl FocusFlag {
    /// Create a default flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a named flag.
    pub fn named(name: &str) -> Self {
        Self(Rc::new(FocusFlagCore::named(name)))
    }

    /// Has the focus.
    #[inline]
    pub fn is_focused(&self) -> bool {
        self.0.focus.get()
    }

    /// Has the focus.
    #[inline]
    pub fn get(&self) -> bool {
        self.0.focus.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self, focus: bool) {
        self.0.focus.set(focus);
    }

    /// Get the field-name.
    #[inline]
    pub fn name(&self) -> &str {
        self.0.name.as_ref()
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.0.lost.get()
    }

    #[inline]
    pub fn set_lost(&self, lost: bool) {
        self.0.lost.set(lost);
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.0.gained.get()
    }

    #[inline]
    pub fn set_gained(&self, gained: bool) {
        self.0.gained.set(gained);
    }

    /// Reset all flags to false.
    #[inline]
    pub fn clear(&self) {
        self.0.focus.set(false);
        self.0.lost.set(false);
        self.0.gained.set(false);
    }
}

impl ContainerFlag {
    /// Create a default flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a named flag.
    pub fn named(name: &str) -> Self {
        Self(Rc::new(FocusFlagCore::named(name)))
    }

    /// Has the focus.
    #[inline]
    pub fn get(&self) -> bool {
        self.0.focus.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self, focus: bool) {
        self.0.focus.set(focus);
    }

    /// Get the field-name.
    #[inline]
    pub fn name(&self) -> &str {
        self.0.name.as_ref()
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.0.lost.get()
    }

    #[inline]
    pub fn set_lost(&self, lost: bool) {
        self.0.lost.set(lost);
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.0.gained.get()
    }

    #[inline]
    pub fn set_gained(&self, gained: bool) {
        self.0.gained.set(gained);
    }

    /// Reset all flags to false.
    #[inline]
    pub fn clear(&self) {
        self.0.focus.set(false);
        self.0.lost.set(false);
        self.0.gained.set(false);
    }
}

impl FocusFlagCore {
    pub(crate) fn named(name: &str) -> Self {
        Self {
            name: name.into(),
            focus: Cell::new(false),
            gained: Cell::new(false),
            lost: Cell::new(false),
        }
    }
}

/// Adapter for widgets that don't use this library.
/// Keep this adapter struct somewhere and use it to
/// manually control the widgets rendering/event handling.
#[derive(Debug, Default)]
pub struct FocusAdapter {
    pub focus: FocusFlag,
    pub area: Rect,
    pub navigation: Navigation,
}

impl FocusAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HasFocusFlag for FocusAdapter {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn navigable(&self) -> Navigation {
        self.navigation
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
