/// Keeps track of the focus.
///
/// This works by adding a FocusFlag to the State of a widget.
/// Focus is constructed with a list of references to these flags
/// and switches the focus that way.
///
/// Each widget stays separate otherwise and can pull its focus state
/// from this struct.
///
use crate::util::{next_circular, prev_circular};
use crate::ControlUI;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};
use std::cell::Cell;

/// Flag structure to be used in components.
#[derive(Debug, Clone)]
pub struct FocusFlag {
    /// A unique tag within one focus-cycle. This value is set when
    /// the focus cycle is created. While it is not recommended to change
    /// this value it's not essential to the operation. It's only used
    /// to change the focus from the outside.
    pub tag: Cell<u16>,
    /// Active focus flag. There is usually only one component with focus==true
    /// within a cycle.
    pub focus: Cell<bool>,
    /// Does this widget require validation
    pub validate: Cell<bool>,
    /// Is the widget content valid
    pub is_valid: Cell<bool>,
}

/// Switch the focus for an ui.
///
/// Uses a list of &FocusFlag for its operation. That way each widget can
/// stay independent.
#[derive(Debug)]
pub struct Focus<'a> {
    areas: Vec<Rect>,
    focus: Vec<&'a FocusFlag>,
}

/// Result of event processing.
#[derive(Debug)]
pub enum FocusChanged {
    Changed,
    Continue,
}

impl FocusChanged {
    /// Convert to ControlUI.
    pub fn into_control<A, E>(self) -> ControlUI<A, E> {
        self.into()
    }
}

impl<A, E> From<Option<FocusChanged>> for ControlUI<A, E> {
    fn from(value: Option<FocusChanged>) -> Self {
        match value {
            None => ControlUI::Continue,
            Some(FocusChanged::Changed) => ControlUI::Changed,
            Some(FocusChanged::Continue) => ControlUI::Continue,
        }
    }
}

impl<A, E> From<FocusChanged> for ControlUI<A, E> {
    fn from(value: FocusChanged) -> Self {
        match value {
            FocusChanged::Changed => ControlUI::Changed,
            FocusChanged::Continue => ControlUI::Continue,
        }
    }
}

impl Default for FocusFlag {
    fn default() -> Self {
        Self {
            tag: Cell::new(0),
            focus: Cell::new(false),
            validate: Cell::new(false),
            is_valid: Cell::new(true),
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

    /// Needs validation. Resets the flag.
    pub fn needs_validation(&self) -> bool {
        self.validate.replace(false)
    }

    /// Is valid
    pub fn is_valid(&self) -> bool {
        self.is_valid.get()
    }

    // Is invalid
    pub fn is_invalid(&self) -> bool {
        !self.is_valid.get()
    }

    // Set valid state.
    pub fn set_valid(&self) {
        self.is_valid.set(true);
    }

    pub fn set_invalid(&self) {
        self.is_valid.set(false);
    }
}

#[macro_export]
macro_rules! validate {
    ($x:expr => $v:expr) => {
        let cond = $x.focus.needs_validation();
        if cond {
            let valid = $v;
            $x.focus.is_valid.set(valid);
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

    /// Reset the focus
    pub fn reset(&self, tag: u16) {
        for f in self.focus.iter() {
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
    pub fn focus<A, E>(&self, tag: u16) -> FocusChanged {
        let mut change = FocusChanged::Continue;

        for f in self.focus.iter() {
            if f.tag.get() == tag {
                if !f.focus.get() {
                    change = FocusChanged::Changed;
                    f.focus.set(true);
                }
            } else {
                if f.focus.get() {
                    f.validate.set(true);
                    f.focus.set(false);
                }
            }
        }

        change
    }

    /// Handle events.
    pub fn handle(&mut self, event: &Event) -> FocusChanged {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                for (i, p) in self.focus.iter().enumerate() {
                    if p.focus.get() {
                        p.validate.set(true);
                        p.focus.set(false);
                        let n = next_circular(i, self.focus.len());
                        self.focus[n].focus.set(true);
                        break;
                    }
                }
                FocusChanged::Changed
            }
            Event::Key(KeyEvent {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                for (i, p) in self.focus.iter().enumerate() {
                    if p.focus.get() {
                        p.validate.set(true);
                        p.focus.set(false);
                        let n = prev_circular(i, self.focus.len());
                        self.focus[n].focus.set(true);
                        break;
                    }
                }
                FocusChanged::Changed
            }

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
                for (idx, r) in self.areas.iter().enumerate() {
                    if r.contains(Position::new(*column, *row)) && !self.focus[idx].focus.get() {
                        for p in self.focus.iter() {
                            if p.focus.get() {
                                p.validate.set(true);
                                p.focus.set(false);
                            }
                        }
                        self.focus[idx].focus.set(true);
                        break 'f FocusChanged::Changed;
                    }
                }
                FocusChanged::Continue
            }

            _ => FocusChanged::Continue,
        }
    }
}
