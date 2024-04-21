/// Helper trait for [crate::tr!].
pub trait SplitResult<T, E> {
    /// Split self in an Ok part and an Err part.
    fn split(self) -> (Option<T>, Option<E>);
}

impl<T, E> SplitResult<T, E> for Result<T, E> {
    fn split(self) -> (Option<T>, Option<E>) {
        match self {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e)),
        }
    }
}

impl<T, E> SplitResult<ControlUI<T, E>, E> for ControlUI<T, E> {
    fn split(self) -> (Option<ControlUI<T, E>>, Option<E>) {
        match self {
            ControlUI::Err(e) => (None, Some(e)),
            v => (Some(v), None),
        }
    }
}

/// Converts from a [Result::Err] to a [ControlUI::Err] and returns early.
/// Evaluates to the value of [Result::Ok].
///
/// and
///
/// Checks if the value is a ControlUI::Err and returns early.
/// Evaluates to the original ControlUI value.
///
/// A second parameter `_` silences unused_must_use.
#[macro_export]
macro_rules! tr {
    ($ex:expr) => {{
        use $crate::{ControlUI, SplitResult};
        let x = $ex;
        let s = SplitResult::split(x);
        match s {
            (None, Some(e)) => {
                let ee = e.into();
                return ControlUI::Err(ee);
            }
            (Some(v), None) => v,
            _ => unreachable!(),
        }
    }};
    ($ex:expr, _) => {{
        use $crate::{ControlUI, SplitResult};
        let x = $ex;
        let s = SplitResult::split(x);
        match s {
            (None, Some(e)) => {
                let ee = e.into();
                return ControlUI::Err(ee);
            }
            (Some(_), None) => {}
            _ => unreachable!(),
        }
    }};
}

/// Breaks the control-flow. If the value is not [ControlUI::Continue] it returns early.
#[macro_export]
macro_rules! check_break {
    ($x:expr) => {{
        let r = $x;
        if !r.is_continue() {
            return r;
        } else {
            _ = r;
        }
    }};
}

/// UI control flow.
///
/// This is the result type for an event-handler.
///
/// * Continue - Continue with execution.
/// * Err(Err) - Equivalent to Result::Err. Use the macro [crate::tr] to convert from Result.
/// * Unchanged - Event processed, no UI update necessary.
/// * Changed - Event processed, UI update necessary.
/// * Action(Action) - Run some computation on the model.
/// * Spawn(Action) - Spawn some computation on the worker thread(s).
/// * Break - Break the event loop; end the program.
///
/// There are multiple continuation functions that work with these states.
/// And the macros [crate::tr!] and [crate::check_break!]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[must_use]
pub enum ControlUI<Action = (), Err = ()> {
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
    pub fn or_do<R: Default>(&self, f: impl FnOnce() -> R) -> R {
        match self {
            ControlUI::Continue => f(),
            _ => R::default(),
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
    pub fn and_do<R: Default>(&self, f: impl FnOnce(&Action) -> R) -> R {
        match self {
            ControlUI::Run(a) => f(a),
            ControlUI::Spawn(a) => f(a),
            _ => R::default(),
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
    pub fn on_no_change_do<R: Default>(&self, f: impl FnOnce() -> R) -> R {
        match self {
            ControlUI::NoChange => f(),
            _ => R::default(),
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
    pub fn on_change_do<R: Default>(&self, f: impl FnOnce() -> R) -> R {
        match self {
            ControlUI::Change => f(),
            _ => R::default(),
        }
    }
}
