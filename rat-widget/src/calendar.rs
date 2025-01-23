use chrono::NaiveDate;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;

mod calendar;
pub(crate) mod event;
mod month;
mod no_selection;
mod range_selection;
mod single_selection;
mod style;

pub use calendar::*;
pub use month::*;
pub use style::*;

/// Selection model for a calendar.
pub trait CalendarSelection {
    /// Clear all selections.
    fn clear(&mut self);

    /// Is the given day selected.
    fn is_selected(&self, date: NaiveDate) -> bool;

    /// Selection lead, or the sole selected day.
    fn lead_selection(&self) -> Option<NaiveDate>;
}

pub mod selection {
    use crate::calendar::CalendarSelection;
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::rc::Rc;

    pub use super::no_selection::*;
    pub use super::range_selection::*;
    pub use super::single_selection::*;

    impl<T: CalendarSelection> CalendarSelection for Rc<RefCell<T>> {
        fn clear(&mut self) {
            self.borrow_mut().clear()
        }

        fn is_selected(&self, date: NaiveDate) -> bool {
            self.borrow().is_selected(date)
        }

        fn lead_selection(&self) -> Option<NaiveDate> {
            self.borrow().lead_selection()
        }
    }
}
