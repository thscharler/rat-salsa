//!
//! Alternative to scrolling by page-breaking a layout.
//!
//! If you have a lot of widgets to display, splitting
//! them into pages is an alternative to scrolling.
//!
//! ```rust no_run
//!     # use std::rc::Rc;
//!     # use rat_widget::pager::{SinglePager, SinglePagerState};
//!     # use rat_widget::checkbox::{Checkbox, CheckboxState};
//!     # use ratatui::prelude::*;
//!     # use rat_focus::FocusFlag;
//!     # use rat_widget::layout::{GenericLayout, LayoutForm};
//!     #
//!     # let l2 = [Rect::ZERO, Rect::ZERO];
//!     # struct State {
//!     #      check_states: Vec<CheckboxState>,
//!     #      pager: SinglePagerState<FocusFlag>
//!     #  }
//!     # let mut state = State {
//!     #      check_states: Vec::default(),
//!     #      pager: Default::default(),
//!     #  };
//!     # let mut buf = Buffer::default();
//!
//!     /// Single pager shows the widgets in one column, and
//!     /// can page through the list.
//!     let pager = SinglePager::new();
//!     let size = pager.layout_size(l2[1]);
//!
//!     if !state.pager.valid_layout(size) {
//!           // GenericLayout is very basic.
//!           // Try LayoutForm or layout_edit() instead.
//!           let mut pl = GenericLayout::new();
//!           for i in 0..100 {
//!               pl.add(
//!                     state.check_states[i].focus.clone(), // widget-key
//!                     Rect::new(10, 10+i as u16, 15, 10), // widget area
//!                     None, // label text
//!                     Rect::default() // label area
//!               );
//!           }
//!           state.pager.set_layout(pl);
//!       }
//!
//!       ///
//!       /// Use [SinglePager] or [DualPager] to calculate the page breaks.
//!       ///
//!       let mut pg_buf = pager
//!             .into_buffer(l2[1], &mut buf, &mut state.pager);
//!
//!       ///
//!       /// Render your widgets with the help of [SinglePagerBuffer]/[DualPagerBuffer]
//!       ///
//!       for i in 0..100 {
//!           pg_buf.render(
//!               state.check_states[i].focus.clone(),
//!               || {
//!                 Checkbox::new().text(format!("{:?}", i).to_string())
//!               },
//!               &mut state.check_states[i],
//!           );
//!       }
//!
//! ```

mod dual_pager;
mod form;
#[allow(clippy::module_inception)]
mod pager;
mod pager_nav;
mod pager_style;
mod single_pager;

pub use dual_pager::*;
pub use form::*;
pub use pager::{Pager, PagerBuffer};
pub use pager_nav::{PageNavigation, PageNavigationState};
pub use pager_style::*;
pub use single_pager::*;

pub(crate) mod event {
    use rat_event::{ConsumedEvent, Outcome};

    /// Result of event handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PagerOutcome {
        /// The given event has not been used at all.
        Continue,
        /// The event has been recognized, but the result was nil.
        /// Further processing for this event may stop.
        Unchanged,
        /// The event has been recognized and there is some change
        /// due to it.
        /// Further processing for this event may stop.
        /// Rendering the ui is advised.
        Changed,
        /// Displayed page changed.
        Page(usize),
    }

    impl ConsumedEvent for PagerOutcome {
        fn is_consumed(&self) -> bool {
            *self != PagerOutcome::Continue
        }
    }

    // Useful for converting most navigation/edit results.
    impl From<bool> for PagerOutcome {
        fn from(value: bool) -> Self {
            if value {
                PagerOutcome::Changed
            } else {
                PagerOutcome::Unchanged
            }
        }
    }

    impl From<Outcome> for PagerOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => PagerOutcome::Continue,
                Outcome::Unchanged => PagerOutcome::Unchanged,
                Outcome::Changed => PagerOutcome::Changed,
            }
        }
    }

    impl From<PagerOutcome> for Outcome {
        fn from(value: PagerOutcome) -> Self {
            match value {
                PagerOutcome::Continue => Outcome::Continue,
                PagerOutcome::Unchanged => Outcome::Unchanged,
                PagerOutcome::Changed => Outcome::Changed,
                PagerOutcome::Page(_) => Outcome::Changed,
            }
        }
    }
}
