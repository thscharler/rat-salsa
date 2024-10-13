/// Popup acts as a container. Has its own focus cycle.
use crate::adapter::textinputf::{TextInputF, TextInputFState};
use crate::mini_salsa::theme::THEME;
use crate::variants::calc_dxy;
use rat_cursor::HasScreenCursor;
use rat_event::{ct_event, HandleEvent, Outcome, Popup, Regular};
use rat_focus::{
    ContainerAdapter, Focus, FocusBuilder, FocusFlag, HasFocus, HasFocusFlag, Navigation,
};
use rat_popup::event::PopupOutcome;
use rat_popup::{Placement, PopupCore, PopupCoreState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, BorderType};
use std::cmp::max;

#[derive(Debug, Default)]
pub struct PopLockMagenta;

#[derive(Debug)]
pub struct PopLockMagentaState {
    pub outer_focus: FocusFlag,

    /// Where to place the popup
    pub placement: Placement,
    /// Internalized popup state.
    pub popup: PopupCoreState,

    pub edit1: TextInputFState,
    pub edit2: TextInputFState,
    pub edit3: TextInputFState,
}

impl StatefulWidget for PopLockMagenta {
    type State = PopLockMagentaState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.popup.is_active() {
            PopupCore::new()
                .placement(state.placement)
                .offset(calc_dxy(state.placement, 1))
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .style(Style::new().black().on_dark_gray()),
                )
                .render(area, buf, &mut state.popup);

            let mut a1 = state.popup.widget_area;
            a1.height = 1;
            TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus())
                .render(a1, buf, &mut state.edit1);

            let mut a2 = state.popup.widget_area;
            a2.y += 1;
            a2.height = 1;
            TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus())
                .render(a2, buf, &mut state.edit2);

            let mut a3 = state.popup.widget_area;
            a3.y += 2;
            a3.height = 1;
            TextInputF::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_input_focus())
                .render(a3, buf, &mut state.edit3);
        }
    }
}

impl Default for PopLockMagentaState {
    fn default() -> Self {
        Self {
            outer_focus: Default::default(),
            placement: Default::default(),
            popup: Default::default(),
            edit1: TextInputFState::named("edit1"),
            edit2: TextInputFState::named("edit2"),
            edit3: TextInputFState::named("edit3"),
        }
    }
}

impl HasScreenCursor for PopLockMagentaState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.edit1
            .screen_cursor()
            .or_else(|| self.edit2.screen_cursor())
            .or_else(|| self.edit3.screen_cursor())
    }
}

impl HasFocusFlag for PopLockMagentaState {
    fn focus(&self) -> FocusFlag {
        self.outer_focus.clone()
    }

    fn area(&self) -> Rect {
        self.popup.area
    }

    fn navigable(&self) -> Navigation {
        // don't give away the focus as long as we are active.
        if self.is_active() {
            Navigation::Lock
        } else {
            Navigation::None
        }
    }
}

impl PopLockMagentaState {
    pub fn new() -> Self {
        Self {
            outer_focus: Default::default(),
            placement: Default::default(),
            popup: PopupCoreState::named("act-popup"),
            edit1: Default::default(),
            edit2: Default::default(),
            edit3: Default::default(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.popup.is_active()
    }

    pub fn show(&mut self, placement: Placement, focus: &mut Focus) {
        self.placement = placement;
        // set outer focus and active
        self.popup.set_active(true);
        focus.focus(&*self);
        // set inner focus
        self.inner_focus().first();
    }

    pub fn hide(&mut self, focus: &mut Focus) {
        // set outer focus and active
        self.popup.set_active(false);
        focus.expel_focus(&*self);
        // clear inner focus
        self.inner_focus().none();
    }

    fn inner_focus(&mut self) -> Focus {
        let mut fb = FocusBuilder::new(None);
        // create on-the-fly container
        fb.container(&ContainerAdapter {
            container: self.popup.active.clone(),
            build_fn: &|build| {
                build.widget(&self.edit1);
                build.widget(&self.edit2);
                build.widget(&self.edit3);
            },
            area: self.popup.area,
        });
        fb.build()
    }
}

impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopLockMagentaState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
        let r0 = match self.popup.handle(event, Popup) {
            // don't auto hide
            PopupOutcome::Hide => PopupOutcome::Continue,
            r => r,
        };

        if self.is_active() {
            // handle inner focus
            let f = match self.inner_focus().handle(event, Regular) {
                Outcome::Continue => PopupOutcome::Continue,
                Outcome::Unchanged => PopupOutcome::Unchanged,
                Outcome::Changed => PopupOutcome::Changed,
            };

            let r1 = match event {
                // hide on esc
                ct_event!(keycode press Esc) => PopupOutcome::Hide,
                _ => PopupOutcome::Continue,
            };

            max(f, max(r0, r1))
        } else {
            r0
        }
    }
}
