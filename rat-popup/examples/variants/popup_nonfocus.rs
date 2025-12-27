//! Popup doesn't interact with focus.

use crate::variants::calc_dxy;
use rat_event::{HandleEvent, Popup, Regular};
use rat_popup::event::PopupOutcome;
use rat_popup::{PopupConstraint, PopupCore, PopupCoreState};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::text::Span;
use ratatui_core::widgets::{StatefulWidget, Widget};
use ratatui_crossterm::crossterm::event::Event;
use ratatui_widgets::block::Block;
use ratatui_widgets::borders::BorderType;

#[derive(Debug, Default)]
pub struct PopNonFocusRed;

impl StatefulWidget for PopNonFocusRed {
    type State = PopNonFocusRedState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.popup.is_active() {
            PopupCore::new()
                .constraint(state.placement)
                .offset(calc_dxy(state.placement, 1))
                .render(area, buf, &mut state.popup);

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .style(Style::new().black().on_cyan());
            let widget_area = block.inner(area);
            block.render(area, buf);

            buf.set_style(widget_area, Style::new().black().on_cyan());
            Span::from("*** ***").render(widget_area, buf);
        }
    }
}

#[derive(Debug, Default)]
pub struct PopNonFocusRedState {
    /// Where to place the popup
    pub placement: PopupConstraint,
    /// Internalized popup state.
    pub popup: PopupCoreState,
}

impl PopNonFocusRedState {
    pub fn new() -> Self {
        Self {
            placement: Default::default(),
            popup: Default::default(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.popup.is_active()
    }

    pub fn show(&mut self, placement: PopupConstraint) {
        self.placement = placement;
        // only use the active flag.
        self.popup.set_active(true);
    }

    pub fn hide(&mut self) {
        // only use the active flag.
        self.popup.set_active(false);
    }
}

impl HandleEvent<Event, Regular, PopupOutcome> for PopNonFocusRedState {
    fn handle(&mut self, event: &Event, _qualifier: Regular) -> PopupOutcome {
        let r0 = self.popup.handle(event, Popup);
        // questionable whether we should do anything else here
        // as this widget doesn't have the focus
        r0
    }
}
