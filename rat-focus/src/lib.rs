#![doc = include_str!("../readme.md")]

mod focus;

pub use crate::focus::{Focus, FocusBuilder, handle_focus};
use ratatui_core::layout::Rect;
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
        let name = self.0.name.borrow();
        if let Some(name) = &*name {
            write!(f, "|{}|", name)
        } else {
            write!(f, "")
        }
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
    name: RefCell<Option<Box<str>>>,
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
/// use ratatui_core::layout::Rect;
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
/// use ratatui_core::layout::Rect;
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
/// use ratatui_core::layout::Rect;
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
    /// Build the focus-structure for the container/widget.
    fn build(&self, builder: &mut FocusBuilder);

    /// Build the focus-structure for the container/widget.
    /// This is called when the default navigation will be
    /// overridden by the builder.
    ///
    /// It defaults to calling build and ignoring the navigable flag.
    ///
    /// You still have to implement build() for the baseline functionality.
    /// This is just an extra.
    #[allow(unused_variables)]
    fn build_nav(&self, navigable: Navigation, builder: &mut FocusBuilder) {
        self.build(builder);
    }

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
            .field("name", &self.0.name)
            .field("focus", &self.0.focus.get())
            .field("widget_id", &self.widget_id())
            .field("gained", &self.0.gained.get())
            .field("on_gained", &self.0.on_gained.borrow().is_some())
            .field("lost", &self.0.lost.get())
            .field("on_lost", &self.0.on_lost.borrow().is_some())
            .finish()
    }
}

impl FocusFlag {
    /// Create a default flag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a deep copy of the FocusFlag.
    ///
    /// Caution
    ///
    /// It will lose the on_gained() and on_lost() callbacks.
    /// Those can not be replicated/cloned as they will
    /// most probably hold some Rc's to somewhere.
    ///
    /// You will need to set them anew.
    pub fn new_instance(&self) -> Self {
        Self(Rc::new(self.0.fake_clone()))
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
    #[deprecated(
        since = "1.4.0",
        note = "to dangerous, use FocusFlag::new().with_name(..) or FocusFlag::fake_clone(..) for a clone."
    )]
    pub fn named(name: impl AsRef<str>) -> Self {
        Self(Rc::new(FocusFlagCore::default().named(name.as_ref())))
    }

    /// Set a name for a FocusFlag.
    pub fn with_name(self, name: &str) -> Self {
        self.set_name(name);
        self
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
    pub fn name(&self) -> Box<str> {
        self.0.name.borrow().clone().unwrap_or_default()
    }

    /// Set the field-name.
    #[inline]
    pub fn set_name(&self, name: &str) {
        *self.0.name.borrow_mut() = Some(Box::from(name))
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.0.lost.get()
    }

    /// Set the lost-flag.
    ///
    /// This doesn't call the on_lost callback.
    #[inline]
    pub fn set_lost(&self, lost: bool) {
        self.0.lost.set(lost);
    }

    /// Set an on_lost callback. The intention is that widget-creators
    /// can use this to get guaranteed notifications on focus-changes.
    ///
    /// This is not an api for widget *users.
    #[inline]
    pub fn on_lost(&self, on_lost: impl Fn() + 'static) {
        *(self.0.on_lost.borrow_mut()) = Some(Box::new(on_lost));
    }

    /// Notify an on_lost() tragedy.
    #[inline]
    pub fn call_on_lost(&self) {
        let borrow = self.0.on_lost.borrow();
        if let Some(f) = borrow.as_ref() {
            f();
        }
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.0.gained.get()
    }

    /// Set the gained-flag.
    ///
    /// This doesn't call the on_gained callback.
    #[inline]
    pub fn set_gained(&self, gained: bool) {
        self.0.gained.set(gained);
    }

    /// Set an on_gained callback. The intention is that widget-creators
    /// can use this to get guaranteed notifications on focus-changes.
    ///
    /// This is not an api for widget *users.
    #[inline]
    pub fn on_gained(&self, on_gained: impl Fn() + 'static) {
        *(self.0.on_gained.borrow_mut()) = Some(Box::new(on_gained));
    }

    /// Notify an on_gained() comedy.
    #[inline]
    pub fn call_on_gained(&self) {
        let borrow = self.0.on_gained.borrow();
        if let Some(f) = borrow.as_ref() {
            f();
        }
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
    #[inline(always)]
    pub(crate) fn named(self, name: &str) -> Self {
        *self.name.borrow_mut() = Some(Box::from(name));
        self
    }

    pub(crate) fn fake_clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            focus: Cell::new(self.focus.get()),
            gained: Cell::new(self.gained.get()),
            on_gained: RefCell::new(None),
            lost: Cell::new(self.lost.get()),
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
/// # use ratatui_core::layout::Rect;
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

            fn area(&self) -> ratatui_core::layout::Rect {
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

            fn area(&self) -> ratatui_core::layout::Rect {
                ratatui_core::layout::Rect::default()
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

            fn area(&self) -> ratatui_core::layout::Rect {
                unimplemented!("not defined")
            }
        }
    };
}
