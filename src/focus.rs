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
use crate::widget::{DefaultKeys, HandleCrossterm, Input, MouseOnly};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::error;
use ratatui::layout::{Position, Rect};
use std::cell::Cell;

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

/// Action.
#[derive(Debug)]
pub enum InputRequest {
    Next,
    Prev,
    Tag(u16),
}

/// Result of event processing.
#[derive(Debug)]
pub enum FocusChanged {
    Changed,
    Continue,
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
    ($field:expr => $validate:expr) => {
        let cond = $field.focus.lost();
        if cond {
            let valid = $validate;
            $field.is_valid = valid;
        }
    };
}

impl<'a> Focus<'a> {
    /// Create a focus cycle.
    ///
    /// Take a reference to a FocusFlag and a Rect for mouse-events.
    pub fn new<const N: usize>(np: [(&'a FocusFlag, Rect); N]) -> Self {
        let mut zelf = Focus {
            areas: Vec::new(),
            focus: Vec::new(),
        };

        for (n, (f, rect)) in np.into_iter().enumerate() {
            f.tag.set(n as u16);
            zelf.focus.push(f);
            zelf.areas.push(rect);
        }

        zelf
    }

    /// Resets the focus to the given field and clears all lost values.
    /// Can be used to reset the focus after a failed validation without triggering another one.
    pub fn reset(&self, tag: u16) {
        for f in self.focus.iter() {
            f.lost.set(false);
            if f.tag.get() == tag {
                if !f.focus.get() {
                    f.focus.set(true);
                }
            } else {
                if f.focus.get() {
                    f.focus.set(false);
                }
            }
        }
    }

    /// Change the focused part. Used for focus changes unrelated to standard
    /// navigation.
    pub fn focus(&self, tag: u16) -> FocusChanged {
        let mut change = FocusChanged::Continue;

        for f in self.focus.iter() {
            f.lost.set(false);
            if f.tag.get() == tag {
                if !f.focus.get() {
                    change = FocusChanged::Changed;
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
    pub fn next(&self) {
        for (i, p) in self.focus.iter().enumerate() {
            p.lost.set(false);
            if p.focus.get() {
                p.lost.set(true);
                p.focus.set(false);
                let n = next_circular(i, self.focus.len());
                self.focus[n].focus.set(true);
                break;
            }
        }
    }

    /// Focus the previous widget in the cycle.
    pub fn prev(&self) {
        for (i, p) in self.focus.iter().enumerate() {
            p.lost.set(false);
            if p.focus.get() {
                p.lost.set(true);
                p.focus.set(false);
                let n = prev_circular(i, self.focus.len());
                self.focus[n].focus.set(true);
                break;
            }
        }
    }
}

impl<'a> Input<FocusChanged> for Focus<'a> {
    type Request = InputRequest;

    fn perform(&mut self, action: Self::Request) -> FocusChanged {
        match action {
            InputRequest::Next => {
                self.next();
                FocusChanged::Changed
            }
            InputRequest::Prev => {
                self.prev();
                FocusChanged::Changed
            }
            InputRequest::Tag(tag) => self.focus(tag),
        }
    }
}

impl<'a> HandleCrossterm<FocusChanged> for Focus<'a> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> FocusChanged {
        use crossterm::event::*;

        let req = match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => Some(InputRequest::Next),
            Event::Key(KeyEvent {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => Some(InputRequest::Prev),
            _ => return self.handle(event, MouseOnly),
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            FocusChanged::Continue
        }
    }
}

impl<'a> HandleCrossterm<FocusChanged, MouseOnly> for Focus<'a> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> FocusChanged {
        use crossterm::event::*;

        let req = match event {
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
            ) => 'f: {
                for (idx, area) in self.areas.iter().enumerate() {
                    if area.contains(Position::new(*column, *row)) {
                        break 'f Some(InputRequest::Tag(self.focus[idx].tag()));
                    }
                }
                None
            }
            _ => None,
        };

        if let Some(req) = req {
            self.perform(req)
        } else {
            FocusChanged::Continue
        }
    }
}
