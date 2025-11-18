![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-theme.svg)](https://crates.io/crates/rat-theme)
[![Documentation](https://docs.rs/rat-theme/badge.svg)](https://docs.rs/rat-theme)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-theme/changes.md)

# Theming support for rat-salsa

This splits themes in two parts,

* [Palette](crate::Palette)
  This defines a color palette. It contains a rainbow-table,  
  explicit primary, secondary colors and light/bright/dark/black
  text-colors.

  Plus it contains a list of aliases for semantic colors.
  e.g.: "label-fg", "focus", "select", "container-base"
  These point to specific colors in the palette and can
  be used to create the actual theme composition.

* [SalsaTheme](crate::SalsaTheme)
  Takes a palette and creates `Styles`.

  And it creates concrete `xxStyle` structs to configure
  specific rat-widgets. It can also store `yyStyle` structs
  for your own widgets.

## Extras

There is `pal-edit` a visual editor for palettes.
It can create the .rs palettes and has its own storage format too.
And, it can be configured to use extra aliases needed by your
application.

## Application specific palettes.

It's not too complicated, I just had no time to make an example yet.
Mostly it boils down to mapping a theme-name to a SalsaTheme+Palette.

## Loadable palettes.

It's doable. The .pal files are sufficient to create a Palette, but
it's not implemented yet.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/
