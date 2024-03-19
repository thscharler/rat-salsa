use crate::util::{next_circular, prev_circular};
use crate::{ControlUI, HandleCrossterm};
use crate::{DefaultKeys, MouseOnly};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
#[allow(unused_imports)]
use log::error;
use ratatui::layout::{Position, Rect};
use std::cell::Cell;
use std::iter::Zip;
use std::vec;

/// Contains flags for the focus.
///
/// This struct is used as part of the widget state.
///
/// See [HasFocus], [validate!] and also [on_gained!], [on_lost!].
///
#[derive(Debug, Clone, Default)]
pub struct FocusFlag {
    /// A unique tag within one focus-cycle. It is set when the focus cycle is created.
    /// See [Focus::focus]
    pub tag: Cell<u16>,
    /// Focus. See [on_focus!](crate::on_focus!())
    pub focus: Cell<bool>,
    /// This widget just gained the focus.
    /// It is reset at the beginning of all handle_xxx() calls.
    pub gained: Cell<bool>,
    /// This widget just lost the focus. See [validate!](crate::validate!())
    /// It is reset at the beginning of all handle_xxx() calls.
    pub lost: Cell<bool>,
}

/// Trait for a widget that has a focus flag.
pub trait HasFocus {
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

    /// Focus cycle tag.
    fn focus_tag(&self) -> u16 {
        self.focus().tag()
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
/// Focus::new([
///     (&widget1.focus, widget1.area),
///     (&widget2.focus, widget2.area),
/// ]).handle(evt, DefaultKeys)
/// .and_do(|_| uistate.repaint.set());
/// ```
///
/// repaint in the example is a [Repaint]. This is necessary as the focus change does not
/// automatically trigger a repaint.
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

    /// Just gained the focus.
    #[inline]
    pub fn gained(&self) -> bool {
        self.gained.get()
    }
}

/// Executes the block if [HasFocus::lost_focus()] is true.
#[macro_export]
macro_rules! on_lost {
    ($field:expr => $validate:expr) => {{
        use $crate::HasFocus;
        let cond = $field.lost_focus();
        if cond {
            $validate;
        }
    }};
}

/// Executes the block if [HasFocus::gained_focus()] is true.
#[macro_export]
macro_rules! on_gained {
    ($field:expr => $gained:expr) => {{
        use $crate::HasFocus;
        let cond = $field.gained_focus();
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

    /// Sets the focus to the widget with `tag`.
    ///
    /// Sets focus and gained but not lost. This can be used to prevent validation of the field.
    pub fn focus_no_lost(&self, tag: u16) {
        self.start_focus_change(false);
        if let Some(f) = self.focus.iter().find(|f| f.tag.get() == tag) {
            f.focus.set(true);
            f.gained.set(true);
        }
    }

    /// Sets the focus to the widget with `tag`.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    pub fn focus(&self, tag: u16) {
        self.start_focus_change(true);
        if let Some(f) = self.focus.iter().find(|f| f.tag.get() == tag) {
            f.focus.set(true);
            f.gained.set(true);
        }
    }

    /// Reset lost + gained flags.
    ///
    /// This is done automatically in `HandleCrossterm::handle()` for every event.
    /// This means these flags are only ever set if `handle()` returns Run(true) to allow
    /// immediate reactions to focus changes.
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

impl<'a> HandleCrossterm<ControlUI<bool, ()>> for Focus<'a> {
    /// Handle events. This is somewhat special as it doesn't blend into the
    /// Action/Error types of the application, but returns its own action on
    /// focus change.
    ///
    /// The idea is to react to the change, but not to cancel further processing of
    /// the event. This is crucial for handling mouse-events, otherwise the first
    /// click would focus, but do nothing otherwise. e.g. select a row in a table.
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<bool, ()> {
        use crossterm::event::*;
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.next() {
                    ControlUI::Run(true)
                } else {
                    ControlUI::Continue
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.prev() {
                    ControlUI::Run(true)
                } else {
                    ControlUI::Continue
                }
            }
            _ => self.handle(event, MouseOnly),
        }
    }
}

impl<'a> HandleCrossterm<ControlUI<bool, ()>, MouseOnly> for Focus<'a> {
    /// Only do mouse-events.
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<bool, ()> {
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
            ) => 'f: {
                for (idx, area) in self.areas.iter().enumerate() {
                    if area.contains(Position::new(*column, *row)) {
                        self.focus_idx(idx);
                        break 'f ControlUI::Run(true);
                    }
                }
                self.reset_lost_gained();
                ControlUI::Continue
            }
            _ => {
                self.reset_lost_gained();
                ControlUI::Continue
            }
        }
    }
}
