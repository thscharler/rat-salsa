![alpha 1](https://img.shields.io/badge/stability-ɑ--1-850101)
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
use rat_dialog::widgets::{FileDialog, FileDialogState};
# use rat_salsa::{AppState, Control};
# use rat_theme2::DarkTheme;
# use rat_widget::event::FileOutcome;
# struct GlobalState { dialogs: DialogStackState<GlobalState, MyEvent, Error>, theme: DarkTheme }
# enum MyEvent { Event(crossterm::event::Event), Status(u16, String) }
# struct MyAppState {}

impl AppState<GlobalState, MyEvent, Error> for MyAppState {
    fn event(
        &mut self,
        event: &MyEvent,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MyEvent, Error>,
    ) -> Result<Control<MyEvent>, Error> {
        if matches!(event, MyEvent::Event(event)) {
            ctx.g.dialogs.push_dialog(
                |_, ctx| {
                    Box::new(FileDialog::new().styles(ctx.g.theme.file_dialog_style()))
                },
                FileDialogState::new()
                    .save_dialog_ext(PathBuf::from("."), "", "pas")?
                    .map_outcome(|r| match r {
                        FileOutcome::Ok(f) => {
                            Control::Event(MyEvent::Status(0, format!("New file {:?}", f)))
                        }
                        r => r.into(),
                    }),
            );
            Ok(Control::Changed)
        } else {
            Ok(Control::Continue)
        }
    }
```

During rendering of the application:

```rust no_run
# use anyhow::Error;
# use ratatui::buffer::Buffer;
# use ratatui::layout::Rect;
use rat_dialog::{DialogStack, DialogStackState};
# use rat_salsa::{AppWidget, RenderContext};
# use rat_theme2::DarkTheme;
# struct MainApp;
# struct GlobalState { dialogs: DialogStackState<GlobalState, MyEvent, Error>, theme: DarkTheme }
# enum MyEvent { Event(crossterm::event::Event), Status(u16, String) }
# struct MainAppState {}

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
``` 



