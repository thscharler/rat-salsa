#![doc = include_str!("../readme.md")]

mod focus;

pub use crate::focus::{Focus, FocusBuilder, handle_focus};
use ratatui::layout::Rect;
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use std::rc::Rc;

/// Holds the flags for the focus.
///
/// Add this to the widget state and implement [HasFocus] to
/// manage your widgets focus state.
///
/// __Note__
///
/// This struct is intended to be cloned and uses a Rc internally
/// to share the state.
///
/// __Note__
///
/// Equality and Hash and the id() function use the memory address of the
/// FocusFlag behind the internal Rc<>.
///
/// __See__
/// [HasFocus], [on_gained!](crate::on_gained!) and
/// [on_lost!](crate::on_lost!).
///
#[derive(Clone, Default)]
pub struct FocusFlag(Rc<FocusFlagCore>);

/// Equality for FocusFlag means pointer equality of the underlying
/// Rc using Rc::ptr_eq.
impl PartialEq for FocusFlag {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for FocusFlag {}

impl Hash for FocusFlag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl Display for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "|{}|", self.0.name)
    }
}

impl HasFocus for FocusFlag {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    fn focus(&self) -> FocusFlag {
        self.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }

    fn area_z(&self) -> u16 {
        0
    }

    fn navigable(&self) -> Navigation {
        Navigation::Regular
    }
}

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
    /// Callback for set of gained.
    on_gained: RefCell<Option<Box<dyn Fn()>>>,
    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    lost: Cell<bool>,
    /// Callback for set of lost.
    on_lost: RefCell<Option<Box<dyn Fn()>>>,
}

/// Focus navigation for widgets.
///
/// The effects that hinder focus-change (`Reach*`, `Lock`) only work
/// when navigation changes via next()/prev()/focus_at().
///
/// Programmatic focus changes are always possible.
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

/// Trait for a widget that takes part of focus handling.
///
/// When used for a simple widget implement
/// - build()
/// - focus()
/// - area()
///
/// and optionally
///
/// - area_z() and navigable()
///
/// ```rust no_run
/// use ratatui::layout::Rect;
/// use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
///
/// struct MyWidgetState { pub focus: FocusFlag, pub area: Rect }
///
/// impl HasFocus for MyWidgetState {
///     fn build(&self, builder: &mut FocusBuilder) {
///         builder.leaf_widget(self);
///     }
///
///     fn focus(&self) -> FocusFlag {
///         self.focus.clone()
///     }
///
///     fn area(&self) -> Rect {
///         self.area
///     }
/// }
/// ```
///
///
/// When used for a container widget implement
/// - build()
/// ```rust no_run
/// use ratatui::layout::Rect;
/// use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
///
/// struct MyWidgetState { pub focus: FocusFlag, pub area: Rect }
/// # impl HasFocus for MyWidgetState {
/// #     fn build(&self, builder: &mut FocusBuilder) {
/// #         builder.leaf_widget(self);
/// #     }
/// #
/// #     fn focus(&self) -> FocusFlag {
/// #         self.focus.clone()
/// #     }
/// #
/// #     fn area(&self) -> Rect {
/// #         self.area
/// #     }
/// # }
/// struct SomeWidgetState { pub focus: FocusFlag, pub area: Rect, pub component_a: MyWidgetState, pub component_b: MyWidgetState }
///
/// impl HasFocus for SomeWidgetState {
///     fn build(&self, builder: &mut FocusBuilder) {
///         let tag = builder.start(self);
///         builder.widget(&self.component_a);
///         builder.widget(&self.component_b);
///         builder.end(tag);
///     }
///
///     fn focus(&self) -> FocusFlag {
///         self.focus.clone()
///     }
///
///     fn area(&self) -> Rect {
///         self.area
///     }
/// }
/// ```
/// Creates a container with an identity.
///
/// Or
/// ```rust no_run
/// use ratatui::layout::Rect;
/// use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
///
/// struct MyWidgetState { pub focus: FocusFlag, pub area: Rect }
/// # impl HasFocus for MyWidgetState {
/// #     fn build(&self, builder: &mut FocusBuilder) {
/// #         builder.leaf_widget(self);
/// #     }
/// #
/// #     fn focus(&self) -> FocusFlag {
/// #         self.focus.clone()
/// #     }
/// #
/// #     fn area(&self) -> Rect {
/// #         self.area
/// #     }
/// # }
/// struct SomeWidgetState { pub focus: FocusFlag, pub area: Rect, pub component_a: MyWidgetState, pub component_b: MyWidgetState }
///
/// impl HasFocus for SomeWidgetState {
///     fn build(&self, builder: &mut FocusBuilder) {
///         builder.widget(&self.component_a);
///         builder.widget(&self.component_b);
///     }
///
///     fn focus(&self) -> FocusFlag {
///         unimplemented!("not in use")
///     }
///
///     fn area(&self) -> Rect {
///         unimplemented!("not in use")
///     }
/// }
/// ```
/// for an anonymous container.
///
/// focus(), area() and area_z() are only used for the first case.
/// navigable() is ignored for containers, leave it at the default.
///
pub trait HasFocus {
    /// Build the focus-structure for the container.
    fn build(&self, builder: &mut FocusBuilder);

    /// Access to the flag for the rest.
    fn focus(&self) -> FocusFlag;

    /// Provide a unique id for the widget.
    fn id(&self) -> usize {
        self.focus().widget_id()
    }

    /// Area for mouse focus.
    ///
    /// This area shouldn't overlap with areas returned by other widgets.
    /// If it does, the widget should use `area_z()` for clarification.
    /// Otherwise, the areas are searched in order of addition.
    fn area(&self) -> Rect;

    /// Z value for the area.
    ///
    /// When testing for mouse interactions the z-value is taken into
    /// account too.
    fn area_z(&self) -> u16 {
        0
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

impl FocusFlag {
    /// Create a default flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return an identity value.
    ///
    /// This uses the memory address of the backing Rc so it will
    /// be unique during the runtime but will be different for each
    /// run.
    pub fn widget_id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }

    /// Create a named flag.
    ///
    /// The name is only used for debugging.
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

    /// Set the lost-flag and call any on_lost() function.
    ///
    /// on_lost() is only called if the change is from false to true.
    #[inline]
    pub fn set_lost(&self, lost: bool) {
        self.0.lost.set(lost);
    }

    #[inline]
    pub(crate) fn notify_on_lost(&self) {
        if let Some(on_lost) = self.0.on_lost.borrow().as_ref() {
            on_lost();
        }
    }

    #[inline]
    pub fn on_lost(&self, on_lost: impl Fn() + 'static) {
        *(self.0.on_lost.borrow_mut()) = Some(Box::new(on_lost));
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.0.gained.get()
    }

    /// Set the gained-flag and call any on_gained() function.
    ///
    /// on_gained() is only called if the change is from false to true.
    #[inline]
    pub fn set_gained(&self, gained: bool) {
        self.0.gained.set(gained);
    }

    #[inline]
    pub(crate) fn notify_on_gained(&self) {
        if let Some(on_gained) = self.0.on_gained.borrow().as_ref() {
            on_gained();
        }
    }

    #[inline]
    pub fn on_gained(&self, on_gained: impl Fn() + 'static) {
        *(self.0.on_gained.borrow_mut()) = Some(Box::new(on_gained));
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
            on_gained: RefCell::new(None),
            lost: Cell::new(false),
            on_lost: RefCell::new(None),
        }
    }
}

/// Does a match on the state struct of a widget. If `widget_state.lost_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocus],
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
        use $crate::HasFocus;
        $(if $field.lost_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If `widget_state.gained_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocus],
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
        use $crate::HasFocus;
        $(if $field.gained_focus() { _ = $validate })*
    }};
}

/// Does a match on several fields and can return a result.
/// Does a `widget_state.is_focused()` for each field and returns
/// the first that is true. There is an `else` branch too.
///
/// This requires that `widget_state` implements [HasFocus],
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
///     else => {
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
    ($($field:expr => $block:expr),* $(, else => $final:expr)?) => {{
        use $crate::HasFocus;
        if false {
            unreachable!();
        }
        $(else if $field.is_focused() { $block })*
        $(else { $final })?
    }};
}

/// Create the implementation of HasFocus for the
/// given list of struct members.
///
/// Create a container with no identity.
/// ```
/// # use rat_focus::{impl_has_focus, FocusFlag};
/// # struct MyState { field1: FocusFlag, field2: FocusFlag, field3: FocusFlag }
/// impl_has_focus!(field1, field2, field3 for MyState);
/// ```
///
/// Create a container with an identity.
/// ```
/// # use rat_focus::{impl_has_focus, FocusFlag};
/// # struct MyState { container: FocusFlag, field1: FocusFlag, field2: FocusFlag, field3: FocusFlag }
/// impl_has_focus!(container: field1, field2, field3 for MyState);
/// ```
///
/// Create a container with an identity and an area that will react to mouse clicks.
/// ```
/// # use ratatui::layout::Rect;
/// # use rat_focus::{impl_has_focus, FocusFlag};
/// # struct MyState { container: FocusFlag, area: Rect, field1: FocusFlag, field2: FocusFlag, field3: FocusFlag }
/// impl_has_focus!(container:area: field1, field2, field3 for MyState);
/// ```
#[macro_export]
macro_rules! impl_has_focus {
    ($cc:ident:$area:ident: $($n:ident),* for $ty:ty) => {
        impl $crate::HasFocus for $ty {
            fn build(&self, builder: &mut $crate::FocusBuilder) {
                let tag = builder.start(self);
                $(builder.widget(&self.$n);)*
                builder.end(tag);
            }

            fn focus(&self) -> $crate::FocusFlag {
                self.$cc.clone()
            }

            fn area(&self) -> ratatui::layout::Rect {
                self.$area
            }
        }
    };
    ($cc:ident: $($n:ident),* for $ty:ty) => {
        impl $crate::HasFocus for $ty {
            fn build(&self, builder: &mut $crate::FocusBuilder) {
                let tag = builder.start(self);
                $(builder.widget(&self.$n);)*
                builder.end(tag);
            }

            fn focus(&self) -> $crate::FocusFlag {
                self.$cc.clone()
            }

            fn area(&self) -> ratatui::layout::Rect {
                ratatui::layout::Rect::default()
            }
        }
    };
    ($($n:ident),* for $ty:ty) => {
        impl $crate::HasFocus for $ty {
            fn build(&self, builder: &mut $crate::FocusBuilder) {
                $(builder.widget(&self.$n);)*
            }

            fn focus(&self) -> $crate::FocusFlag {
                unimplemented!("not defined")
            }

            fn area(&self) -> ratatui::layout::Rect {
                unimplemented!("not defined")
            }
        }
    };
}
