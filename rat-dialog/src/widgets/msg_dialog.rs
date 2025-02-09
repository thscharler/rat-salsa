use crate::DialogState;
use rat_salsa::{AppState, AppWidget, Control};
use rat_widget::event::{Dialog, HandleEvent};
use rat_widget::layout::layout_middle;
use rat_widget::msgdialog::MsgDialogStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::StatefulWidget;

#[derive(Debug)]
pub struct MsgDialog {
    widget: rat_widget::msgdialog::MsgDialog<'static>,
}

#[derive(Debug)]
pub struct MsgDialogState {
    state: rat_widget::msgdialog::MsgDialogState,
}

impl MsgDialog {
    pub fn new() -> Self {
        Self {
            widget: Default::default(),
        }
    }

    pub fn styles(mut self, styles: MsgDialogStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }
}

impl<Global, Event, Error> AppWidget<Global, Event, Error> for MsgDialog
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
{
    type State = dyn DialogState<Global, Event, Error>;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        _ctx: &mut rat_salsa::RenderContext<'_, Global>,
    ) -> Result<(), Error> {
        let state = state.downcast_mut::<MsgDialogState>().expect("state");

        let dlg_area = layout_middle(
            area,
            Constraint::Percentage(19),
            Constraint::Percentage(19),
            Constraint::Length(4),
            Constraint::Length(4),
        );

        self.widget.clone().render(dlg_area, buf, &mut state.state);

        Ok(())
    }
}

impl MsgDialogState {
    pub fn new(msg: impl AsRef<str>) -> Self {
        let msg_dialog = rat_widget::msgdialog::MsgDialogState::default();
        msg_dialog.append(msg.as_ref());
        msg_dialog.set_active(true);
        Self { state: msg_dialog }
    }

    pub fn append(&self, msg: &str) {
        self.state.append(msg);
    }
}

impl<Global, Event, Error> AppState<Global, Event, Error> for MsgDialogState
where
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
    Error: Send + 'static,
    Event: Send + 'static,
{
    fn event(
        &mut self,
        event: &Event,
        _ctx: &mut rat_salsa::AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        let r = if let Ok(event) = event.try_into() {
            self.state.handle(event, Dialog).into()
        } else {
            Control::Continue
        };
        Ok(r)
    }
}

impl<Global, Event, Error> DialogState<Global, Event, Error> for MsgDialogState
where
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static,
    for<'a> &'a crossterm::event::Event: TryFrom<&'a Event>,
{
    fn active(&self) -> bool {
        self.state.active()
    }
}
