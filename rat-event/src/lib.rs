#![doc = include_str!("../readme.md")]

use std::cmp::max;

pub mod crossterm;
pub mod util;

/// All the regular and expected event-handling a widget can do.
///
/// All the normal key-handling, maybe dependent on an internal
/// focus-state, all the mouse-handling.
#[derive(Debug, Default, Clone, Copy)]
pub struct Regular;

/// Handle mouse-events only. Useful whenever you want to write new key-bindings,
/// but keep the mouse-events.
#[derive(Debug, Default, Clone, Copy)]
pub struct MouseOnly;

/// Popup/Overlays are a bit difficult to handle, as there is no z-order/area tree,
/// which would direct mouse interactions. We can simulate a z-order in the
/// event-handler by trying the things with a higher z-order first.
///
/// If a widget should be seen as pure overlay, it would define only a Popup
/// event-handler. In the event-handling functions you would call all Popup
/// event-handlers before the regular ones.
///
/// Example:
/// * Context menu. If the context-menu is active, it can consume all mouse-events
///   that fall into its range, and the widgets behind it only get the rest.
/// * Menubar. Would define _two_ event-handlers, a regular one for all events
///   on the main menu bar, and a popup event-handler for the menus. The event-handling
///   function calls the popup handler first and the regular one at some time later.
#[derive(Debug, Default, Clone, Copy)]
pub struct Popup;

/// Event-handling for a dialog like widget.
///
/// Similar to [Popup] but with the extra that it consumes _all_ events when active.
/// No regular widget gets any event, and we have modal behaviour.
#[derive(Debug, Default, Clone, Copy)]
pub struct Dialog;

/// Event-handler for double-click on a widget.
///
/// Events for this handler must be processed *before* calling
/// any other event-handling routines for the same widget.
/// Otherwise, the regular event-handling might interfere with
/// recognition of double-clicks by consuming the first click.
///
/// This event-handler doesn't consume the first click, just
/// the second one.
#[derive(Debug, Default, Clone, Copy)]
pub struct DoubleClick;

///
/// A very broad trait for an event handler.
///
/// Ratatui widgets have two separate structs, one that implements
/// Widget/StatefulWidget and the associated State. As the StatefulWidget
/// has a lifetime and is not meant to be kept, HandleEvent should be
/// implemented for the state struct. It can then modify some state and
/// the tui can be rendered anew with that changed state.
///
/// HandleEvent is not limited to State structs of course, any Type
/// that wants to interact with events can implement it.
///
/// * Event - The actual event type.
/// * Qualifier - The qualifier allows creating more than one event-handler
///         for a widget.
///
///   This can be used as a variant of type-state, where the type given
///   selects the widget's behaviour, or to give some external context
///   to the widget, or to write your own key-bindings for a widget.
///
/// * R - Result of event-handling. This can give information to the
///   application what changed due to handling the event. This can
///   be very specific for each widget, but there is one general [Outcome]
///   that describes a minimal set of results.
///
///   There should be one value that indicates 'I don't know this event'.
///   This is expressed with the ConsumedEvent trait.
///
pub trait HandleEvent<Event, Qualifier, Return>
where
    Return: ConsumedEvent,
{
    /// Handle an event.
    ///
    /// * self - The widget state.
    /// * event - Event type.
    /// * qualifier - Event handling qualifier.
    ///   This library defines some standard values [Regular], [MouseOnly].
    ///   Further ideas:
    ///     * ReadOnly
    ///     * Special behaviour like DoubleClick, HotKey.
    /// * Returns some result, see [Outcome]
    fn handle(&mut self, event: &Event, qualifier: Qualifier) -> Return;
}

/// Catch all event-handler for the null state `()`.
impl<E, Q> HandleEvent<E, Q, Outcome> for () {
    fn handle(&mut self, _event: &E, _qualifier: Q) -> Outcome {
        Outcome::Continue
    }
}

/// When calling multiple event-handlers, the minimum information required
/// from the result is consumed the event/didn't consume the event.
///
/// See also [flow] and [try_flow] macros.
pub trait ConsumedEvent {
    /// Is this the 'consumed' result.
    fn is_consumed(&self) -> bool;

    /// Or-Else chaining with `is_consumed()` as the split.
    #[inline(always)]
    fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
        Self: Sized,
    {
        if self.is_consumed() {
            self
        } else {
            f()
        }
    }

    /// Or-Else chaining with `is_consumed()` as the split.
    #[inline(always)]
    fn or_else_try<F, E>(self, f: F) -> Result<Self, E>
    where
        Self: Sized,
        F: FnOnce() -> Result<Self, E>,
    {
        if self.is_consumed() {
            Ok(self)
        } else {
            Ok(f()?)
        }
    }

    /// Then-chaining. Returns max(self, f()).
    #[inline(always)]
    fn and<F>(self, f: F) -> Self
    where
        Self: Sized + Ord,
        F: FnOnce() -> Self,
    {
        max(self, f())
    }

    /// Then-chaining. Returns max(self, f()).
    #[inline(always)]
    fn and_try<F, E>(self, f: F) -> Result<Self, E>
    where
        Self: Sized + Ord,
        F: FnOnce() -> Result<Self, E>,
    {
        Ok(max(self, f()?))
    }
}

impl<V, E> ConsumedEvent for Result<V, E>
where
    V: ConsumedEvent,
{
    fn is_consumed(&self) -> bool {
        match self {
            Ok(v) => v.is_consumed(),
            Err(_) => true,
        }
    }
}

/// The baseline outcome for an event-handler.
///
/// A widget can define its own type, if it has more things to report.
/// It would be nice if those types are convertible to/from Outcome.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// The given event has not been used at all.
    #[default]
    Continue,
    /// The event has been recognized, but nothing noticeable has changed.
    /// Further processing for this event may stop.
    /// Rendering the ui is not necessary.
    Unchanged,
    /// The event has been recognized and there is some change due to it.
    /// Further processing for this event may stop.
    /// Rendering the ui is advised.
    Changed,
}

impl ConsumedEvent for Outcome {
    fn is_consumed(&self) -> bool {
        *self != Outcome::Continue
    }
}

/// Widgets often define functions that return bool to indicate a changed state.
/// This converts `true` / `false` to `Outcome::Changed` / `Outcome::Unchanged`.
impl From<bool> for Outcome {
    fn from(value: bool) -> Self {
        if value {
            Outcome::Changed
        } else {
            Outcome::Unchanged
        }
    }
}

/// Breaks the control-flow if the block returns a value
/// for which [ConsumedEvent::is_consumed] is true.
///
/// It does the classic `into()`-conversion and returns the result.
///
/// *The difference to [try_flow] is that this on doesn't Ok-wrap the result.*
///
/// Extras: If you add a marker as in `flow!(log ident: {...});`
/// the result of the operation is written to the log.
#[macro_export]
macro_rules! flow {
    (log $n:ident: $x:expr) => {{
        use log::debug;
        use $crate::ConsumedEvent;
        let r = $x;
        if r.is_consumed() {
            debug!("{} {:#?}", stringify!($n), r);
            return r.into();
        } else {
            debug!("{} continue", stringify!($n));
            _ = r;
        }
    }};
    ($x:expr) => {{
        use $crate::ConsumedEvent;
        let r = $x;
        if r.is_consumed() {
            return r.into();
        } else {
            _ = r;
        }
    }};
}

/// Breaks the control-flow if the block returns a value
/// for which [ConsumedEvent::is_consumed] is true.
///
/// It does the classic `into()`-conversion and returns the result.
///
/// *The difference to [flow] is that this one Ok-wraps the result.*
///
/// Extras: If you add a marker as in `try_flow!(log ident: {...});`
/// the result of the operation is written to the log.
#[macro_export]
macro_rules! try_flow {
    (log $n:ident: $x:expr) => {{
        use log::debug;
        use $crate::ConsumedEvent;
        let r = $x;
        if r.is_consumed() {
            debug!("{} {:#?}", stringify!($n), r);
            return Ok(r.into());
        } else {
            debug!("{} continue", stringify!($n));
            _ = r;
        }
    }};
    ($x:expr) => {{
        use $crate::ConsumedEvent;
        let r = $x;
        if r.is_consumed() {
            return Ok(r.into());
        } else {
            _ = r;
        }
    }};
}
