use rat_event::{ConsumedEvent, Outcome};
use std::cmp::Ordering;
use std::mem;

/// Result enum for event handling.
///
/// The result of an event is processed immediately after the
/// function returns, before polling new events. This way an action
/// can trigger another action which triggers the repaint without
/// other events intervening.
///
/// If you ever need to return more than one result from event-handling,
/// you can hand it to AppContext/RenderContext::queue(). Events
/// in the queue are processed in order, and the return value of
/// the event-handler comes last. If an error is returned, everything
/// send to the queue will be executed nonetheless.
///
/// __See__
///
/// - [flow!](rat_event::flow)
/// - [try_flow!](rat_event::try_flow)
/// - [ConsumedEvent]
#[derive(Debug, Clone, Copy)]
#[must_use]
#[non_exhaustive]
pub enum Control<Event> {
    /// Continue with event-handling.
    /// In the event-loop this waits for the next event.
    Continue,
    /// Break event-handling without repaint.
    /// In the event-loop this waits for the next event.
    Unchanged,
    /// Break event-handling and repaints/renders the application.
    /// In the event-loop this calls `render`.
    Changed,
    /// Eventhandling can cause secondary application specific events.
    /// One common way is to return this `Control::Message(my_event)`
    /// to reenter the event-loop with your own secondary event.
    ///
    /// This acts quite like a message-queue to communicate between
    /// disconnected parts of your application. And indeed there is
    /// a hidden message-queue as part of the event-loop.
    ///
    /// The other way is to call [SalsaAppContext::queue] to initiate such
    /// events.
    Event(Event),
    /// A dialog close event. In the main loop it will be handled
    /// just like [Control::Event]. But the DialogStack can react
    /// separately and close the window.
    #[cfg(feature = "dialog")]
    Close(Event),
    /// Quit the application.
    Quit,
}

impl<Event> Eq for Control<Event> {}

impl<Event> PartialEq for Control<Event> {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl<Event> Ord for Control<Event> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Control::Continue => match other {
                Control::Continue => Ordering::Equal,
                Control::Unchanged => Ordering::Less,
                Control::Changed => Ordering::Less,
                Control::Event(_) => Ordering::Less,
                #[cfg(feature = "dialog")]
                Control::Close(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Unchanged => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Equal,
                Control::Changed => Ordering::Less,
                Control::Event(_) => Ordering::Less,
                #[cfg(feature = "dialog")]
                Control::Close(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Changed => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Equal,
                Control::Event(_) => Ordering::Less,
                #[cfg(feature = "dialog")]
                Control::Close(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            Control::Event(_) => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Event(_) => Ordering::Equal,
                #[cfg(feature = "dialog")]
                Control::Close(_) => Ordering::Less,
                Control::Quit => Ordering::Less,
            },
            #[cfg(feature = "dialog")]
            Control::Close(_) => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Event(_) => Ordering::Greater,
                Control::Close(_) => Ordering::Equal,
                Control::Quit => Ordering::Less,
            },
            Control::Quit => match other {
                Control::Continue => Ordering::Greater,
                Control::Unchanged => Ordering::Greater,
                Control::Changed => Ordering::Greater,
                Control::Event(_) => Ordering::Greater,
                #[cfg(feature = "dialog")]
                Control::Close(_) => Ordering::Greater,
                Control::Quit => Ordering::Equal,
            },
        }
    }
}

impl<Event> PartialOrd for Control<Event> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Event> ConsumedEvent for Control<Event> {
    fn is_consumed(&self) -> bool {
        !matches!(self, Control::Continue)
    }
}

impl<Event, T: Into<Outcome>> From<T> for Control<Event> {
    fn from(value: T) -> Self {
        let r = value.into();
        match r {
            Outcome::Continue => Control::Continue,
            Outcome::Unchanged => Control::Unchanged,
            Outcome::Changed => Control::Changed,
        }
    }
}
