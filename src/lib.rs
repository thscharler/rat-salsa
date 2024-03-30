#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![doc = include_str!("../crate.md")]

use std::fmt::Debug;

pub mod layout;
pub mod number;
pub mod widget;

pub(crate) mod util;

mod lib_focus;
mod lib_framework;
mod lib_repaint;
mod lib_timer;
mod lib_validate;
mod lib_widget;

pub use lib_focus::{Focus, FocusFlag, HasFocusFlag};
pub use lib_framework::{run_tui, RunConfig, TaskSender, ThreadPool, TuiApp};
pub use lib_repaint::{Repaint, RepaintEvent};
pub use lib_timer::{Timer, TimerEvent, Timers};
pub use lib_validate::{CanValidate, HasValidFlag, ValidFlag};
pub use lib_widget::{
    DefaultKeys, FrameWidget, HandleCrossterm, Input, MouseOnly, RenderFrameWidget,
};

pub mod prelude {
    //! Import common traits.
    pub use super::lib_focus::HasFocusFlag;
    pub use super::lib_validate::{CanValidate, HasValidFlag};
    pub use super::lib_widget::RenderFrameWidget;
}

/// Converts from a [Result::Err] to a [ControlUI::Err] and returns early.
/// Evaluates to the value of [Result::Ok].
///
/// A second parameter `_` silences unused_must_use.
#[macro_export]
macro_rules! try_result {
    ($ex:expr) => {{
        match $ex {
            Ok(v) => v,
            Err(e) => return $crate::ControlUI::Err(e.into()),
        }
    }};
    ($ex:expr, _) => {{
        match $ex {
            Ok(_) => {}
            Err(e) => return $crate::ControlUI::Err(e.into()),
        }
    }};
}

/// Breaks the control-flow. If the value is not [ControlUI::Continue] it returns early.
#[macro_export]
macro_rules! check_break {
    ($x:expr) => {{
        use $crate::optional::OptionalControlUI;
        let r = $x;
        if r.is_control_ui() {
            let s = r.unwrap_control_ui();
            if !s.is_continue() {
                return s;
            } else {
                _ = s;
            }
        } else {
            _ = r;
        }
    }};
}

/// Breaks the control-flow. If the value is [ControlUI::Err] it returns early.
/// Evaluates to any other value.
///
/// A second parameter `_` silences unused_must_use.
#[macro_export]
macro_rules! try_ui {
    ($x:expr) => {{
        use $crate::optional::OptionalControlUI;
        let r = $x;
        if r.is_control_ui() {
            let s = r.unwrap_control_ui();
            if s.is_err() {
                return s;
            } else {
                OptionalControlUI::rewrap_control_ui(s)
            }
        } else {
            r
        }
    }};
    ($x:expr, _) => {{
        use $crate::optional::OptionalControlUI;
        let r = $x;
        if r.is_control_ui() {
            let s = r.unwrap_control_ui();
            if s.is_err() {
                return s;
            } else {
                _ = s;
            }
        }
        _ = r;
    }};
}

pub mod optional {
    use crate::ControlUI;

    /// Just a helper trait for the macros [try_ui!] and [check_break!]
    pub trait OptionalControlUI<A, E> {
        /// Is a ControlUI?
        fn is_control_ui(&self) -> bool;
        /// Unwrap if necessary.
        fn unwrap_control_ui(self) -> ControlUI<A, E>;
        /// Rewrap if necessary.
        fn rewrap_control_ui(v: ControlUI<A, E>) -> Self;
    }
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
    NoChange,
    /// Event processed: changes happened, ui update.
    Change,
    /// Run some action.
    Run(Action),
    /// Start some background action.
    Spawn(Action),
    /// Break the event loop.
    Break,
}

impl<Action, Err> ControlUI<Action, Err> {
    /// Continue case
    #[inline]
    pub fn is_continue(&self) -> bool {
        matches!(self, ControlUI::Continue)
    }

    /// Err case
    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, ControlUI::Err(_))
    }

    /// Unchanged case
    #[inline]
    pub fn is_unchanged(&self) -> bool {
        matches!(self, ControlUI::NoChange)
    }

    /// Changed case
    #[inline]
    pub fn is_changed(&self) -> bool {
        matches!(self, ControlUI::Change)
    }

    /// Action
    #[inline]
    pub fn is_run(&self) -> bool {
        matches!(self, ControlUI::Run(_))
    }

    /// Background action
    #[inline]
    pub fn is_spawn(&self) -> bool {
        matches!(self, ControlUI::Spawn(_))
    }

    /// Break case.
    #[inline]
    pub fn is_break(&self) -> bool {
        matches!(self, ControlUI::Break)
    }

    /// If the value is Continue, change to c.
    #[inline]
    pub fn or(self, c: impl Into<ControlUI<Action, Err>>) -> ControlUI<Action, Err> {
        match self {
            ControlUI::Continue => c.into(),
            _ => self,
        }
    }

    /// Run the continuation if the value is Continue.
    #[inline]
    pub fn or_else(self, f: impl FnOnce() -> ControlUI<Action, Err>) -> ControlUI<Action, Err> {
        match self {
            ControlUI::Continue => f(),
            _ => self,
        }
    }

    /// Run the continuation if the value is Continue. May return some R.
    #[inline]
    pub fn or_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::Continue => Some(f()),
            _ => None,
        }
    }

    /// Does some error conversion.
    #[inline]
    pub fn map_err<OtherErr>(
        self,
        f: impl FnOnce(Err) -> ControlUI<Action, OtherErr>,
    ) -> ControlUI<Action, OtherErr> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => f(e),
            ControlUI::NoChange => ControlUI::NoChange,
            ControlUI::Change => ControlUI::Change,
            ControlUI::Run(a) => ControlUI::Run(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Convert an error to another error type with into().
    #[inline]
    pub fn err_into<OtherErr>(self) -> ControlUI<Action, OtherErr>
    where
        Err: Into<OtherErr>,
    {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e.into()),
            ControlUI::NoChange => ControlUI::NoChange,
            ControlUI::Change => ControlUI::Change,
            ControlUI::Run(a) => ControlUI::Run(a),
            ControlUI::Spawn(a) => ControlUI::Spawn(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Run or Spawn.
    ///
    /// Allows the result action to differ from the input to convert
    /// widget actions to more global ones.
    ///
    /// Caveat: Allows no differentiation between Run and Spawn.
    #[inline]
    pub fn and_then<Other>(
        self,
        f: impl FnOnce(Action) -> ControlUI<Other, Err>,
    ) -> ControlUI<Other, Err> {
        match self {
            ControlUI::Continue => ControlUI::Continue,
            ControlUI::Err(e) => ControlUI::Err(e),
            ControlUI::NoChange => ControlUI::NoChange,
            ControlUI::Change => ControlUI::Change,
            ControlUI::Run(a) => f(a),
            ControlUI::Spawn(a) => f(a),
            ControlUI::Break => ControlUI::Break,
        }
    }

    /// Run the continuation if the value is Run or Spawn. May return some R.
    ///
    /// Caveat: Allows no differentiation between Run and Spawn.
    #[inline]
    pub fn and_do<R>(&self, f: impl FnOnce(&Action) -> R) -> Option<R> {
        match self {
            ControlUI::Run(a) => Some(f(a)),
            ControlUI::Spawn(a) => Some(f(a)),
            _ => None,
        }
    }

    /// Run the continuation if the value is Unchanged.
    #[inline]
    pub fn on_no_change(
        self,
        f: impl FnOnce() -> ControlUI<Action, Err>,
    ) -> ControlUI<Action, Err> {
        match self {
            ControlUI::NoChange => f(),
            _ => self,
        }
    }

    /// Run the if the value is Unchanged. Allows to return some value.
    #[inline]
    pub fn on_no_change_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::NoChange => Some(f()),
            _ => None,
        }
    }

    /// Run the continuation if the value is Changed.
    #[inline]
    pub fn on_change(self, f: impl FnOnce() -> ControlUI<Action, Err>) -> ControlUI<Action, Err> {
        match self {
            ControlUI::Change => f(),
            _ => self,
        }
    }

    /// Run if the value is Changed. Allows to return some value.
    #[inline]
    pub fn on_change_do<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        match self {
            ControlUI::Change => Some(f()),
            _ => None,
        }
    }
}

impl<Action, Err> optional::OptionalControlUI<Action, Err> for ControlUI<Action, Err> {
    #[inline]
    fn is_control_ui(&self) -> bool {
        true
    }

    #[inline]
    fn unwrap_control_ui(self) -> ControlUI<Action, Err> {
        self
    }

    #[inline]
    fn rewrap_control_ui(v: ControlUI<Action, Err>) -> Self {
        v
    }
}

impl<Action, Err> optional::OptionalControlUI<Action, Err> for Option<ControlUI<Action, Err>> {
    #[inline]
    fn is_control_ui(&self) -> bool {
        self.is_some()
    }

    #[inline]
    fn unwrap_control_ui(self) -> ControlUI<Action, Err> {
        self.unwrap()
    }

    #[inline]
    fn rewrap_control_ui(v: ControlUI<Action, Err>) -> Self {
        Some(v)
    }
}
