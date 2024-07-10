#![doc = include_str!("../readme.md")]

use std::cmp::max;

pub mod crossterm;
pub mod util;

/// Default key bindings when focused.
/// Also handles all events that are MouseOnly.
#[derive(Debug, Default)]
pub struct FocusKeys;

/// Handler for mouse events only.
/// * Useful when writing a new key binding.
/// * FocusKeys should only be handled by the focused widget,
///   while mouse events are not bound by the focus.
#[derive(Debug, Default)]
pub struct MouseOnly;

/// Runs only the event-handling for the popup-parts of a widget.
/// These should be run before the standard `FocusKey` or `MouseOnly` event-handlers,
/// to mitigate the front/back problem of overlaying widgets.
///
/// There is no separate `MouseOnlyPopup`, as popups should always have the
/// input focus.
#[derive(Debug)]
pub struct Popup;

/// Runs the event-handling for a dialog like widget.
/// Such widgets consume all incoming events. This blocks every widget in
/// the 'background'.
#[derive(Debug)]
pub struct Dialog;

///
/// A very broad trait for an event handler for widgets.
///
/// As widget types are only short-lived, this trait should be implemented
/// for the state type. Thereby it can modify any state, and it can
/// return an arbitrary result, that fits the widget.
///
pub trait HandleEvent<Event, Qualifier, R: ConsumedEvent> {
    /// Handle an event.
    ///
    /// * self - Should be the widget state.
    /// * event - Event
    /// * qualifier - Allows specifying/restricting the behaviour of the
    ///   event-handler.
    ///
    ///   This library defines two possible types:
    ///   * FocusKeys - The event-handler does all the interactions for
    ///     a focused widget. This calls the event-handler for MouseOnly too.
    ///   * MouseOnly - Interactions for a non-focused widget. Mostly only
    ///     reacting to mouse-events. But might check for hotkeys or the like.
    ///
    /// Further ideas:
    /// * ReadOnly
    /// * Additional special behaviour like DoubleClick, HotKeyAlt, HotKeyCtrl.
    /// * Opt-in behaviour like different key-bindings.
    /// * Configurable key-map.
    /// * Other context or configuration parameters.
    ///
    fn handle(&mut self, event: &Event, qualifier: Qualifier) -> R;
}

impl<E, Q> HandleEvent<E, Q, Outcome> for () {
    fn handle(&mut self, _event: &E, _qualifier: Q) -> Outcome {
        Outcome::NotUsed
    }
}

/// When composing several widgets, the minimum information from the outcome
/// of the inner widget is, whether it used & consumed the event.
///
/// This allows shortcutting the event-handling and prevents double
/// interactions with the event. The inner widget can special case an event,
/// and the fallback behaviour on the outer layer should not run too.
pub trait ConsumedEvent {
    fn is_consumed(&self) -> bool;

    /// Or-Else chaining with `is_consumed()` as the split.
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

    /// Chaining. Returns max(self, f()).
    fn then<F>(self, f: F) -> Self
    where
        Self: Sized + Ord,
        F: FnOnce() -> Self,
    {
        max(self, f())
    }
}

impl<V, E> ConsumedEvent for Result<V, E>
where
    V: ConsumedEvent,
{
    fn is_consumed(&self) -> bool {
        match self {
            Ok(v) => v.is_consumed(),
            Err(_) => true, // this can somewhat be argued for.
        }
    }
}

/// A baseline Outcome for event-handling.
///
/// A widget can define its own, if it has more things to report.
/// It would be nice of the widget though, if its outcome would be
/// convertible to this baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    /// The given event has not been used at all.
    NotUsed,
    /// The event has been recognized, but the result was nil.
    /// Further processing for this event may stop.
    Unchanged,
    /// The event has been recognized and there is some change
    /// due to it.
    /// Further processing for this event may stop.
    /// Rendering the ui is advised.
    Changed,
}

impl ConsumedEvent for Outcome {
    fn is_consumed(&self) -> bool {
        !matches!(self, Outcome::NotUsed)
    }
}

/// Event functions in widgets often return bool to indicate
/// a meaningful change occured. This converts `true / false` to
///` Outcome::Changed / Outcome::Unchanged`.
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
/// for with [ConsumedEvent::is_consumed] is true.
///
/// It then does the classic `into()`-conversion and returns.
///
/// Special widget result-types are encourage to map down to Outcome
/// as a baseline.
///
/// *The difference to [flow] is that this one Ok-wraps the result.*
///
/// Extras: If you add a marker as in `flow_ok!(log ident: {...});`
/// the result of the operation is written to the log.
///
/// Extras: Combining this with the result of focus-handling is tricky.
/// The result of processing events for focus should not break early,
/// as widgets probably want to act on the same event. That leads
/// to two result-values to be considered.
/// Therefore, one more extension for this macro:
/// `flow_ok!(_do_something_with_an_outcome(), consider focus_outcome)`
/// Where `focus_outcome` is the variable that holds the result.
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
    ($x:expr, consider $f:expr) => {{
        use std::cmp::max;
        use $crate::ConsumedEvent;
        let r = $x;
        if r.is_consumed() {
            return max(r.into(), $f);
        } else {
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

/// Breaks and Ok-wraps the control-flow if the block
/// returns a value for with [ConsumedEvent::is_consumed] is true.
///
/// It then does the classic `into()`-conversion and wraps the
/// result in `Ok()`.
///
/// *The difference to [flow] is that this one Ok-wraps the result.*
///
/// Extras: If you add a marker as in `flow_ok!(log ident: {...});`
/// the result of the operation is written to the log.
///
/// Extras: Combining this with the result of focus-handling is tricky.
/// The result of processing events for focus should not break early,
/// as widgets probably want to act on the same event. That leads
/// to two result-values to be considered.
/// Therefore, one more extension for this macro:
/// `flow_ok!(_do_something_with_an_outcome(), consider focus_outcome)`
/// Where `focus_outcome` is the variable that holds the result.
#[macro_export]
macro_rules! flow_ok {
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
    ($x:expr, consider $f:expr) => {{
        use std::cmp::max;
        use $crate::ConsumedEvent;
        let r = $x;
        if r.is_consumed() {
            return Ok(max(r.into(), $f));
        } else {
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

/// One more control-flow macro with ConsumedEvent.
///
/// If you don't want to return early with is_consumed(), you
/// can define a variable, and or_else! the different options.
///
/// ```not_rust
/// let mut r;
///
/// r = first_activity();
/// or_else!(r, second_activity());
/// or_else!(r, third_activity());
/// ```
///
/// This executes `second_activity` if !r.is_consumed() and stores the
/// result in r. The same with `third_activity` ...
///
#[macro_export]
macro_rules! or_else {
    ($x:ident, $e:expr) => {
        if !$crate::ConsumedEvent::is_consumed(&$x) {
            $x = $e;
        }
    };
}
