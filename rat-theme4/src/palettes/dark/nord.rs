use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Nord
/// Credits to original https://github.com/arcticicestudio/nord-vim
/// 
const DARKNESS: u8 = 63;

pub const NORD: Palette = Palette {
    name: Cow::Borrowed("Nord"), 

    color: [
        Palette::interpolate2(0xe5e9f0, 0xe5e9f0, 0x0, 0x0),
        Palette::interpolate2(0x2e3440, 0x2e3440, 0x0, 0x0),
        Palette::interpolate(0xd8dee9, 0xd8dee9, DARKNESS),
        Palette::interpolate(0x9ab3d3, 0x9ab3d3, DARKNESS),
        Palette::interpolate(0xd8dee9, 0xd8dee9, DARKNESS),
        Palette::interpolate(0x1b1f26, 0x2e3440, DARKNESS),
        Palette::interpolate(0x434c5e, 0x66748e, DARKNESS),
        Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
        Palette::interpolate(0xd08770, 0xd08770, DARKNESS),
        Palette::interpolate(0xeadbbe, 0xeadbbe, DARKNESS),
        Palette::interpolate(0xa3be8c, 0xa3be8c, DARKNESS),
        Palette::interpolate(0x8fbcbb, 0x8fbcbb, DARKNESS),
        Palette::interpolate(0x88c0d0, 0x88c0d0, DARKNESS),
        Palette::interpolate(0x88c0d0, 0x88c0d0, DARKNESS),
        Palette::interpolate(0x81a1c1, 0x81a1c1, DARKNESS),
        Palette::interpolate(0x5e81ac, 0x5e81ac, DARKNESS),
        Palette::interpolate(0xb48ead, 0xb48ead, DARKNESS),
        Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
        Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 1),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Gray, 0),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 0),
        define_alias("document-arrow.fg", Colors::None, 0),
        define_alias("document-base.bg", Colors::Black, 1),
        define_alias("document-border.fg", Colors::None, 0),
        define_alias("focus.bg", Colors::Primary, 0),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Blue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Blue, 0),
        define_alias("hover.bg", Colors::Primary, 0),
        define_alias("input.bg", Colors::Gray, 2),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("menu-base.bg", Colors::Gray, 0),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::White, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 0),
        define_alias("shadow.bg", Colors::TextDark, 0),
        define_alias("status-base.bg", Colors::Gray, 0),
        define_alias("text-focus.bg", Colors::Primary, 0),
        define_alias("text-select.bg", Colors::Secondary, 0),
        define_alias("title.bg", Colors::Red, 0),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::Yellow, 0),
    ]),
};

