use crossterm::event::Event;
use rat_theme4::WidgetStyle;
use rat_theme4::theme::SalsaTheme;
use rat_widget::dialog_frame::{DialogFrame, DialogFrameState, DialogOutcome};
use rat_widget::event::{Dialog, HandleEvent, Outcome, Regular, ct_event, event_flow};
use rat_widget::focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_widget::reloc::RelocatableState;
use rat_widget::text::HasScreenCursor;
use rat_widget::text_input::{TextInput, TextInputState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::widgets::Block;
use std::cmp::min;

#[derive(Debug)]
pub struct SampleDialog<'a> {
    theme: &'a SalsaTheme,
}

#[derive(Debug, Default)]
pub struct SampleDialogState {
    pub frame: DialogFrameState,
    pub input: TextInputState,
    pub container: FocusFlag,
}

impl<'a> SampleDialog<'a> {
    pub fn new(theme: &'a SalsaTheme) -> Self {
        Self { theme }
    }
}

impl<'a> StatefulWidget for SampleDialog<'a> {
    type State = SampleDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        DialogFrame::new()
            .left(Constraint::Length(3))
            .right(Constraint::Length(3))
            .top(Constraint::Length(3))
            .bottom(Constraint::Length(3))
            .block(Block::bordered().title("Dialog"))
            .styles(self.theme.style(WidgetStyle::DIALOG_FRAME))
            .render(area, buf, &mut state.frame);

        let txt_area = Rect::new(
            state.frame.widget_area.x + 1,
            state.frame.widget_area.y + 1,
            min(state.frame.widget_area.width.saturating_sub(1), 20),
            1,
        );

        TextInput::new()
            .styles(self.theme.style(WidgetStyle::TEXT))
            .render(txt_area, buf, &mut state.input);
    }
}

impl HasFocus for SampleDialogState {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        builder.widget(&self.input);
        builder.widget(&self.frame);
        builder.end(tag);
    }

    fn focus(&self) -> FocusFlag {
        self.container.focus()
    }

    fn area(&self) -> Rect {
        self.frame.area
    }
}

impl HasScreenCursor for SampleDialogState {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.input.screen_cursor()
    }
}

impl RelocatableState for SampleDialogState {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.input.relocate(shift, clip);
        self.frame.relocate(shift, clip);
    }
}

impl HandleEvent<Event, Regular, Outcome> for SampleDialogState {
    fn handle(&mut self, event: &Event, _qualifier: Regular) -> Outcome {
        event_flow!(return Outcome::from(self.input.handle(event, Regular)));

        // don't want the dialog to eat these keys.
        if !matches!(
            event,
            ct_event!(keycode press Esc)
                | ct_event!(keycode press Enter)
                | ct_event!(keycode press F(12))
        ) {
            event_flow!(
                return match self.frame.handle(event, Dialog) {
                    DialogOutcome::Unchanged => {
                        // don't block event-loop
                        Outcome::Continue
                    }
                    r => {
                        r.into()
                    }
                }
            );
        }
        Outcome::Continue
    }
}
