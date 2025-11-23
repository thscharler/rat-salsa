use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Solarized
/// credit https://github.com/altercation/solarized/tree/master/vim-colors-solarized
const DARKNESS: u8 = 63;

pub const SOLARIZED: Palette = Palette {
    name: Cow::Borrowed("Solarized"), 

    color: [
        Palette::interpolate2(0xeee8d5, 0xfdf6e3, 0x0, 0x0),
        Palette::interpolate2(0x002b36, 0x073642, 0x0, 0x0),
        Palette::interpolate(0xcb4b16, 0xcb4b16, DARKNESS),
        Palette::interpolate(0x586e75, 0x839496, DARKNESS),
        Palette::interpolate(0xeee8d5, 0xfdf6e3, DARKNESS),
        Palette::interpolate(0x002b36, 0x073642, DARKNESS),
        Palette::interpolate(0x586e75, 0x839496, DARKNESS),
        Palette::interpolate(0xdc322f, 0xdc322f, DARKNESS),
        Palette::interpolate(0xcb4b16, 0xcb4b16, DARKNESS),
        Palette::interpolate(0xb58900, 0xb58900, DARKNESS),
        Palette::interpolate(0x859900, 0x859900, DARKNESS),
        Palette::interpolate(0x859900, 0x859900, DARKNESS),
        Palette::interpolate(0x2aa198, 0x2aa198, DARKNESS),
        Palette::interpolate(0x2aa198, 0x2aa198, DARKNESS),
        Palette::interpolate(0x268bd2, 0x268bd2, DARKNESS),
        Palette::interpolate(0x268bd2, 0x268bd2, DARKNESS),
        Palette::interpolate(0x6c71c4, 0x6c71c4, DARKNESS),
        Palette::interpolate(0xd33682, 0xd33682, DARKNESS),
        Palette::interpolate(0xd33682, 0xd33c82, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Secondary, 3),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Black, 3),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 0),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Blue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Blue, 0),
        define_alias("hover.bg", Colors::Primary, 0),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Red, 0),
        define_alias("key-binding.bg", Colors::Gray, 0),
        define_alias("label.fg", Colors::TextLight, 0),
        define_alias("md+hidden", Colors::None, 0),
        define_alias("menu-base.bg", Colors::Black, 2),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::Gray, 1),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Gray, 7),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Black, 2),
        define_alias("text-focus.bg", Colors::Primary, 1),
        define_alias("text-select.bg", Colors::Secondary, 0),
        define_alias("title.bg", Colors::None, 0),
        define_alias("title.fg", Colors::Blue, 0),
        define_alias("week-header.fg", Colors::Gray, 3),
    ]),
};

