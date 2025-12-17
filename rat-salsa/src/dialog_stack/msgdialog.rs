#[cfg(feature = "crossterm")]
use crate::Control;
use crate::SalsaContext;
#[cfg(feature = "crossterm")]
use rat_event::{Dialog, HandleEvent, Outcome, try_flow};
use rat_widget::layout::LayoutOuter;
use rat_widget::msgdialog::{MsgDialog, MsgDialogState, MsgDialogStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use std::any::Any;
#[cfg(feature = "crossterm")]
use try_as_traits::TryAsRef;

/// Create a render-fn for MsgDialog to be used with DialogStack.
#[allow(unused_variables)]
pub fn msg_dialog_render<Event, Error, Context: SalsaContext<Event, Error>>(
    layout: LayoutOuter,
    style: MsgDialogStyle,
) -> impl Fn(Rect, &mut Buffer, &mut dyn Any, &mut Context)
where
    Event: 'static,
    Error: 'static,
{
    move |area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Context| {
        let state = state
            .downcast_mut::<MsgDialogState>()
            .expect("dialog-state");

        let area = layout.layout(area);
        MsgDialog::new()
            .styles(style.clone())
            .render(area, buf, state);
    }
}

/// Create an event-fn for MsgDialog to be used with DialogStack.
#[cfg(feature = "crossterm")]
#[allow(unused_variables)]
pub fn msg_dialog_event<Event, Error, Context: SalsaContext<Event, Error>>(
    map: impl Fn() -> Event,
) -> impl Fn(&Event, &mut dyn Any, &mut Context) -> Result<Control<Event>, Error>
where
    Event: TryAsRef<crossterm::event::Event> + 'static,
    Error: 'static,
{
    move |event: &Event, state: &mut dyn Any, ctx: &mut Context| -> Result<Control<Event>, Error> {
        let state = state
            .downcast_mut::<MsgDialogState>()
            .expect("dialog-state");

        if let Some(event) = event.try_as_ref() {
            try_flow!(match state.handle(event, Dialog) {
                Outcome::Changed => {
                    if !state.active() {
                        Control::Close(map())
                    } else {
                        Control::Changed
                    }
                }
                r => r.into(),
            });
        }

        Ok(Control::Continue)
    }
}
