#[cfg(feature = "crossterm")]
use crate::Control;
use crate::SalsaContext;
#[cfg(feature = "crossterm")]
use rat_event::{Dialog, HandleEvent, try_flow};
#[cfg(feature = "crossterm")]
use rat_widget::event::FileOutcome;
use rat_widget::file_dialog::{FileDialog, FileDialogState, FileDialogStyle};
use rat_widget::layout::LayoutOuter;
use rat_widget::text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use std::any::Any;
use std::cell::RefCell;
#[cfg(feature = "crossterm")]
use std::path::PathBuf;
use std::rc::Rc;
#[cfg(feature = "crossterm")]
use try_as_traits::TryAsRef;

/// Create a render-fn for FileDialog to be used with DialogStack.
pub fn file_dialog_render<Event, Error, Context: SalsaContext<Event, Error>>(
    layout: LayoutOuter,
    style: FileDialogStyle,
) -> impl Fn(Rect, &mut Buffer, &mut dyn Any, &mut Context)
where
    Event: 'static,
    Error: From<std::io::Error> + 'static,
{
    move |area: Rect, buf: &mut Buffer, state: &mut dyn Any, ctx: &mut Context| {
        if let Some(state) = state.downcast_mut::<FileDialogState>() {
            let area = layout.layout(area);
            FileDialog::new()
                .styles(style.clone())
                .render(area, buf, state);
            ctx.set_screen_cursor(state.screen_cursor());
        } else if let Some(state) = state.downcast_mut::<Rc<RefCell<FileDialogState>>>() {
            let mut state = state.borrow_mut();

            let area = layout.layout(area);
            FileDialog::new()
                .styles(style.clone())
                .render(area, buf, &mut *state);
            ctx.set_screen_cursor(state.screen_cursor());
        } else {
            panic!("unknown state type");
        }
    }
}

/// Create an event-fn for FileDialog to be used with DialogStack.
#[allow(unused_variables)]
#[cfg(feature = "crossterm")]
pub fn file_dialog_event<Event, Error, Context: SalsaContext<Event, Error>>(
    map: impl Fn(Result<PathBuf, ()>) -> Event,
) -> impl Fn(&Event, &mut dyn Any, &mut Context) -> Result<Control<Event>, Error>
where
    Event: TryAsRef<crossterm::event::Event> + 'static,
    Error: From<std::io::Error> + 'static,
{
    move |event: &Event, state: &mut dyn Any, ctx: &mut Context| -> Result<Control<Event>, Error> {
        if let Some(state) = state.downcast_mut::<FileDialogState>() {
            if let Some(event) = event.try_as_ref() {
                try_flow!(match state.handle(event, Dialog)? {
                    FileOutcome::Cancel => {
                        Control::Close(map(Err(())))
                    }
                    FileOutcome::Ok(f) => {
                        Control::Close(map(Ok(f)))
                    }
                    r => r.into(),
                });
            }
        } else if let Some(state) = state.downcast_mut::<Rc<RefCell<FileDialogState>>>() {
            let mut state = state.borrow_mut();

            if let Some(event) = event.try_as_ref() {
                try_flow!(match state.handle(event, Dialog)? {
                    FileOutcome::Cancel => {
                        Control::Close(map(Err(())))
                    }
                    FileOutcome::Ok(f) => {
                        Control::Close(map(Ok(f)))
                    }
                    r => r.into(),
                });
            }
        } else {
            panic!("unknown state type");
        }
        Ok(Control::Continue)
    }
}

/// Create an event-fn for FileDialog to be used with DialogStack.
#[allow(unused_variables)]
#[cfg(feature = "crossterm")]
pub fn file_dialog_event2<Event, Error, Context: SalsaContext<Event, Error>>(
    map: impl Fn(FileOutcome) -> Event,
) -> impl Fn(&Event, &mut dyn Any, &mut Context) -> Result<Control<Event>, Error>
where
    Event: TryAsRef<crossterm::event::Event> + 'static,
    Error: From<std::io::Error> + 'static,
{
    move |event: &Event, state: &mut dyn Any, ctx: &mut Context| -> Result<Control<Event>, Error> {
        if let Some(state) = state.downcast_mut::<FileDialogState>() {
            if let Some(event) = event.try_as_ref() {
                try_flow!(match state.handle(event, Dialog)? {
                    r @ FileOutcome::Cancel => Control::Close(map(r)),
                    r @ FileOutcome::Ok(_) => Control::Close(map(r)),
                    r @ FileOutcome::OkList(_) => Control::Close(map(r)),
                    r => r.into(),
                });
            }
        } else if let Some(state) = state.downcast_mut::<Rc<RefCell<FileDialogState>>>() {
            let mut state = state.borrow_mut();

            if let Some(event) = event.try_as_ref() {
                try_flow!(match state.handle(event, Dialog)? {
                    r @ FileOutcome::Cancel => Control::Close(map(r)),
                    r @ FileOutcome::Ok(_) => Control::Close(map(r)),
                    r @ FileOutcome::OkList(_) => Control::Close(map(r)),
                    r => r.into(),
                });
            }
        } else {
            panic!("unknown state type");
        }
        Ok(Control::Continue)
    }
}
