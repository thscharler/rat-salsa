#![allow(clippy::collapsible_if)]
#![allow(clippy::collapsible_else_if)]
#![doc = include_str!("../readme.md")]

pub mod layout;
pub mod widget;

pub(crate) mod util;

mod lib_control_flow;
mod lib_event_handler;
mod lib_focus;
mod lib_framework;
mod lib_repaint;
mod lib_scroll;
mod lib_selection;
mod lib_timer;
mod lib_validate;
mod lib_widget;
pub mod rere;

pub use lib_control_flow::{ControlUI, SplitResult};
pub use lib_event_handler::{modifiers, DefaultKeys, HandleCrossterm, MouseOnly};
pub use lib_focus::{Focus, FocusFlag, HasFocusFlag};
pub use lib_framework::{run_tui, RunConfig, TuiApp};
pub use lib_repaint::{Repaint, RepaintEvent};
pub use lib_scroll::{HasScrolling, ScrollParam, ScrolledWidget};
pub use lib_selection::{ListSelection, NoSelection, SetSelection, SingleSelection};
pub use lib_timer::{Timed, TimerDef, TimerEvent, Timers};
pub use lib_validate::{CanValidate, HasValidFlag, ValidFlag};
pub use lib_widget::{FrameWidget, RenderFrameWidget};

pub use pure_rust_locales::Locale;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
