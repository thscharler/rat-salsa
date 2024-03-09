//! 1. There is an event-loop with [run_tui()] which runs some [TuiApp].
//!
//! 2. An alternate WidgetExt trait, that receives a frame instead of buffer. This is
//!    helpful if you want the widget to manipulate the cursor. Otherwise, the traits
//!    from ratatui will do nicely.
//! 2.1 To bring the [WidgetExt] trait in line with the standard traits there is a simple
//!     extension trait [FrameExt] that has a render_ext() function.
//!

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
pub(crate) mod util;

pub use framework::{run_tui, TaskSender, ThreadPool, TuiApp};

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

/// Another kind of widget that takes a frame instead of a buffer.
/// Allows to set the cursor while rendering.
/// This also always takes a state, just use () if not needed.
pub trait WidgetExt {
    /// Type of the corresponding state struct.
    type State: ?Sized;

    /// Do render.
    fn render(self, frame: &mut Frame<'_>, area: Rect, state: &mut Self::State);
}

/// This trait capture event-handling. It's intended to be implemented
/// on some ui-state struct. It returns a ControlUI state.
pub trait HandleEvent<Action, Err> {
    /// Event handling.
    fn handle(&mut self, evt: &Event) -> ControlUI<Action, Err>;
}

/// Try macro that uses ControlUI instead of Result for control-flow.
/// Converts a Result::Err into a ControlUI::Err with into() conversion.
#[macro_export]
macro_rules! try_ui {
    ($ex:expr) => {{
        match $ex {
            Ok(v) => v,
            Err(e) => return $crate::ControlUI::Err(e.into()),
        }
    }};
}

/// Cuts the control-flow. If the value is not ControlUI::Continue it returns early.
#[macro_export]
macro_rules! cut {
    ($x:expr) => {
        let r = $x;
        if !r.is_continue() {
            return r;
        }
    };
}

/// UI control flow.
///
/// This is the result type for an event-handler.
///
/// * Continue - Continue with execution.
/// * Err(Err) - Equivalent to Result::Err. Use the macro [try_ui] to convert from Result.
/// * Unchanged - Event processed, no UI update necessary.
/// * Changed - Event processed, UI update necessary.
/// * Action(Action) - Run some computation on the model.
/// * Spawn(Action) - Spawn some computation on the worker thread(s).
/// * Break - Break the event loop; end the program.
///
/// There are multiple continuation functions that work with these states.
#[derive(Debug)]
pub enum ControlUI<Action, Err> {
    /// Continue execution.
    Continue,
    /// Error
    Err(Err),
    /// Event processed: no changes, no ui update.
    Unchanged,
    /// Event processed: changes happened, ui update.
    Changed,
    /// Run some action.
    Action(Action),
    /// Start some background action.
    Spawn(Action),
    /// Break the event loop.
    Break,
}

impl<Action, Err> ControlUI<Action, Err> {
    pub fn is_continue(&self) -> bool {
        matches!(self, ControlUI::Continue)
    }

    pub fn is_err(&self) -> bool {
        matches!(self, ControlUI::Err(_))
    }

    pub fn is_unchanged(&self) -> bool {
        matches!(self, ControlUI::Unchanged)
    }

    pub fn is_changed(&self) -> bool {
        matches!(self, ControlUI::Changed)
    }

    pub fn is_action(&self) -> bool {
        matches!(self, ControlUI::Action(_))
    }

    pub fn is_spawn(&self) -> bool {
        matches!(self, ControlUI::Spawn(_))
    }

    pub fn is_break(&self) -> bool {
        matches!(self, ControlUI::Break)
    }

    /// If the value is Continue, change to c.
    pub fn or(self, c: impl Into<ControlUI<Action, Err>>) -> ControlUI<Action, Err> {
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
    pub fn or_else(self, f: impl FnOnce() -> ControlUI<Action, Err>) -> ControlUI<Action, Err> {
        match self {
            ControlUI::Continue => f(),
            _ => self,
        }
    }

    /// Run the continuation if the value is Continue. May return some R.
    pub fn or_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::Continue => Some(f()),
            _ => None,
        }
    }

    /// Does some error conversion.
    pub fn map_err<F>(self, f: impl FnOnce(Err) -> ControlUI<Action, F>) -> ControlUI<Action, F> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => f(e),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Just convert the error to another type with into().
    pub fn err_into<F>(self) -> ControlUI<Action, F>
    where
        Err: Into<F>,
    {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e.into()),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => ControlUI::Action(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Action or Spawn.
    ///
    /// Allows the result action to differ from the input to convert
    /// component actions to more global ones.
    ///
    /// Caveat: Allows no differentiation between Action and Spawn.
    pub fn and_then<B>(self, f: impl FnOnce(Action) -> ControlUI<B, Err>) -> ControlUI<B, Err> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::Unchanged => ControlUI::Unchanged,
            ControlUI::Changed => ControlUI::Changed,
            ControlUI::Action(a) => f(a),
            ControlUI::Spawn(a) => f(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Unchanged
    pub fn on_unchanged(
        self,
        f: impl FnOnce() -> ControlUI<Action, Err>,
    ) -> ControlUI<Action, Err> {
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
    pub fn on_changed(self, f: impl FnOnce() -> ControlUI<Action, Err>) -> ControlUI<Action, Err> {
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
