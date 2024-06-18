#![doc = include_str!("../readme.md")]

mod focus;

#[allow(unused_imports)]
use log::debug;
use rat_event::HandleEvent;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::iter::Zip;
use std::{ptr, vec};

pub use crate::focus::Focus;

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::{
        crossterm, ct_event, flow, flow_ok, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly,
        Outcome,
    };
}

/// Contains flags for the focus.
/// This struct is embedded in the widget state.
///
/// See [HasFocusFlag], [on_gained!](crate::on_gained!) and
/// [on_lost!](crate::on_lost!).
///
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FocusFlag {
    /// Field name for debugging purposes.
    pub name: Cell<&'static str>,
    /// Focus.
    pub focus: Cell<bool>,
    /// This widget just gained the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_gained!](crate::on_gained!)
    pub gained: Cell<bool>,
    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    pub lost: Cell<bool>,
}

/// Trait for a widget that has a focus flag.
pub trait HasFocusFlag {
    /// Access to the flag for the rest.
    fn focus(&self) -> &FocusFlag;

    /// Access the area for mouse focus.
    fn area(&self) -> Rect;

    /// Focused?
    fn is_focused(&self) -> bool {
        self.focus().get()
    }

    /// Just lost focus.
    fn lost_focus(&self) -> bool {
        self.focus().lost()
    }

    /// Just gained focus.
    fn gained_focus(&self) -> bool {
        self.focus().gained()
    }
}

/// Is this a container widget of sorts.
pub trait HasFocus {
    /// Returns a Focus struct.
    fn focus(&self) -> Focus<'_>;
}

impl Debug for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusFlag")
            .field("name", &self.name.get())
            .field("focus", &self.focus.get())
            .field("gained", &self.gained.get())
            .field("lost", &self.lost.get())
            .finish()
    }
}
