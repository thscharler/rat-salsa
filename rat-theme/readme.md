![stable](https://img.shields.io/badge/stability-β--3-850101)
[![crates.io](https://img.shields.io/crates/v/rat-theme.svg)](https://crates.io/crates/rat-theme)
[![Documentation](https://docs.rs/rat-theme/badge.svg)](https://docs.rs/rat-theme)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-theme)

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-theme/blob/master/changes.md)

# Theming support for rat-salsa

This splits themes in two parts,

* [Scheme](crate::Scheme) - The underlying color-scheme with enough colors to play
  around.
* [DarkTheme](crate::dark_theme::DarkTheme) takes that scheme and produces Styles
  for widgets.

  This intentionally doesn't adhere to any trait, just provides some
  baselines for each widget type. You can use this as is, or copy it
  and adapt it for your applications needs.

  > In the end I think this will be just some building blocks for
  > an application defined theme. I think most applications will need
  > more semantics than just 'some table', 'some list'.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

