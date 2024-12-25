//!
//! If you are tired of scrolling, try paging :)
//!
//! If you have a lot of widgets to display, splitting
//! them into pages is an alternative to scrolling.
//!
//! ```rust no_run
//!     # use rat_widget::pager::{SinglePager, AreaHandle, PagerLayout, SinglePagerState};
//!     # use rat_widget::checkbox::{Checkbox, CheckboxState};
//!     # use ratatui::prelude::*;
//!     #
//!     # let l2 = [Rect::ZERO, Rect::ZERO];
//!     # struct State {
//!     #      layout: PagerLayout,
//!     #      handles: Vec<AreaHandle>,
//!     #      check_states: Vec<CheckboxState>,
//!     #      pager: SinglePagerState
//!     #  }
//!     # let mut state = State {
//!     #      layout: PagerLayout::new(1),
//!     #      handles: Vec::default(),
//!     #      pager: Default::default(),
//!     #      check_states: Vec::default()
//!     #  };
//!     # let mut buf = Buffer::default();
//!
//!     /// Single pager shows the widgets in one column, and
//!     /// can page through the list.
//!     let pager = SinglePager::new();
//!     let width = pager.layout_width(l2[1]);
//!
//!     if state.layout.width_changed(width) {
//!           ///
//!           /// PagerLayout collects the bounds for all widgets.
//!           ///
//!           let mut pl = PagerLayout::new(1);
//!           for i in 0..100 {
//!               let handle = pl.add(&[Rect::new(10, i*11, 15, 10)]);
//!               state.handles[i as usize] = handle;
//!
//!               if i > 0 && i % 17 == 0 {
//!                   pl.break_before(i);
//!               }
//!           }
//!           state.layout = pl;
//!       }
//!
//!       ///
//!       /// Use [SinglePager] or [DualPager] to calculate the page breaks.
//!       ///
//!       let mut pg_buf = pager
//!             .layout(state.layout.clone())
//!             .into_buffer(l2[1], &mut buf, &mut state.pager);
//!
//!       ///
//!       /// Render your widgets with the help of [SinglePagerBuffer]/[DualPagerBuffer]
//!       ///
//!       for i in 0..100 {
//!           // calculate an area
//!           let v_area = pg_buf.layout().layout_area(state.handles[i])[0];
//!           let w_area = Rect::new(5, v_area.y, 5, 1);
//!           pg_buf.render_widget_area(Span::from(format!("{:?}:", i)), w_area);
//!
//!           // use the handle
//!           pg_buf.render(
//!               Checkbox::new()
//!                   .text(format!("{:?}", state.handles[i]).to_string()),
//!               state.handles[i],
//!               0,
//!               &mut state.check_states[i],
//!           );
//!       }
//!       ///
//!       /// Render the finishings.
//!       ///
//!       pg_buf
//!           .into_widget()
//!           .render(l2[1], &mut buf, &mut state.pager);
//! ```

mod dual_pager;
mod pager;
mod pager_layout;
mod pager_nav;
mod pager_style;
mod single_pager;

pub use crate::commons::AreaHandle;
pub use dual_pager::*;
pub use pager::{Pager, PagerBuffer};
pub use pager_layout::*;
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
