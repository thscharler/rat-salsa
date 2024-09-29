# rat-salsa

An application event-loop with ratatui and crossterm.

![image][refMDEditGif]

## Companion Crates

rat-salsa provides

- application event loop [run_tui]
    - [background tasks](AppContext::spawn)
    - [timers](AppContext::add_timer)
    - crossterm
    - [messages](AppContext::queue)
    - [focus](AppContext::focus)
    - [control-flow](Control)
- traits for
    - [AppWidget]
    - [AppState]

There is more:

* [rat-widget](https://docs.rs/rat-widget)
  widget library
* [rat-scrolled](https://docs.rs/rat-scrolled)
  utilities for scrolling. Included in rat-widget.
* [rat-ftable](https://docs.rs/rat-ftable)
  table. uses traits to render your data, and renders only the visible cells.
  this makes rendering effectively O(1) in regard to the number of rows.
  Included in rat-widget.
* [rat-focus](https://docs.rs/rat-focus)
  Primitives for focus-handling as used by rat-widget. Included in rat-widget.
* [rat-event](https://docs.rs/rat-event)
  Defines the primitives for event-handling. Included in rat-widget.
* [rat-theme](https://docs.rs/rat-theme)
  Color-palettes and widget styles.

## Example

The examples directory contains some examples

- [files.rs][refFiles]: Minimal filesystem browser.
- [mdedit.rs][refMDEdit]: Minimal markdown editor.
- [life.rs][refLife]: Game of Life.

There are some starters too

- [minimal.rs][refMinimal]: Minimal application with a menubar and statusbar.
- [ultra.rs][refUltra]: Absolute minimum setup.

![image][refFilesGif]


[refFilesGif]: https://github.com/thscharler/rat-salsa/blob/master/files.gif?raw=true

[refMDEditGif]: https://github.com/thscharler/rat-salsa/blob/master/mdedit.gif?raw=true

[refLife]: https://github.com/thscharler/rat-salsa/blob/master/examples/life.rs

[refMDEdit]: https://github.com/thscharler/rat-salsa/blob/master/examples/mdedit.rs

[refFiles]: https://github.com/thscharler/rat-salsa/blob/master/examples/files.rs

[refMinimal]: https://github.com/thscharler/rat-salsa/blob/master/examples/minimal.rs

[refUltra]: https://github.com/thscharler/rat-salsa/blob/master/examples/ultra.rs
