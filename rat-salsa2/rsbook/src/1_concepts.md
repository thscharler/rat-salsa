
# Don't reinvent the wheel

## What works nicely with ratatui

- Widgets/StatefulWidget

The traits are well established, don't touch those.

- Buffer

The buffer is less intriguing. Can't do layers and
setting the cursor only works on the Frame. But its
tightly integrated with Widget, so work with what you
have.

> All the widgets are plain ratatui Widgets/
> StatefulWidgets and work around the given
> limitations.

You can see this in the [rat-widget][refRatWidget] crate. 

## Missing bits

- [Event handling][refRatEvent]

Customizable, no Rc/Arc necessary because no callbacks, supports not only crossterm events. 

- [Screen cursor][refRatCursor]

A tiny tweak to get the darn cursor position up to the main renderer. 

- [Focus handling][refRatFocus]

I need this.

- [Scrolling][refRatScrolled]

There is Scrollbar and some widgets support offsets, but nothing reusable. 

- [Windows/Dialogs][refRatDialog]

No need to build a full window-manager, but showing dialog-windows is a nice to have. 


## More missing bits

- The [main event-loop][refRatSalsa]

There are examples how to do it, but a bit more is always nice:

- poll multiple event-sources
- rendering on demand vs game-loop
- communication between application components/subsystems
- clean shutdown even on panic!
- background threads and futures
- timers


[refRatEvent]: https://docs.rs/rat-event

[refRatCursor]: https://docs.rs/rat-cursor

[refRatFocus]: https://docs.rs/rat-focus

[refRatScrolled]: https://docs.rs/rat-scrolled

[refRatDialog]: https://docs.rs/rat-dialog

[refRatSalsa]: https://docs.rs/rat-salsa

[refRatWidget]: https://docs.rs/rat-widget





