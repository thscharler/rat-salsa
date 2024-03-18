use crate::lib_widget::HandleCrosstermRepaint;
use crate::util::{next_circular, prev_circular};
use crate::ControlUI;
use crate::{DefaultKeys, MouseOnly, Repaint};
use crossterm::event::Event;
use log::debug;
#[allow(unused_imports)]
use log::error;
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::iter::Zip;
use std::vec;

/// Contains flags for focus.
///
/// This struct is used as part of the widget state. Works with [HasFocusFlag].
#[derive(Debug, Clone, Default)]
pub struct FocusFlag {
    /// A unique tag within one focus-cycle. It is set when the focus cycle is created.
    /// See [Focus::focus]
    pub tag: Cell<u16>,
    /// Focus. See [on_focus!](crate::on_focus!())
    pub focus: Cell<bool>,
    /// This widget just lost the focus. See [validate!](crate::validate!())
    pub lost: Cell<bool>,
}

/// Trait for a widget that has a focus flag.
pub trait HasFocusFlag {
    /// Access to the flag for the rest.
    fn focus(&self) -> &FocusFlag;

    /// Focused?
    fn is_focused(&self) -> bool {
        self.focus().get()
    }

    /// Just lost focus.
    fn lost_focus(&self) -> bool {
        self.focus().lost()
    }

    /// Focus cycle tag.
    fn focus_tag(&self) -> u16 {
        self.focus().tag()
    }
}

/// Contains a valid flag.
/// Can be used as part of the widget state. Works with [HasValidFlag].
#[derive(Debug, Clone)]
pub struct ValidFlag {
    /// Valid flag.
    pub valid: Cell<bool>,
}

/// Trait for a widget that can have a valid/invalid state.
pub trait HasValidFlag {
    /// Access to the flag for the rest.
    fn valid(&self) -> &ValidFlag;

    /// Widget state is valid.
    fn is_valid(&self) -> bool {
        self.valid().get()
    }

    /// Widget state is invalid.
    fn is_invalid(&self) -> bool {
        !self.valid().get()
    }

    /// Change the valid state.
    fn set_valid(&self, valid: bool) {
        self.valid().set(valid)
    }

    /// Set the valid state from a result. Ok == Valid.
    fn set_valid_from<T, E>(&self, result: Result<T, E>) -> Option<T> {
        self.valid().set(result.is_ok());
        result.ok()
    }
}

/// Trait for a widget that has an area for mouse interaction.
pub trait HasArea {
    fn area(&self) -> Rect;
}

/// Trait for a widget evaluating the content.
pub trait Validate {
    fn validate(&mut self) -> bool;
}

/// Keeps track of the focus.
///
/// It works by adding a [FocusFlag] to the State of a widget.
/// Focus is constructed with a list of references to these flags
/// and switches the focus that way. Each widget stays separate otherwise
/// and can pull its focus state from this struct.
///
/// ```rust ignore
/// Focus::new([
///     (&widget1.focus, widget1.area),
///     (&widget2.focus, widget2.area),
/// ]).handle_repaint(evt, repaint, DefaultKeys);
/// ```
///
/// repaint in the example is a [Repaint] as Focus doesn't consume any events.
///
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

    /// Associated tag.
    #[inline]
    pub fn tag(&self) -> u16 {
        self.tag.get()
    }

    /// Just lost the focus.
    #[inline]
    pub fn lost(&self) -> bool {
        self.lost.get()
    }
}

impl Default for ValidFlag {
    fn default() -> Self {
        Self {
            valid: Cell::new(true),
        }
    }
}

impl ValidFlag {
    /// Is valid
    #[inline]
    pub fn get(&self) -> bool {
        self.valid.get()
    }

    /// Set the focus.
    #[inline]
    pub fn set(&self, valid: bool) {
        self.valid.set(valid);
    }
}

/// Validates the given widget if `lost_focus()` is true.
///
/// Uses the traits [HasFocusFlag] and [HasValidFlag] for its function.
///
/// ```rust ignore
/// validate!(state.firstframe.widget1 => {
///     // do something ...
///     true
/// })
/// ```
///
/// There is a variant without the block that uses the [Validate] trait.
///
/// ```rust ignore
/// validate!(state.firstframe.numberfield1);
/// ```
#[macro_export]
macro_rules! validate {
    ($field:expr => $validate:expr) => {{
        let cond = $field.lost_focus();
        if cond {
            let valid = $validate;
            $field.set_valid(valid);
        }
    }};
    ($field:expr) => {{
        let cond = $field.lost_focus();
        if cond {
            let valid = $field.validate();
            $field.set_valid(valid);
        }
    }};
}

/// Executes the block if `lost_focus()` is true.
///
/// This uses the [HasFocusFlag] trait for its function.
#[macro_export]
macro_rules! on_lost {
    ($field:expr => $validate:expr) => {{
        let cond = $field.lost_focus();
        if cond {
            $validate;
        }
    }};
}

/// Executes the expression if `is_focused()` is true.
///
/// ```rust ignore
/// let flow = Focus::new([
///     (&widget1.focus, widget1.area),
///     (&widget2.focus, widget2.area),
/// ]).handle_repaint(evt, repaint, DefaultKeys);
///
/// on_focus!(widget1 => {
///     // ... do something useful ...
/// });
/// on_focus!(widget2 => {
///     // ... do something else ...
/// });
/// ```
///
#[macro_export]
macro_rules! on_focus {
    ($field:expr => $gained:expr) => {{
        let cond = $field.is_focused();
        if cond {
            $gained;
        }
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
    /// Create a focus cycle.
    ///
    /// Take a reference to a FocusFlag and a Rect for mouse-events.
    pub fn new(np: impl IntoIterator<Item = (&'a FocusFlag, Rect)>) -> Self {
        Focus::default().append(np)
    }

    /// Add more to the focus cycle.
    pub fn append(mut self, np: impl IntoIterator<Item = (&'a FocusFlag, Rect)>) -> Self {
        let mut tag = self.focus.len();

        for (f, rect) in np.into_iter() {
            f.tag.set(tag as u16);
            self.focus.push(f);
            self.areas.push(rect);

            tag += 1;
        }

        self
    }

    /// Resets the focus to the last widget that lost the focus.
    ///
    /// The widget then has the focus and no other field has the lost-flag set.
    ///
    /// Can be used to reset the focus after a failed validation without triggering a new one.
    pub fn reset_lost(&self) {
        for f in self.focus.iter() {
            if f.focus.get() {
                f.focus.set(false);
            }
            if f.lost() {
                f.focus.set(true);
            }
        }
    }

    /// Change the focus.
    ///
    /// Sets the focus and the lost flags. Calling this for the widget that currently
    /// has the focus returns *false*, but it resets any lost flag.
    pub fn focus_idx(&self, idx: usize) -> bool {
        let mut change = false;

        for (i, f) in self.focus.iter().enumerate() {
            f.lost.set(false);
            if i == idx {
                if !f.focus.get() {
                    change = true;
                    f.focus.set(true);
                }
            } else {
                if f.focus.get() {
                    f.lost.set(true);
                    f.focus.set(false);
                }
            }
        }

        change
    }

    /// Change the focus using the tag. Flags a repaint if something changed.
    pub fn focus_and_repaint(&self, tag: u16, repaint: &Repaint) {
        if self.focus(tag) {
            repaint.set();
        }
    }

    /// Change the focus using the tag. This resets all lost flags.
    ///
    /// Sets the focus and the lost flags. Calling this for the widget that currently
    /// has the focus returns *false*, but it resets any lost flag.
    pub fn focus_no_lost(&self, tag: u16) -> bool {
        self.focus_impl(tag, false)
    }

    /// Change the focus using the tag.
    ///
    /// Sets the focus and the lost flags. Calling this for the widget that currently
    /// has the focus returns *false*, but it resets any lost flag.
    pub fn focus(&self, tag: u16) -> bool {
        self.focus_impl(tag, true)
    }

    fn focus_impl(&self, tag: u16, use_lost: bool) -> bool {
        for p in self.focus.iter() {
            p.lost.set(false);
            if p.focus.get() {
                p.focus.set(false);
                if use_lost {
                    p.lost.set(true);
                }
            }
        }
        for f in self.focus.iter() {
            if f.tag.get() == tag {
                f.focus.set(true);
                return true;
            }
        }
        false
    }

    pub fn next_and_repaint(&self, repaint: &Repaint) {
        if self.next() {
            repaint.set();
        }
    }

    pub fn prev_and_repaint(&self, repaint: &Repaint) {
        if self.prev() {
            repaint.set();
        }
    }

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *true* and sets both the focus and lost flag.
    /// If no field has the focus the first one gets it.
    pub fn next(&self) -> bool {
        for p in self.focus.iter() {
            p.lost.set(false);
            if p.focus.get() {
                p.lost.set(true);
            }
        }
        for (i, p) in self.focus.iter().enumerate() {
            if p.focus.get() {
                p.focus.set(false);
                let n = next_circular(i, self.focus.len());
                self.focus[n].focus.set(true);
                return true;
            }
        }
        if !self.focus.is_empty() {
            self.focus[0].focus.set(true);
            return true;
        }
        false
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *true* and sets both the focus and lost flag.
    /// If no field has the focus the first one gets it.
    pub fn prev(&self) -> bool {
        for p in self.focus.iter() {
            p.lost.set(false);
            if p.focus.get() {
                p.lost.set(true);
            }
        }
        for (i, p) in self.focus.iter().enumerate() {
            if p.focus.get() {
                p.focus.set(false);
                let n = prev_circular(i, self.focus.len());
                self.focus[n].focus.set(true);
                return true;
            }
        }
        if !self.focus.is_empty() {
            self.focus[0].focus.set(true);
            return true;
        }
        false
    }
}

impl<'a, A, E> HandleCrosstermRepaint<ControlUI<A, E>> for Focus<'a> {
    fn handle_with_repaint(
        &mut self,
        event: &Event,
        repaint: &Repaint,
        _: DefaultKeys,
    ) -> ControlUI<A, E> {
        use crossterm::event::*;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.next() {
                    repaint.set();
                }
                ControlUI::Continue
            }
            Event::Key(KeyEvent {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.prev() {
                    repaint.set();
                }
                ControlUI::Continue
            }
            _ => self.handle_with_repaint(event, repaint, MouseOnly),
        }
    }
}

impl<'a, A, E> HandleCrosstermRepaint<ControlUI<A, E>, MouseOnly> for Focus<'a> {
    /// Only do mouse-events.
    fn handle_with_repaint(
        &mut self,
        event: &Event,
        repaint: &Repaint,
        _: MouseOnly,
    ) -> ControlUI<A, E> {
        use crossterm::event::*;

        match event {
            Event::Mouse(
                MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }
                | MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                },
            ) => {
                for (idx, area) in self.areas.iter().enumerate() {
                    if area.contains(Position::new(*column, *row)) {
                        if self.focus_idx(idx) {
                            repaint.set();
                        }
                        break;
                    }
                }
                ControlUI::Continue
            }
            _ => ControlUI::Continue,
        }
    }
}
