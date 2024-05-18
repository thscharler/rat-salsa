#![doc = include_str!("../readme.md")]

use log::debug;
use rat_event::util::Outcome;
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::iter::Zip;
use std::{ptr, vec};

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::util::Outcome;
    pub use rat_event::{FocusKeys, HandleEvent, MouseOnly, UsedEvent};
}

/// Contains flags for the focus.
/// This struct is embedded in the widget state.
///
/// See [HasFocusFlag], [on_gained!](crate::on_gained!) and
/// [on_lost!](crate::on_lost!).
///
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusFlag {
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

/// Keeps track of the focus.
///
/// It works by adding a [FocusFlag] to the State of a widget.
/// Focus is constructed with a list of references to these flags
/// and switches the focus that way. Each widget stays separate otherwise
/// and can pull its focus state from this struct.
///
/// ```rust ignore
/// use rat_focus::Focus;
///
/// let f = Focus::new(&[
///     &widget1,
///     &widget2
/// ]).handle(evt, FocusKeys);
/// ```
///
/// The result `f` indicates whether the focus has changed.
#[derive(Debug, Default)]
pub struct Focus<'a> {
    /// Areas for each widget.
    pub areas: Vec<Rect>,
    /// List of flags.
    pub focus: Vec<&'a FocusFlag>,
}

impl FocusFlag {
    /// Has the focus.
    #[inline]
    pub fn get(&self) -> bool {
        self.focus.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self) {
        self.focus.set(true);
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.lost.get()
    }

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.gained.get()
    }
}

/// Does a match on the state struct of a widget. If `widget_state.lost_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocusFlag],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_lost;
///
/// on_lost!(
///     state.field1 => {
///         // do checks
///     },
///     state.field2 => {
///         // do checks
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_lost {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocusFlag;
        $(if $field.lost_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If `widget_state.gained_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocusFlag],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_gained;
///
/// on_gained!(
///     state.field1 => {
///         // do prep
///     },
///     state.field2 => {
///         // do prep
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_gained {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocusFlag;
        $(if $field.gained_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If
/// `widget_state.is_focused()` is true the block is executed.
/// There is a `_` branch too, that is evaluated if none of the
/// given widget-states has the focus.
///
/// This requires that `widget_state` implements [HasFocusFlag],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::match_focus;
///
/// let res = match_focus!(
///     state.field1 => {
///         // do this
///         true
///     },
///     state.field2 => {
///         // do that
///         true
///     },
///     _ => {
///         false
///     }
/// );
///
/// if res {
///     // react
/// }
/// ```
///
#[macro_export]
macro_rules! match_focus {
    ($($field:expr => $block:expr),* $(, _ => $final:expr)?) => {{
        use $crate::HasFocusFlag;
        if false {
            unreachable!();
        }
        $(else if $field.is_focused() { $block })*
        $(else { $final })?
    }};
}

impl<'a> IntoIterator for Focus<'a> {
    type Item = (&'a FocusFlag, Rect);
    type IntoIter = Zip<vec::IntoIter<&'a FocusFlag>, vec::IntoIter<Rect>>;

    fn into_iter(self) -> Self::IntoIter {
        self.focus.into_iter().zip(self.areas)
    }
}

impl<'a> Focus<'a> {
    pub fn new(list: &[&'a dyn HasFocusFlag]) -> Self {
        Focus::default().append(list)
    }

    /// Add more to the focus cycle.
    pub fn append(mut self, list: &[&'a dyn HasFocusFlag]) -> Self {
        for f in list {
            self.focus.push(f.focus());
            self.areas.push(f.area());
        }
        self
    }

    // reset flags for a new round.
    fn start_focus_change(&self, set_lost: bool) {
        for p in self.focus.iter() {
            if set_lost {
                p.lost.set(p.focus.get());
            } else {
                p.lost.set(false);
            }
            p.gained.set(false);
            p.focus.set(false);
        }
    }

    /// Sets the focus to the widget.
    ///
    /// Sets focus and gained but not lost. This can be used to prevent validation of the field.
    pub fn focus_no_lost(&self, flag: &FocusFlag) {
        self.start_focus_change(false);
        if let Some(f) = self.focus.iter().find(|f| ptr::eq(**f, flag)) {
            f.focus.set(true);
            f.gained.set(true);
        }
    }

    /// Sets the focus to the widget with `tag`.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    pub fn focus(&self, flag: &FocusFlag) {
        self.start_focus_change(true);
        if let Some(f) = self.focus.iter().find(|f| ptr::eq(**f, flag)) {
            f.focus.set(true);
            f.gained.set(true);
        }
    }

    /// Reset lost + gained flags.
    /// This is done automatically in `HandleEvent::handle()` for every event.
    pub fn reset_lost_gained(&self) {
        for p in self.focus.iter() {
            p.lost.set(false);
            p.gained.set(false);
        }
    }

    /// Change the focus.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If the field at idx has the focus all three are set.
    pub fn focus_idx(&self, idx: usize) {
        self.start_focus_change(true);
        if let Some(f) = self.focus.get(idx) {
            f.focus.set(true);
            f.gained.set(true);
        }
    }

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    ///
    /// If no field has the focus the first one gets it.
    pub fn next(&self) -> bool {
        self.start_focus_change(true);
        for (i, p) in self.focus.iter().enumerate() {
            if p.lost.get() {
                let n = next_circular(i, self.focus.len());
                self.focus[n].focus.set(true);
                self.focus[n].gained.set(true);
                return i != n;
            }
        }
        if !self.focus.is_empty() {
            self.focus[0].focus.set(true);
            self.focus[0].gained.set(true);
            return true;
        }

        false
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *true* and sets the focus, gained and lost flag.
    ///
    /// If no field has the focus the first one gets it.
    pub fn prev(&self) -> bool {
        self.start_focus_change(true);
        for (i, p) in self.focus.iter().enumerate() {
            if p.lost.get() {
                let n = prev_circular(i, self.focus.len());
                self.focus[n].focus.set(true);
                self.focus[n].gained.set(true);
                return i != n;
            }
        }
        if !self.focus.is_empty() {
            self.focus[0].focus.set(true);
            self.focus[0].gained.set(true);
            return true;
        }
        false
    }
}

/// Next but circle around.
fn next_circular(select: usize, max: usize) -> usize {
    if select + 1 < max {
        select + 1
    } else {
        0
    }
}

/// Prev but circle around.
fn prev_circular(select: usize, max: usize) -> usize {
    if select > 0 {
        select - 1
    } else {
        max - 1
    }
}

impl<'a> HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for Focus<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                self.next();
                debug!("tab next -> Changed");
                Outcome::Changed
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                self.prev();
                debug!("tab prev -> Changed");
                Outcome::Changed
            }
            _ => self.handle(event, MouseOnly),
        }
    }
}

impl<'a> HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for Focus<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse down Left for column, row) => {
                for (idx, area) in self.areas.iter().enumerate() {
                    if area.contains(Position::new(*column, *row)) {
                        self.focus_idx(idx);
                        return Outcome::Changed;
                    }
                }
                self.reset_lost_gained();
                Outcome::NotUsed
            }
            _ => {
                self.reset_lost_gained();
                Outcome::NotUsed
            }
        }
    }
}
