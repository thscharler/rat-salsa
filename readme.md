# rat-salsa

An application event-loop with ratatui and crossterm.

## run-tui

This function runs the event-loop.

It takes a [TuiApp] trait, which collects all the involved types, and provides
the basic operations. There is a separation of the data-model, ui-state and
actions upon the data. Actions can also be executed on a background worker thread
and communicate back to the event-loop via a channel.

* Continue -
  Break,
  Repaint,
  Action(Action),
  Quit,

TODO