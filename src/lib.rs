use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::fmt::Debug;

pub mod basic;
pub mod button;
pub mod calendar;
pub mod focus;
mod framework;
pub mod input;
pub mod layout;
pub mod menuline;
pub mod message;
pub mod table;
pub mod util;

pub use framework::{run_tui, TaskSender, ThreadPool, TuiApp};

/// Another kind of widget that takes a frame instead of a buffer.
/// Allows to set the cursor while rendering.
/// This also always takes a state, just use () if not needed.
pub trait WidgetExt {
    /// Type of the corresponding state struct.
    type State: ?Sized;

    /// Do render.
    fn render(self, frame: &mut Frame, area: Rect, state: &mut Self::State);
}

/// This trait capture event-handling. It's intended to be implemented
/// on some ui-state struct. It returns a ControlUI state.
pub trait HandleEvent<A, E> {
    /// Event handling.
    fn handle(&mut self, evt: &Event) -> ControlUI<A, E>;
}

#[macro_export]
macro_rules! try_ui {
    ($ex:expr) => {{
        match $ex {
            Ok(v) => v,
            Err(e) => return $crate::tui::libui::ControlUI::Err(e.into()),
        }
    }};
}
#[allow(unused_imports)]
pub use try_ui;

#[macro_export]
macro_rules! cut {
    ($x:expr) => {
        let r = $x;
        if !matches!(r, $crate::tui::libui::ControlUI::Continue) {
            return r;
        }
    };
}

/// UI control flow.
///
/// This is the result type for an event-handler.
///
/// * Continue - Event not processed.
/// * Err(E) - Error occured, doesn't use the usual Result, flattens every state here instead.
///   there is a macro try_ui! instead of ?-operator.
/// * Unchanged - Event processed, no UI update necessary.
/// * Changed - Event processed, UI update necessary.
/// * Action(A) - Some action is triggered. Actions of this type are handled in the main loop.
/// * Focus - A component-like struct requests to get the focus. This event is usually returned
///   due to some mouse interaction. It must be processed in the closest container that manages
///   the focus list. If it bubbles up to the main loop it panics.
/// * Break - Break the event loop.
///
/// There are multiple continuation style functions to act upon ControlUI states.
#[allow(dead_code)]
#[derive(Debug)]
pub enum ControlUI<A, E> {
    /// Event doesn't apply.
    Continue,
    /// Failure condition.
    Err(E),
    /// Event processed, no changes.
    Unchanged,
    /// Event processed, changes happened.
    Changed,
    /// Start some action.
    Action(A),
    /// Start some background action.
    Spawn(A),
    /// Break the main loop.
    Break,
}

#[allow(dead_code)]
impl<A, E> ControlUI<A, E> {
    /// If the value is Continue, change to c.
    pub fn or(self, c: impl Into<ControlUI<A, E>>) -> ControlUI<A, E> {
        match self {
            ControlUI::Continue => c.into(),
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Continue.
    pub fn or_else(self, f: impl FnOnce() -> ControlUI<A, E>) -> ControlUI<A, E> {
        match self {
            ControlUI::Continue => f(),
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Continue. May return some R.
    pub fn or_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::Continue => Some(f()),
            ControlUI::Err(_)
            | ControlUI::Unchanged
            | ControlUI::Changed
            | ControlUI::Action(_)
            | ControlUI::Spawn(_)
            | ControlUI::Break => None,
        }
    }

    pub fn err_into<F>(self) -> ControlUI<A, F>
    where
        E: Into<F>,
    {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e.into()),
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Action. Allows the result action to differ from
    /// the input to convert component actions to more global ones.
    ///
    /// Panic
    /// panics if the value is of ControlUI::Spawn()
    pub fn and_then<B>(self, f: impl FnOnce(A) -> ControlUI<B, E>) -> ControlUI<B, E> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => f(a),
            ControlUI::Spawn(_) => panic!("spawn not possible"),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Unchanged
    pub fn on_unchanged(self, f: impl FnOnce() -> ControlUI<A, E>) -> ControlUI<A, E> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::Unchanged => f(),
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Changed
    pub fn on_changed(self, f: impl FnOnce() -> ControlUI<A, E>) -> ControlUI<A, E> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => f(),
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }
}

/// Extra rendering with passes on the frame to a WidgetExt.
/// Allows setting the cursor in a component.
pub trait FrameExt {
    fn render_ext<W: WidgetExt>(&mut self, widget: W, area: Rect, state: &mut W::State);
}

impl<'a> FrameExt for Frame<'a> {
    fn render_ext<W: WidgetExt>(&mut self, widget: W, area: Rect, state: &mut W::State) {
        widget.render(self, area, state)
    }
}

//
