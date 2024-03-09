use crate::tui::libui::util::{next_circular, prev_circular};
use crate::tui::libui::ControlUI;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};
use std::cell::Cell;

/// Flag structure to be used in components.
#[derive(Debug, Default, Clone)]
pub struct FocusFlag {
    pub tag: Cell<u16>,
    pub focus: Cell<bool>,
}

// Used internally
#[derive(Debug)]
struct FocusFlagRef<'a> {
    pub tag: &'a Cell<u16>,
    pub focus: &'a Cell<bool>,
}

/// Switch the focused part.
///
/// The thought here is to represent the focus as multiple bool-flags as part of the ui-state.
/// Focus collects &mut references to these bools and switches between them.
///
#[derive(Debug)]
pub struct Focus<'a> {
    areas: Vec<Rect>,
    focus: Vec<FocusFlagRef<'a>>,
}

#[derive(Debug)]
pub enum FocusChanged {
    Changed,
    Continue,
}

impl FocusChanged {
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

impl FocusFlag {
    pub fn get(&self) -> bool {
        self.focus.get()
    }

    pub fn set(&self) {
        self.focus.set(true);
    }

    pub fn tag(&self) -> u16 {
        self.tag.get()
    }
}

impl<'a> Focus<'a> {
    ///
    pub fn new<const N: usize>(np: [(&'a FocusFlag, Rect); N]) -> Self {
        let mut zelf = Focus {
            areas: Vec::new(),
            focus: Vec::new(),
        };

        for (n, (flags, rect)) in np.into_iter().enumerate() {
            flags.tag.set(n as u16);
            zelf.focus.push(FocusFlagRef {
                tag: &flags.tag,
                focus: &flags.focus,
            });
            zelf.areas.push(rect);
        }

        zelf
    }

    /// Change the focused part. Used for focus changes unrelated to standard
    /// navigation.
    pub fn focus<A, E>(&mut self, tag: u16) -> FocusChanged {
        let mut change = FocusChanged::Continue;

        for f in self.focus.iter() {
            if f.tag.get() == tag {
                if !f.focus.get() {
                    change = FocusChanged::Changed;
                    f.focus.set(true);
                }
            } else {
                f.focus.set(false);
            }
        }

        change
    }

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
                        let n = next_circular(i, self.focus.len());
                        self.focus[i].focus.set(false);
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
                        let n = prev_circular(i, self.focus.len());
                        self.focus[i].focus.set(false);
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
                            p.focus.set(false);
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
