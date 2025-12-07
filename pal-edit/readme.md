This crate is a part of [rat-salsa][refRatSalsa].

This application can edit `.pal` palette files.

* It can export them as .rs palettes.
* It can read base46 themes used by neovim and convert them.

* As of latest the palette can be exported to `.json` too.
  And loaded back of course.

## Config

On start-up it creates a sample config in your systems
application config directory.

There you can add extra color-aliases if you need them.

## Parameters

`pal-edit [palette-file] [--alias aliases.ini]`