
# Don't reinvent the wheel

## What works nicely with ratatui

- Widgets/StatefulWidget

The traits are well established, don't touch those.

- Buffer

The buffer is less intriguing. Can't do layers and setting the
cursor only works on the Frame. But its tightly integrated with
Widget, so don't touch this too.

- All the background stuff to make things work. Terminals have
  a legacy that's far older than COBOL, that should tell you
  something...
  
I'm very grateful that I don't have to deal with ESC sequences 
myself :-)  

=> So any widgets work as plain ratatui Widget/StatefulWidget and
workaround its limitations, maybe add things at the edges. 
  
You can see the result in the [rat-widget][refRatWidget] crate. 


## And the missing bits

- [Event handling][refRatEvent]

Defines some common ground that is customizable, needs no Arc or callbacks. 
And theoretically supports other event-systems than crossterm even if 
I don't implement any. 

- [Screen cursor][refRatCursor]

A tiny tweak to get the darn cursor position up to the main renderer. 

- [Focus handling][refRatFocus]

This is basics. Works with a tiny bit of state shared between the Focus
management and the widgets. So both can be kept mostly independent and
the widgets will still work even if no Focus is present. 

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





