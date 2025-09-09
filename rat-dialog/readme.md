![alpha 2](https://img.shields.io/badge/stability-ɑ--2-850101)
[![crates.io](https://img.shields.io/crates/v/rat-dialog.svg)](https://crates.io/crates/rat-dialog)
[![Documentation](https://docs.rs/rat-dialog/badge.svg)](https://docs.rs/rat-dialog)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

__unstable__

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-dialog/changes.md)

# Rat Dialog

This crates provides a DialogStack that can be used
with rat-salsa apps. It can stack and render any number of
dialog-windows on top of the main application.

It uses the rat-salsa traits for AppWidget/AppState for rendering
and event handling.

```rust no_run

# use std::path::PathBuf;
use anyhow::Error;
use rat_dialog::DialogStackState;
use crate::rat_dialog::DialogWidget;
use rat_dialog::widgets::{FileDialog, FileDialogState};
# use rat_salsa::{AppState, Control};
# use rat_theme2::DarkTheme;
# use rat_widget::event::FileOutcome;
# struct GlobalState { dialogs: DialogStackState<GlobalState, MyEvent, Error>, theme: DarkTheme }
# enum MyEvent { Event(crossterm::event::Event), Status(u16, String) }
# impl TryFrom<&MyEvent> for &crossterm::event::Event {
#   type Error = ();
#   fn try_from(value: &MyEvent) -> Result<Self, Self::Error> {
#       match self {
#           MyEvent::Event(event) => Ok(event),
#           _ => Err(())    
#       }
#   }
# }
# struct MyAppState {}

impl AppState<GlobalState, MyEvent, Error> for MyAppState {
    fn event(
        &mut self,
        event: &MyEvent,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MyEvent, Error>,
    ) -> Result<Control<MyEvent>, Error> {
        if matches!(event, MyEvent::Event(event)) {
            let mut state = FileDialogState::new();
            state.save_dialog_ext(PathBuf::from("."), "", "pas")?;
            state.map_outcome(|r| match r {
                FileOutcome::Ok(f) => {
                    Control::Event(MyEvent::Status(0, format!("New file {:?}", f)))
                }
                r => r.into(),
            });
            
            ctx.g.dialogs.push_dialog(
                |area, buf, state, ctx| {
                    FileDialog::new()
                        .styles(ctx.g.theme.file_dialog_style())
                        .render(area, buf, state, ctx)
                },
                state                    
            );
            
            Ok(Control::Changed)
        } else {
            Ok(Control::Continue)
        }
    }
}
```

During rendering of the application:

```rust no_run
# use anyhow::Error;
# use ratatui::buffer::Buffer;
# use ratatui::layout::Rect;
use rat_dialog::{DialogStack, DialogStackState};
# use rat_salsa::{AppWidget,AppState, RenderContext};
# use rat_theme2::DarkTheme;
# struct MainApp;
# struct GlobalState { dialogs: DialogStackState<GlobalState, MyEvent, Error>, theme: DarkTheme }
# enum MyEvent { Event(crossterm::event::Event), Status(u16, String) }
# struct MainAppState {}
# impl AppState<GlobalState, MyEvent, Error> for MainAppState {}


impl AppWidget<GlobalState, MyEvent, Error> for MainApp {
    type State = MainAppState;

    fn render(
            &self,
            area: Rect,
            buf: &mut Buffer,
            state: &mut Self::State,
            ctx: &mut RenderContext<'_, GlobalState>,
    ) -> Result<(), Error> {

        // ... do all the rendering ...

        DialogStack.render(area, buf, &mut ctx.g.dialogs.clone(), ctx)?;

        Ok(())
   }
}   
``` 

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/


