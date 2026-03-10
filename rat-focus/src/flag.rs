use crate::{FocusBuilder, HasFocus, Navigation, ratatui};
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

    fn area(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::default()
    }

    fn area_z(&self) -> u16 {
        0
    }

    fn navigable(&self) -> Navigation {
        Navigation::Regular
    }
}

struct FocusFlagCore {
    /// Field name for debugging purposes.
    name: RefCell<Option<Box<str>>>,
    /// Does this widget have the focus.
    /// Or, if the flag is used for a container, does any of
    /// widget inside the container have the focus.
    ///
    /// This flag is set by [Focus::handle].
    focus: Cell<bool>,
    /// This widget just gained the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_gained!](crate::on_gained!)
    gained: Cell<bool>,
    /// Callback for set of gained.
    ///
    /// A widget can set this callback and will be notified
    /// by Focus whenever it gains the focus.
    ///
    /// It's a bit crude, as you have set up any widget-state
    /// you want to change as shared state with the callback-closure.
    /// But it's still preferable to relying on the fact that
    /// the `handle` event for a widget will be called while
    /// the gained flag is still set.
    on_gained: RefCell<Option<Box<dyn Fn()>>>,
    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    lost: Cell<bool>,
    /// Callback for set of lost.
    ///
    /// A widget can set this callback and will be notified
    /// by Focus whenever it looses the focus.
    ///
    /// It's a bit crude, as you have set up any widget-state
    /// you want to change as shared state with the callback-closure.
    /// But it's still preferable to relying on the fact that
    /// the `handle` event for a widget will be called while
    /// the lost flag is still set.
    on_lost: RefCell<Option<Box<dyn Fn()>>>,
    /// This flag is set by [Focus::handle], if a mouse-event
    /// matches one of the areas associated with a widget.
    ///
    /// > It searches all containers for an area-match. All
    /// matching areas will have the flag set.
    /// If an area with a higher z is found, all previously
    /// found areas are discarded.
    ///
    /// > The z value for the last container is taken as a baseline.
    /// Only widgets with a z greater or equal are considered.
    /// If multiple widget areas are matching, the last one
    /// will get the flag set.
    ///
    /// This rules enable popup-windows with complex ui's.
    /// The popup-container starts with a z=1 and all widgets
    /// within also get the same z. With the given rules, all
    /// widgets underneath the popup are ignored.
    ///
    /// * This flag starts with a default `true`. This allows
    ///   widgets to work, even if Focus is not used.
    /// * Mouse drag events are not bound to any area.
    ///   Instead, they set the mouse-focus to true for all
    ///   widgets and containers.
    mouse_focus: Cell<bool>,
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
    pub fn call_on_lost(&self) -> bool {
        let borrow = self.0.on_lost.borrow();
        if let Some(f) = borrow.as_ref() {
            f();
            true
        } else {
            false
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
    pub fn call_on_gained(&self) -> bool {
        let borrow = self.0.on_gained.borrow();
        if let Some(f) = borrow.as_ref() {
            f();
            true
        } else {
            false
        }
    }

    /// Set the mouse-focus for this widget.
    #[inline]
    pub fn set_mouse_focus(&self, mf: bool) {
        self.0.mouse_focus.set(mf);
    }

    /// Is the mouse-focus set for this widget.
    ///
    /// This function will return true, if [Focus::handle] is never called.
    ///
    /// See [HasFocus::has_mouse_focus()]
    #[inline]
    pub fn mouse_focus(&self) -> bool {
        self.0.mouse_focus.get()
    }

    /// Reset all flags to false.
    #[inline]
    pub fn clear(&self) {
        self.0.focus.set(false);
        self.0.lost.set(false);
        self.0.gained.set(false);
        self.0.mouse_focus.set(true);
    }
}

impl Default for FocusFlagCore {
    fn default() -> Self {
        Self {
            name: RefCell::new(None),
            focus: Cell::new(false),
            gained: Cell::new(false),
            on_gained: RefCell::new(None),
            lost: Cell::new(false),
            on_lost: RefCell::new(None),
            mouse_focus: Cell::new(true),
        }
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
            mouse_focus: Cell::new(self.mouse_focus.get()),
        }
    }
}
