#![doc = include_str!("../readme.md")]

use log::debug;
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly, Outcome};
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::iter::Zip;
use std::{ptr, vec};

pub mod event {
    //! Rexported eventhandling traits.
    pub use rat_event::{
        crossterm, ct_event, util, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly, Outcome,
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

impl Debug for FocusFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusFlag")
            .field("focus", &self.focus.get())
            .field("gained", &self.gained.get())
            .field("lost", &self.lost.get())
            .finish()
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
    /// Summarizes all the contained FocusFlags.
    /// If any of them has the focus set, this will
    /// be set too.
    ///
    /// This can help if you build compound widgets.
    accu: Option<&'a FocusFlag>,
    /// Area for the whole compound. Only set if accu is some.
    area: Rect,

    /// Areas for each widget.
    areas: Vec<Rect>,
    /// List of flags.
    focus: Vec<&'a FocusFlag>,

    /// List of sub-accumulators and their dependencies.
    /// Keeps track of all the Flags of a compound widget and its
    /// accumulator.
    ///
    /// This is filled if you call [Focus::append]. The accu of the
    /// appended Focus and all its focus-flags are added. And
    /// all the sub_accu of it are appended too.
    sub_accu: Vec<(&'a FocusFlag, Rect, Vec<&'a FocusFlag>)>,
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
    /// Construct a new focus list.
    pub fn new(list: &[&'a dyn HasFocusFlag]) -> Self {
        let mut s = Focus::default();
        for f in list {
            s.focus.push(f.focus());
            s.areas.push(f.area());
        }
        s
    }

    /// Construct a new focus list with an accumulator.
    ///
    /// The accumulator has its focus set if any of the contained
    /// flags has the focus.
    ///
    /// If you have a Focus you can append the Focus of a subcomponent
    /// with [append](Focus::append). In that case the accumulator
    /// of the subcomponent gets its lost+gained flags set too, if
    /// you switch the focus between different subcomponents.
    ///
    /// See `examples/focus_recursive` and `examples/focus_recursive2`
    pub fn new_accu(accu: &'a dyn HasFocusFlag, list: &[&'a dyn HasFocusFlag]) -> Self {
        let mut s = Self {
            accu: Some(accu.focus()),
            area: accu.area(),
            ..Focus::default()
        };
        for f in list {
            s.focus.push(f.focus());
            s.areas.push(f.area());
        }
        s
    }

    /// Add a sub-focus cycle.
    ///
    /// All its widgets are appended to this list. If the sub-cycle
    /// has an accumulator it's added to the accumulators. All
    /// sub-accumulators are appended too.
    pub fn append(mut self, focus: Focus<'a>) -> Self {
        for (focus, area, list) in focus.sub_accu {
            self.sub_accu.push((focus, area, list));
        }
        if let Some(accu) = focus.accu {
            self.sub_accu.push((accu, focus.area, focus.focus.clone()))
        }
        self.focus.extend(focus.focus);
        self.areas.extend(focus.areas);
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

    // accumulate everything
    fn accumulate(&self) {
        if let Some(accu) = self.accu {
            accu.focus.set(false);
            for p in self.focus.iter() {
                if p.focus.get() {
                    accu.focus.set(true);
                    break;
                }
            }
        }

        for (f, _, list) in &self.sub_accu {
            let mut any_gained = false;
            let mut any_lost = false;
            let mut any_focused = false;

            for f in list {
                any_gained |= f.gained.get();
                any_lost |= f.lost.get();
                any_focused |= f.focus.get();
            }

            f.focus.set(any_focused);
            f.lost.set(any_lost && !any_gained);
            f.gained.set(any_gained && !any_lost);
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
        self.accumulate();
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
        self.accumulate();
    }

    /// Reset lost + gained flags.
    /// This is done automatically in `HandleEvent::handle()` for every event.
    pub fn reset_lost_gained(&self) {
        for p in self.focus.iter() {
            p.lost.set(false);
            p.gained.set(false);
        }
        for (p, _, _) in self.sub_accu.iter() {
            p.gained.set(false);
            p.lost.set(false);
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
        self.accumulate();
    }

    /// Change to focus to the given position.
    ///
    pub fn focus_at(&self, col: u16, row: u16) -> bool {
        debug!("focus {:#?}", self);

        let pos = Position::new(col, row);
        for (idx, area) in self.areas.iter().enumerate() {
            if area.contains(pos) {
                debug!("widget area {:?}", area);
                self.focus_idx(idx);
                return true;
            }
        }
        for (_, area, list) in self.sub_accu.iter() {
            if area.contains(pos) {
                debug!("sub area {:?}", area);
                if let Some(ff) = list.first() {
                    self.focus(ff);
                    return true;
                }
            }
        }
        if self.area.contains(pos) {
            debug!("main area {:?}", self.area);
            if let Some(ff) = self.focus.first() {
                self.focus(ff);
                return true;
            }
        }

        false
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
                self.accumulate();
                return true;
            }
        }
        if !self.focus.is_empty() {
            self.focus[0].focus.set(true);
            self.focus[0].gained.set(true);
            self.accumulate();
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
                self.accumulate();
                return true;
            }
        }
        if !self.focus.is_empty() {
            self.focus[0].focus.set(true);
            self.focus[0].gained.set(true);
            self.accumulate();
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
                Outcome::Changed
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                self.prev();
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
                if self.focus_at(*column, *row) {
                    Outcome::Changed
                } else {
                    self.reset_lost_gained();
                    Outcome::NotUsed
                }
            }
            _ => {
                self.reset_lost_gained();
                Outcome::NotUsed
            }
        }
    }
}
