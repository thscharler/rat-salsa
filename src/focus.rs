/// Keeps track of the focus.
///
/// This works by adding a FocusFlag to the State of a widget.
/// Focus is constructed with a list of references to these flags
/// and switches the focus that way.
///
/// Each widget stays separate otherwise and can pull its focus state
/// from this struct.
///
/// There is one additional flag [FocusFlag::validate] which is set if a widget
/// looses the focus. This can be used as a validation marker, but is otherwise not used.
/// There is a macro [validate!] which can be used for this. It evaluates a block and
/// stores the result in [FocusFlag::is_valid] which can be used by the widget.
///
use crate::util::{next_circular, prev_circular};
use crate::widget::{DefaultKeys, HandleCrossterm, MouseOnly, Repaint};
use crate::ControlUI;
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
    /// A unique tag within one focus-cycle. This value is set when
    /// the focus cycle is created. While it is not recommended to change
    /// this value it's not essential to the operation. It's only used
    /// to change the focus from the outside.
    pub tag: Cell<u16>,
    /// Active focus flag. There is usually only one widget with focus==true
    /// within a cycle.
    pub focus: Cell<bool>,
    /// Indicates that the field just lost the focus.
    pub lost: Cell<bool>,
}

/// Switch the focus for an ui.
///
/// Uses a list of [FocusFlag] for its operation. That way each widget can
/// stay independent.
#[derive(Debug)]
pub struct Focus<'a> {
    pub areas: Vec<Rect>,
    pub focus: Vec<&'a FocusFlag>,
}

impl Default for FocusFlag {
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
    pub fn get(&self) -> bool {
        self.focus.get()
    }

    /// Set the focus.
    pub fn set(&self) {
        self.focus.set(true);
    }

    /// Associated tag.
    pub fn tag(&self) -> u16 {
        self.tag.get()
    }

    /// Just lost the focus.
    pub fn lost(&self) -> bool {
        self.lost.get()
    }
}

/// Validates the given widget.
///
/// It expects that the widget has the fields `focus: FocusFlag` and `is_valid: bool` that
/// are both public.
///
/// If `focus.lost()` is set, the expression is evaluated. The result is set into `is_valid`.
#[macro_export]
macro_rules! validate {
    ($field:expr => $validate:expr) => {{
        let cond = $field.focus.lost();
        if cond {
            let valid = $validate;
            $field.is_valid = valid;
        }
    }};
}

/// Focus gained.
/// Might be replaced with some fn on_focus(..) on the state struct.
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

    /// Resets the focus to the last field that lost the focus.
    /// Can be used to reset the focus after a failed validation without triggering another one.
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

    /// Change the focused part. Uses an index into the list.
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

    /// Change the focused part. Used for focus changes unrelated to standard
    /// navigation.
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
