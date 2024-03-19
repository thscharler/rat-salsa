use crate::FocusFlag;
use std::cell::Cell;

/// A valid flag for a widget that can indicate such a state.
///
/// Can be used as part of the widget state.
///
/// See [HasValidFlag], [CanValidate], [validate!](crate::validate!)
#[derive(Debug, Clone)]
pub struct ValidFlag {
    /// Valid flag.
    pub valid: Cell<bool>,
}

/// Trait for a widget that can have a valid/invalid state.
pub trait HasValidFlag {
    /// Access to the flag for the rest.
    fn valid(&self) -> &ValidFlag;

    /// Widget state is valid.
    fn is_valid(&self) -> bool {
        self.valid().get()
    }

    /// Widget state is invalid.
    fn is_invalid(&self) -> bool {
        !self.valid().get()
    }

    /// Change the valid state.
    fn set_valid(&self, valid: bool) {
        self.valid().set(valid)
    }

    /// Set the valid state from a result. Ok == Valid.
    fn set_valid_from<T, E>(&self, result: Result<T, E>) -> Option<T> {
        self.valid().set(result.is_ok());
        result.ok()
    }
}

impl Default for ValidFlag {
    fn default() -> Self {
        Self {
            valid: Cell::new(true),
        }
    }
}

impl ValidFlag {
    /// Is valid
    #[inline]
    pub fn get(&self) -> bool {
        self.valid.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self, valid: bool) {
        self.valid.set(valid);
    }
}

/// Validates the given widget if `lost_focus()` is true.
///
/// Uses the traits [HasFocusFlag](crate::HasFocusFlag)  and [HasValidFlag](crate::HasValidFlag)
/// for its function.
///
/// ```rust ignore
/// validate!(state.firstframe.widget1 => {
///     // do something ...
///     true
/// })
/// ```
///
/// There is a variant without the block that uses the [CanValidate](crate::CanValidate) trait.
///
/// ```rust ignore
/// validate!(state.firstframe.numberfield1);
/// ```
#[macro_export]
macro_rules! validate {
    ($field:expr => $validate:expr) => {{
        use $crate::{HasFocusFlag, HasValidFlag};
        let cond = $field.lost_focus();
        if cond {
            let valid = $validate;
            $field.set_valid(valid);
        }
    }};
    ($field:expr) => {{
        use $crate::{CanValidate, HasFocusFlag};
        let cond = $field.lost_focus();
        if cond {
            $field.validate();
        }
    }};
}

/// Trait for a widget that can do some sort of validation.
pub trait CanValidate {
    /// Run some validation for the widget.
    ///
    /// This is an extra entrypoint for an application, as any widget will validate
    /// its state when doing a [ratatui::widgets::StatefulWidget::render].
    ///
    /// Note: At the point `render` is called [FocusFlag::lost](FocusFlag::lost) and
    /// [FocusFlag::gained](FocusFlag::gained) will still be valid. So content
    /// validation can be conditional on one of those.
    fn validate(&mut self);
}
