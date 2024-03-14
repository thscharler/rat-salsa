#![doc = include_str!("../crate.md")]

use std::fmt::Debug;

pub mod layout;
pub mod widget;

pub(crate) mod util;

mod lib_action_trigger;
mod lib_focus;
mod lib_framework;
mod lib_widget;

pub use lib_action_trigger::ActionTrigger;
pub use lib_focus::{Focus, FocusFlag};
pub use lib_framework::{run_tui, TaskSender, ThreadPool, TuiApp};
pub use lib_widget::{
    DefaultKeys, FrameWidget, HandleCrossterm, Input, MouseOnly, RenderFrameWidget, Repaint,
};

/// Converts from a [Result::Err] to a [ControlUI::Err] and returns early.
/// Evaluates to the value of [Result::Ok].
#[macro_export]
macro_rules! try_result {
    ($ex:expr) => {{
        match $ex {
            Ok(v) => v,
            Err(e) => return $crate::ControlUI::Err(e.into()),
        }
    }};
}

/// Cuts the control-flow. If the value is not [ControlUI::Continue] it returns early.
#[macro_export]
macro_rules! check_break {
    ($x:expr) => {{
        let r = $x;
        if !r.is_continue() {
            return r;
        }
        _ = r; // avoid must_use warnings.
    }};
}

/// Cuts the control-flow. If the value is [ControlUI::Err] it returns early.
/// Evaluates to any other value.
#[macro_export]
macro_rules! try_ui {
    ($x:expr) => {{
        let r = $x;
        if r.is_err() {
            return r;
        }
        r
    }};
    ($x:expr, _) => {{
        let r = $x;
        if r.is_err() {
            return r;
        }
        _ = r;
    }};
}

/// UI control flow.
///
/// This is the result type for an event-handler.
///
/// * Continue - Continue with execution.
/// * Err(Err) - Equivalent to Result::Err. Use the macro [try_result] to convert from Result.
/// * Unchanged - Event processed, no UI update necessary.
/// * Changed - Event processed, UI update necessary.
/// * Action(Action) - Run some computation on the model.
/// * Spawn(Action) - Spawn some computation on the worker thread(s).
/// * Break - Break the event loop; end the program.
///
/// There are multiple continuation functions that work with these states.
/// And the macros [try_result!], [check_break!] and [try_ui!]
#[derive(Debug)]
#[must_use]
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
    /// Continue case
    pub fn is_continue(&self) -> bool {
        matches!(self, ControlUI::Continue)
    }

    /// Err case
    pub fn is_err(&self) -> bool {
        matches!(self, ControlUI::Err(_))
    }

    /// Unchanged case
    pub fn is_unchanged(&self) -> bool {
        matches!(self, ControlUI::Unchanged)
    }

    /// Changed case
    pub fn is_changed(&self) -> bool {
        matches!(self, ControlUI::Changed)
    }

    /// Action
    pub fn is_action(&self) -> bool {
        matches!(self, ControlUI::Action(_))
    }

    /// Background action
    pub fn is_spawn(&self) -> bool {
        matches!(self, ControlUI::Spawn(_))
    }

    /// Break case.
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

    /// Convert an error to another error type with into().
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
    /// widget actions to more global ones.
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

    /// Run the continuation if the value is Unchanged.
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

    /// Run the if the value is Unchanged. Allows to return some value.
    pub fn on_unchanged_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::Unchanged => Some(f()),
            _ => None,
        }
    }

    /// Run the continuation if the value is Changed.
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

    /// Run if the value is Changed. Allows to return some value.
    pub fn on_changed_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::Changed => Some(f()),
            _ => None,
        }
    }
}
