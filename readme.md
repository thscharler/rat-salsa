[![crates.io](https://img.shields.io/crates/v/rat-popup.svg)](https://crates.io/crates/rat-popup)
[![Documentation](https://docs.rs/rat-popup/badge.svg)](https://docs.rs/rat-popup)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-popup)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-popup GitHub][refGitHubPopup]

* [Changes](https://github.com/thscharler/rat-popup/blob/master/changes.md)

# Rat-Popup

This is not a full widget, rather it supports popup widgets.

The main point for its existence is [Placement](crate::Placement)
which locates the popup-widget relative to an area or point.

The rendered size for the popup is given to render(), and the
actual widget can then render its content in PopupCoreState::widget_area.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refGitHubPopup]: https://github.com/thscharler/rat-popup/tree/master/examples