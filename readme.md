# rat-salsa

An application event-loop with ratatui and crossterm.

![image][refMDEditGif]

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

## Book

For a start you can have a look at the [book][refRSBook].

## Companion Crates

* [rat-widget](https://docs.rs/rat-widget)
  widget library. Incorporates everything below, but each crate
  can be used on its own too.

  Foundational crates:

    * [rat-event](https://docs.rs/rat-event)
      Defines the primitives for event-handling.
    * [rat-cursor](https:/docs.rs/rat-cursor)
      Defines just one trait to propagate the required screen cursor position.
    * [rat-focus](https://docs.rs/rat-focus)
      Primitives for focus-handling.
    * [rat-scrolled](https://docs.rs/rat-scrolled)
      Utility widgets for scrolling.
    * [rat-popup](https://docs.rs/rat-popup)
      Utility widget to help with popups.

  Crates that deal with specific categories of widgets.

    * [rat-ftable](https://docs.rs/rat-ftable)
      table. uses traits to render your data, and renders only the visible cells.
      this makes rendering effectively O(1) in regard to the number of rows.
    * [rat-menu](https://docs.rs/rat-menu)
      Menu widgets.
    * [rat-text](https://docs.rs/rat-text)
      Text/Value input widgets.
    * [rat-window](https://docs.rs/rat-window)
      __Stopped__ for now. Implement windows in the tui.
      Can work with dyn StatefulWidgets too. The groundwork is done,
      but it's missing a lot of implementation.

  And my 10ct on theming.

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

[refRSBook]: https://thscharler.github.io/rat-salsa/