//! Popup acts as a container, and takes part of the focus.
//! Hides when loosing focus.

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState};
use crate::mini_salsa::theme::THEME;
use crate::variants::calc_dxy;
use rat_cursor::HasScreenCursor;
use rat_event::{HandleEvent, Popup, Regular};
use rat_focus::{ContainerFlag, Focus, FocusBuilder, FocusContainer};
use rat_popup::event::PopupOutcome;
use rat_popup::{PopupConstraint, PopupCore, PopupCoreState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use ratatui::widgets::{Block, BorderType};

#[derive(Debug, Default)]
pub struct PopEditGreen;

#[derive(Debug)]
pub struct PopEditGreenState {
    /// Where to place the popup
    pub placement: PopupConstraint,
    /// Internalized popup state.
    pub popup: PopupCoreState,

    pub edit1: TextInputMockState,
    pub edit2: TextInputMockState,
    pub edit3: TextInputMockState,
}

impl StatefulWidget for PopEditGreen {
    type State = PopEditGreenState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.popup.is_active() {
            PopupCore::new()
                .constraint(state.placement)
                .offset(calc_dxy(state.placement, 1))
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .style(THEME.text_input()),
                )
                .render(area, buf, &mut state.popup);

            let mut a1 = state.popup.widget_area;
            a1.height = 1;
            TextInputMock::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_focus())
                .render(a1, buf, &mut state.edit1);

            let mut a2 = state.popup.widget_area;
            a2.y += 1;
            a2.height = 1;
            TextInputMock::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_focus())
                .render(a2, buf, &mut state.edit2);

            let mut a3 = state.popup.widget_area;
            a3.y += 2;
            a3.height = 1;
            TextInputMock::default()
                .style(THEME.text_input())
                .focus_style(THEME.text_focus())
                .render(a3, buf, &mut state.edit3);
        }
    }
}

impl Default for PopEditGreenState {
    fn default() -> Self {
        Self {
            placement: Default::default(),
            popup: Default::default(),
            edit1: TextInputMockState::new(),
            edit2: TextInputMockState::new(),
            edit3: TextInputMockState::new(),
        }
    }
}

impl HasScreenCursor for PopEditGreenState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.edit1
            .screen_cursor()
            .or_else(|| self.edit2.screen_cursor())
            .or_else(|| self.edit3.screen_cursor())
    }
}

impl FocusContainer for PopEditGreenState {
    fn build(&self, builder: &mut FocusBuilder) {
        // only has widgets when active.
        if self.popup.is_active() {
            builder.widget(&self.edit1);
            builder.widget(&self.edit2);
            builder.widget(&self.edit3);
        }
    }

    fn container(&self) -> Option<ContainerFlag> {
        // use active as container flag.
        // will be set false when the container looses focus.
        Some(self.popup.active.clone())
    }
}

impl PopEditGreenState {
    pub fn new() -> Self {
        Self {
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

    pub fn show(&mut self, placement: PopupConstraint, focus: &mut Focus) {
        self.placement = placement;
        // set active, update Focus and focus first widget.
        self.popup.set_active(true);
        focus.update_container(&*self);
        focus.first_container(&*self);
    }

    pub fn hide(&mut self, focus: &mut Focus) {
        // move away focus, set inactive and update Focus.
        if self.popup.is_active() {
            focus.expel_focus_container(&*self);
        }
        self.popup.set_active(false);
        focus.update_container(&*self);
    }
}

impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopEditGreenState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
        let r0 = self.popup.handle(event, Popup);
        r0
    }
}
