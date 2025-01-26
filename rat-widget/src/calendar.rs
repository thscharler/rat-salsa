//!
//! Calendars.
//!
//! There's a Month widget to render one month of a calendar.
//! This has limited behaviour and no built-in scrolling.
//! Use a CalendarState<1> if you need that.
//!
//! There is no Calendar widget as such, but there is a CalendarState
//! that can manage N months as a full calendar. With movement/scrolling/selection
//! over all months.
//!
//! There is a Calendar3 widget that can display 3 months in a line/column.
//! Use this as a baseline for other layouts.
//!
//! There are 3 selection-models that can be used:
//! - [NoSelection](selection::NoSelection)  Scrolling is still possible.
//! - [SingleSelection](selection::SingleSelection) Selection of a single day.
//! - [RangeSelection](selection::RangeSelection) Selection of any date range.
//!

use chrono::{Datelike, Days, Months, NaiveDate};

#[allow(clippy::module_inception)]
mod calendar;
mod calendar3;
pub(crate) mod event;
mod month;
mod no_selection;
mod range_selection;
mod single_selection;
mod style;

pub use calendar::*;
pub use calendar3::*;
pub use month::*;
pub use style::*;

/// Selection model for a calendar.
#[allow(clippy::len_without_is_empty)]
pub trait CalendarSelection {
    /// Select day count.
    fn count(&self) -> usize;

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
        fn count(&self) -> usize {
            self.borrow().count()
        }

        fn is_selected(&self, date: NaiveDate) -> bool {
            self.borrow().is_selected(date)
        }

        fn lead_selection(&self) -> Option<NaiveDate> {
            self.borrow().lead_selection()
        }
    }
}

fn is_first_day_of_month(date: NaiveDate) -> bool {
    date.day() == 1
}

fn is_last_day_of_month(date: NaiveDate) -> bool {
    date.month() != (date + Days::new(1)).month()
}

fn first_day_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("date")
}

fn last_day_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("date") + Months::new(1) - Days::new(1)
}

fn is_same_month(start: NaiveDate, end: NaiveDate) -> bool {
    start.year() == end.year() && start.month() == end.month()
}

fn is_same_week(start: NaiveDate, end: NaiveDate) -> bool {
    start.iso_week() == end.iso_week()
}
