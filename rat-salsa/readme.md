![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-salsa2.svg)](https://crates.io/crates/rat-salsa2)
[![Documentation](https://docs.rs/rat-salsa2/badge.svg)](https://docs.rs/rat-salsa2)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa2)

# rat-salsa-2

Runs an application event-loop for ratatui and crossterm.

It can

- poll multiple event-sources fairly
- run [background tasks](SalsaContext::spawn) in one or more worker threads.
- run [background tasks](SalsaContext::spawn_async) as async tasks.
- define [timers](SalsaContext::add_timer).
- work as a message-queue for in-app messages.
- support focus-handling with [SalsaContext::focus](SalsaContext::focus)

All incoming events are converted to an application defined event-type,
and are distributed by calling an event-handler function. This function
returns a [control-flow](Control) which dictates further actions.

## Changes

[Changes](https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/changes.md)

## Book

... coming soon(ish) ...

## Companion Crates

* [rat-widget](https://docs.rs/rat-widget)
  widget library. Incorporates everything below, but each crate
  can be used on its own too.

  Foundational crates

    * [rat-event](https://docs.rs/rat-event)
      Defines the primitives for event-handling.
    * [rat-cursor](https://docs.rs/rat-cursor)
      Defines just one trait to propagate the required screen cursor position.
    * [rat-focus](https://docs.rs/rat-focus)
      Primitives for focus-handling.
    * [rat-reloc](https://docs.rs/rat-reloc)
      Relocate widgets after rendering. Needed support for view-like widgets.
    * [rat-scrolled](https://docs.rs/rat-scrolled)
      Utility widgets for scrolling.
    * [rat-popup](https://docs.rs/rat-popup)
      Utility widget to help with popups.
    * [rat-dialog](https:://docs.rs/rat-dialog)
      Stacks windows/dialogs above the main application.

  Crates for specific widgets

    * [rat-ftable](https://docs.rs/rat-ftable)
      table. uses traits to render your data, and renders only the visible cells.
      this makes rendering effectively O(1) in regard to the number of rows.
    * [rat-menu](https://docs.rs/rat-menu)
      Menu widgets.
    * [rat-text](https://docs.rs/rat-text)
      Text/Value input widgets.
    * [rat-markdown](https://docs.rs/rat-markdown)
      Extension for TextArea for markdown.

  And my 10ct on theming.

    * [rat-theme](https://docs.rs/rat-theme)
      Color-palettes and widget styles.
    * [rat-theme2](https://docs.rs/rat-theme2)
      More colors, mainly.

## Example

The examples directory contains some examples

- [files.rs][refFiles]: Minimal filesystem browser.
- [mdedit.rs][refMDEdit]: Minimal markdown editor.
- [life.rs][refLife]: Game of Life.
- [async1.rs][refAsync1]: Async tasks.
- [logscroll.rs][refLogscroll]: Logfile view and find.
- [theme_samples.rs][refThemeSamples]: Theme show-room.
- [turbo.rs][refTurbo]: Reboot Turbo Pascal.

There are some starters too

- [minimal.rs][refMinimal]: Minimal application with a menubar and statusbar.
- [ultra.rs][refUltra]: Absolute minimum setup.

![image][refFilesGif]


[refFilesGif]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/files.gif?raw=true

[refMDEditGif]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/mdedit.gif?raw=true

[refLife]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/life.rs

[refAsync1]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/async1.rs

[refLogscroll]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/logscroll.rs

[refThemeSamples]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/theme_samples.rs

[refTurbo]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/turbo.rs

[refMDEdit]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/mdedit.rs

[refFiles]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/files.rs

[refMinimal]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/minimal.rs

[refUltra]: https://github.com/thscharler/rat-salsa2/blob/master/rat-salsa2/examples/ultra.rs

[refRSBook]: https://thscharler.github.io/rat-salsa2/




