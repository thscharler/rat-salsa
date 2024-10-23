//!
//! If you are tired of scrolling, try paging :)
//!
//! If you have a lot of widgets to display, splitting
//! them into pages is an alternative to scrolling.
//!
//! * Prepare the work with [SinglePager] or [DualPager].
//!
//!     ```rust ignore
//!     let pager = SinglePager::new()
//!         .nav_style(Style::new().fg(THEME.orange[2]))
//!         .style(THEME.gray(0))
//!         .block(Block::new());
//!
//!     let width = pager.layout_width(l2[1]);
//!     ```
//!
//! * [PagerLayout] collects the bounds for all widgets that
//!   will be rendered.
//!
//!     ```rust ignore
//!       if state.layout.width_changed(width) {
//!           let mut pl = PagerLayout::new();
//!           for i in 0..100 {
//!               let handle = cl.add(Rect::new(10, i*11, 15, 10));
//!               state.handles[i] = handle;
//!
//!               if i > 0 && i % 17 == 0 {
//!                   pl.break_before(row);
//!               }
//!           }
//!           state.layout = pl;
//!       }
//!     ```
//!
//! * Use [SinglePager] or [DualPager] to calculate the page breaks.
//!
//!     ```rust ignore
//!       let mut pg_buf = pager
//!             .layout(state.layout.clone())
//!             .into_buffer(l2[1], frame.buffer_mut(), &mut state.pager);
//!     ```
//!
//! * Render your widgets with the help of [SinglePagerBuffer]/[DualPagerBuffer]
//!
//!   Either ad hoc
//!     ```rust ignore
//!         let v_area = pg_buf.layout_area(state.handles[i]);
//!         let w_area = Rect::new(5, v_area.y, 5, 1);
//!         pg_buf.render_widget(Span::from(format!("{:?}:", i)), w_area);
//!     ```
//!   or by referring to a handle
//!     ```rust ignore
//!         pg_buf.render_stateful_handle(
//!             TextInputMock::default()
//!                 .sample(format!("{:?}", state.hundred_areas[i]))
//!                 .style(THEME.limegreen(0))
//!                 .focus_style(THEME.limegreen(2)),
//!             state.handles[i],
//!             &mut state.widget_states[i],
//!         );
//!     ```
//!
//! * Render the finishings.
//!
//!     ```rust ignore
//!       pg_buf
//!           .into_widget()
//!           .render(l2[1], frame.buffer_mut(), &mut state.pager);
//!     ```

mod dual_pager;
mod pager_layout;
mod pager_style;
mod single_pager;

pub use crate::commons::AreaHandle;
pub use dual_pager::*;
pub use pager_layout::*;
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
