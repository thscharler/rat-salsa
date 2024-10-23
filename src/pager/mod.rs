//!
//! If you are tired of scrolling, try paging :)
//!
//! If you have a lot of widgets to display, splitting
//! them into pages is an alternative to scrolling.
//!
//! [PageLayout] helps with the dynamic page-breaks.
//! [SinglePage] and [DualPage] are the widgets that display
//! everything as one or two columns.
//!
//! Same as the other containers in this crate they leave the
//! actual rendering of the widgets to the caller.
//! [relocate](SinglePageState::relocate) tells you
//! if a widget is visible and where it should be rendered.
//!
