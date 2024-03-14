use crate::util::{next_circular, prev_circular};
use crate::ControlUI;
use crate::{DefaultKeys, HandleCrossterm, MouseOnly, Repaint};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::error;
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::iter::Zip;
use std::vec;

/// Flag structure to be used in widget states.
#[derive(Debug, Clone)]
pub struct FocusFlag {
    /// A unique tag within one focus-cycle. It is set when the focus cycle is created.
    /// See [Focus::focus]
    pub tag: Cell<u16>,
    /// Focus. See [on_focus]
    pub focus: Cell<bool>,
    /// This widget just lost the focus. See [validate]
    pub lost: Cell<bool>,
}

/// Keeps track of the focus.
///
/// It works by adding a [FocusFlag] to the State of a widget.
/// Focus is constructed with a list of references to these flags
/// and switches the focus that way. Each widget stays separate otherwise
/// and can pull its focus state from this struct.
///
/// ```
/// # use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
/// use rat_salsa::{check_break, DefaultKeys, Focus, HandleCrossterm, Repaint};
/// # use rat_salsa::widget::button::ButtonState;
/// # let widget1 = ButtonState::default();
/// # let widget2 = ButtonState::default();
/// # let vevt = crossterm::event::Event::Key(KeyEvent {
/// #     code: KeyCode::Tab,
/// #     modifiers: KeyModifiers::NONE,
/// #     kind: KeyEventKind::Press,
/// #     state: KeyEventState::NONE
/// # });
/// # let evt = &vevt;
/// # let vrepaint = Repaint::default();
/// # let repaint = &vrepaint;
///
/// check_break!(
///     Focus::new([
///         (&widget1.focus, widget1.area),
///         (&widget2.focus, widget2.area),
///     ]).handle(evt, repaint, DefaultKeys)
/// );
/// ```
#[derive(Debug)]
pub struct Focus<'a> {
    /// Areas for each widget.
    pub areas: Vec<Rect>,
    /// List of flags.
    pub focus: Vec<&'a FocusFlag>,
}

impl Default for FocusFlag {
    #[inline]
    fn default() -> Self {
        Self {
            tag: Cell::new(0),
            focus: Cell::new(false),
            lost: Cell::new(false),
        }
    }
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

/// Validates the given widget if `focus.lost()` is true.
///
/// ```
/// # use rat_salsa::{FocusFlag, validate};
/// # #[derive(Default)]
/// # struct State {
/// #     pub focus: FocusFlag,
/// #     pub valid: bool,
/// # }
/// # let mut state = State::default();
/// validate!(state.firstframe.widget1 => {
///     // do something ...
///     true
/// })
/// ```
///
/// It expects that the widget has the fields `focus: FocusFlag` and `valid: bool` that
/// are both public.
#[macro_export]
macro_rules! validate {
    ($field:expr => $validate:expr) => {{
        let cond = $field.focus.lost();
        if cond {
            let valid = $validate;
            $field.valid = valid;
        }
    }};
}

/// Executes the expression if `focus.get()` is true.
///
/// This is only useful if combined with [ControlUI::on_changed].
///
/// ```
/// # use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
/// use rat_salsa::{ControlUI, DefaultKeys, Focus, HandleCrossterm, on_focus, Repaint};
/// # use rat_salsa::widget::button::ButtonState;
/// # let widget1 = ButtonState::default();
/// # let widget2 = ButtonState::default();
/// # let vevt = crossterm::event::Event::Key(KeyEvent {
/// #     code: KeyCode::Tab,
/// #     modifiers: KeyModifiers::NONE,
/// #     kind: KeyEventKind::Press,
/// #     state: KeyEventState::NONE
/// # });
/// # let evt = &vevt;
/// # let vrepaint = Repaint::default();
/// # let repaint = Some(&vrepaint);
///
/// let flow = Focus::new([
///     (&widget1.focus, widget1.area),
///     (&widget2.focus, widget2.area),
/// ]).handle_repaint(evt, repaint, DefaultKeys)
/// .on_changed(|| {
///     on_focus!(widget1 => {
///         // ... do something useful ...
///     });
///     on_focus!(widget2 => {
///         // ... do something else ...
///     });
///
///     ControlUI::Changed
/// });
/// ```
///
#[macro_export]
macro_rules! on_focus {
    ($field:expr => $gained:expr) => {{
        let cond = $field.focus.get();
        if cond {
            $gained;
        }
    }};
}

impl<'a> Default for Focus<'a> {
    fn default() -> Self {
        Self {
            areas: Default::default(),
            focus: Default::default(),
        }
    }
}

impl<'a> IntoIterator for Focus<'a> {
    type Item = (&'a FocusFlag, Rect);
    type IntoIter = Zip<vec::IntoIter<&'a FocusFlag>, vec::IntoIter<Rect>>;

    fn into_iter(self) -> Self::IntoIter {
        self.focus.into_iter().zip(self.areas.into_iter())
    }
}

impl<'a> Focus<'a> {
    /// Create a focus cycle.
    ///
    /// Take a reference to a FocusFlag and a Rect for mouse-events.
    pub fn new(np: impl IntoIterator<Item = (&'a FocusFlag, Rect)>) -> Self {
        Focus::default().add(np)
    }

    /// Add more to the focus cycle.
    pub fn add(mut self, np: impl IntoIterator<Item = (&'a FocusFlag, Rect)>) -> Self {
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

    /// Change the focus using the tag.
    ///
    /// Sets the focus and the lost flags. Calling this for the widget that currently
    /// has the focus returns *false*, but it resets any lost flag.
    pub fn focus(&self, tag: u16) -> bool {
        let mut change = false;

        for f in self.focus.iter() {
            f.lost.set(false);
            if f.tag.get() == tag {
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

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *false*, but it resets any lost flag.
    pub fn next(&self) -> bool {
        for (i, p) in self.focus.iter().enumerate() {
            if p.focus.get() {
                p.focus.set(false);
                let n = next_circular(i, self.focus.len());
                if i != n {
                    p.lost.set(true);
                    self.focus[n].focus.set(true);
                    return true;
                } else {
                    p.lost.set(false);
                    return false;
                }
            }
        }
        false
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *false*, but it resets any lost flag.
    pub fn prev(&self) -> bool {
        for (i, p) in self.focus.iter().enumerate() {
            if p.focus.get() {
                p.focus.set(false);
                let n = prev_circular(i, self.focus.len());
                if i != n {
                    p.lost.set(true);
                    self.focus[n].focus.set(true);
                    return true;
                } else {
                    p.lost.set(false);
                    return false;
                }
            }
        }
        false
    }
}

impl<'a, A, E> HandleCrossterm<ControlUI<A, E>> for Focus<'a> {
    fn handle(&mut self, event: &Event, repaint: &Repaint, _: DefaultKeys) -> ControlUI<A, E> {
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
            _ => return self.handle(event, repaint, MouseOnly),
        }
    }
}

impl<'a, A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for Focus<'a> {
    /// Only do mouse-events.
    fn handle(&mut self, event: &Event, repaint: &Repaint, _: MouseOnly) -> ControlUI<A, E> {
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
