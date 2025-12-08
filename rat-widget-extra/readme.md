![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-widget-extra.svg)](https://crates.io/crates/rat-widget-extra)
[![Documentation](https://docs.rs/rat-widget-extra/badge.svg)](https://docs.rs/rat-widget-extra)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-widget-extra/changes.md)

# rat-widget (extra)

This crate contains optional widgets for [rat-widget][refRatWidget].

These are widgets that are too specialized to be contained
in the main rat-widget crate.

Each widget will be behind a feature gate too.

* IBANInput - Text input and validation for IBAN bank account numbers.
* ColorInput - Color input widget.

[refRatWidget]: https://docs.rs/rat-widget

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/
