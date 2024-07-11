use crate::event::Outcome;
use crate::{FTableState, TableSelection};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use rat_focus::HasFocusFlag;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::ScrollArea;
use std::cmp::max;

/// Doesn't do any selection for the table.
///
/// But it implements scrolling via mouse and keyboard.
#[derive(Debug, Default, Clone)]
pub struct NoSelection;

impl TableSelection for NoSelection {
    fn is_selected_row(&self, _row: usize) -> bool {
        false
    }

    fn is_selected_column(&self, _column: usize) -> bool {
        false
    }

    fn is_selected_cell(&self, _column: usize, _row: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<(usize, usize)> {
        None
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for FTableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.scroll_up(1).into(),
                ct_event!(keycode press Down) => self.scroll_down(1).into(),
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => self.scroll_to_row(0).into(),
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => {
                    self.scroll_to_row(self.rows.saturating_sub(1)).into()
                }

                ct_event!(keycode press PageUp) => self
                    .scroll_up(max(1, self.page_len().saturating_sub(1)))
                    .into(),
                ct_event!(keycode press PageDown) => self
                    .scroll_down(max(1, self.page_len().saturating_sub(1)))
                    .into(),

                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press CONTROL-Left) => self.scroll_to_x(0).into(),
                ct_event!(keycode press CONTROL-Right) => {
                    self.scroll_to_x(self.x_max_offset()).into()
                }
                _ => Outcome::NotUsed,
            }
        } else {
            Outcome::NotUsed
        };

        if res == Outcome::NotUsed {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for FTableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        let r = match ScrollArea(self.inner, Some(&mut self.hscroll), Some(&mut self.vscroll))
            .handle(event, MouseOnly)
        {
            ScrollOutcome::Up(v) => self.scroll_up(v),
            ScrollOutcome::Down(v) => self.scroll_down(v),
            ScrollOutcome::VPos(v) => self.set_row_offset(v),
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::HPos(v) => self.set_x_offset(v),

            ScrollOutcome::NotUsed => false,
            ScrollOutcome::Unchanged => false,
            ScrollOutcome::Changed => true,
        };
        if r {
            return Outcome::Changed;
        }

        Outcome::Unchanged
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut FTableState<NoSelection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    state.focus.set(focus);
    state.handle(event, FocusKeys)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut FTableState<NoSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}
