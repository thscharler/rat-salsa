//! Popup acts as a container, and takes part of the focus.
//! Hides when loosing focus.

use crate::mini_salsa::text_input_mock::{TextInputMock, TextInputMockState, textinput_mock_style};
use crate::variants::calc_dxy;
use rat_cursor::HasScreenCursor;
use rat_event::{HandleEvent, Popup, Regular};
use rat_focus::{Focus, FocusBuilder, FocusFlag, HasFocus};
use rat_popup::event::PopupOutcome;
use rat_popup::{PopupConstraint, PopupCore, PopupCoreState};
use rat_theme4::StyleName;
use rat_theme4::theme::SalsaTheme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType};
use ratatui::widgets::{StatefulWidget, Widget};

#[derive(Debug)]
pub struct PopEditGreen<'a> {
    theme: &'a SalsaTheme,
}

#[derive(Debug)]
pub struct PopEditGreenState {
    /// Where to place the popup
    pub placement: PopupConstraint,
    /// Internalized popup state.
    pub popup: PopupCoreState,

    pub edit1: TextInputMockState,
    pub edit2: TextInputMockState,
    pub edit3: TextInputMockState,

    pub container: FocusFlag,
}

impl<'a> PopEditGreen<'a> {
    pub fn new(theme: &'a SalsaTheme) -> Self {
        Self { theme }
    }
}

impl StatefulWidget for PopEditGreen<'_> {
    type State = PopEditGreenState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.popup.is_active() {
            PopupCore::new()
                .constraint(state.placement)
                .offset(calc_dxy(state.placement, 1))
                .render(area, buf, &mut state.popup);

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .style(self.theme.style_style(Style::CONTAINER_BORDER_FG));
            let widget_area = block.inner(area);
            block.render(area, buf);

            let mut a1 = widget_area;
            a1.height = 1;
            TextInputMock::default()
                .styles(textinput_mock_style(self.theme))
                .render(a1, buf, &mut state.edit1);

            let mut a2 = widget_area;
            a2.y += 1;
            a2.height = 1;
            TextInputMock::default()
                .styles(textinput_mock_style(self.theme))
                .render(a2, buf, &mut state.edit2);

            let mut a3 = widget_area;
            a3.y += 2;
            a3.height = 1;
            TextInputMock::default()
                .styles(textinput_mock_style(self.theme))
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
            container: FocusFlag::new(),
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

impl HasFocus for PopEditGreenState {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        // only has widgets when active.
        if self.popup.is_active() {
            builder.widget(&self.edit1);
            builder.widget(&self.edit2);
            builder.widget(&self.edit3);
        }
        builder.end(tag);
    }

    fn focus(&self) -> FocusFlag {
        self.container.clone()
    }

    fn area(&self) -> Rect {
        Rect::default()
    }
}

impl PopEditGreenState {
    pub fn new() -> Self {
        Self {
            placement: Default::default(),
            popup: Default::default(),
            edit1: Default::default(),
            edit2: Default::default(),
            edit3: Default::default(),
            container: Default::default(),
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
        focus.focus(&*self);
    }

    pub fn hide(&mut self, focus: &mut Focus) {
        // move away focus, set inactive and update Focus.
        if self.popup.is_active() {
            focus.expel_focus(&*self);
        }
        self.popup.set_active(false);
        focus.update_container(&*self);
    }
}

impl HandleEvent<crossterm::event::Event, Regular, PopupOutcome> for PopEditGreenState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> PopupOutcome {
        if self.container.lost_focus() {
            self.popup.set_active(false);
        }
        let r0 = self.popup.handle(event, Popup);
        r0
    }
}
