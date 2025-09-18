![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-popup.svg)](https://crates.io/crates/rat-popup)
[![Documentation](https://docs.rs/rat-popup/badge.svg)](https://docs.rs/rat-popup)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-popup)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-popup GitHub][refGitHubPopup]

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-popup/changes.md)

# Rat-Popup

This is not a standalone widget, this is support for widgets that need
a popup window.

The main function is, that is uses the [Placement](crate::Placement)
constraint and a few others to find out where to render the popup.
When rendering the PopupCore it calculates the area for the popup
and resets the space.

The main widget then can use the resulting PopupCoreState::area
to render its own content.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refGitHubPopup]: https://github.com/thscharler/rat-popup/tree/master/examples