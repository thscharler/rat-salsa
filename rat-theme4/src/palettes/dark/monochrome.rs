use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Monochrome
const DARKNESS: u8 = 63;

pub const MONOCHROME: Palette = Palette {
    name: Cow::Borrowed("Monochrome"), 

    color: [
        Palette::interpolate2(0xd8dee9, 0xd8dee9, 0x0, 0x0),
        Palette::interpolate2(0x101010, 0x202020, 0x0, 0x0),
        Palette::interpolate(0x708187, 0x9ab2ba, DARKNESS),
        Palette::interpolate(0x424242, 0x677777, DARKNESS),
        Palette::interpolate(0xd8dee9, 0xd7dde8, DARKNESS),
        Palette::interpolate(0x1a1a1a, 0x262626, DARKNESS),
        Palette::interpolate(0x424242, 0x677777, DARKNESS),
        Palette::interpolate(0xec8989, 0xec8989, DARKNESS),
        Palette::interpolate(0xefb6a0, 0xefb6a0, DARKNESS),
        Palette::interpolate(0xffe6b5, 0xffe6b5, DARKNESS),
        Palette::interpolate(0xeff6ab, 0xeff6ab, DARKNESS),
        Palette::interpolate(0xc9d36a, 0xc9d36a, DARKNESS),
        Palette::interpolate(0x6484a4, 0x6484a4, DARKNESS),
        Palette::interpolate(0x9aafe6, 0x9aafe6, DARKNESS),
        Palette::interpolate(0x8abae1, 0x8abae1, DARKNESS),
        Palette::interpolate(0xa5c6e1, 0xa5c6e1, DARKNESS),
        Palette::interpolate(0xdb9fe9, 0xdb9fe9, DARKNESS),
        Palette::interpolate(0xda838b, 0xda838b, DARKNESS),
        Palette::interpolate(0xeca8a8, 0xeca8a8, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Black, 2),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::DeepBlue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::DeepBlue, 0),
        define_alias("hover.bg", Colors::Secondary, 2),
        define_alias("input.bg", Colors::Gray, 0),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("md+hidden", Colors::None, 0),
        define_alias("menu-base.bg", Colors::Black, 1),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::Gray, 1),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Gray, 2),
        define_alias("shadow.bg", Colors::None, 0),
        define_alias("status-base.bg", Colors::Black, 1),
        define_alias("text-focus.bg", Colors::Primary, 1),
        define_alias("text-select.bg", Colors::Secondary, 0),
        define_alias("title.bg", Colors::LimeGreen, 0),
        define_alias("title.fg", Colors::TextDark, 3),
        define_alias("week-header.fg", Colors::Gray, 3),
    ]),
};

