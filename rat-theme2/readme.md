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

* [Palette](crate::Palette) - The underlying color-palette with enough colors to play
  around.
* [DarkTheme](crate::dark_theme::DarkTheme) takes a palette and produces Styles
  for rat-widgets.

This intentionally doesn't adhere to any trait, just provides some
baselines for each widget type. You can use this as is, or copy it
and adapt it for your applications needs.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

