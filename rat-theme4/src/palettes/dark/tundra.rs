use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Tundra
/// An adaption of nvchad's tundra theme.
/// -- Thanks to original theme for existing <https://github.com/sam4llis/nvim-tundra>
const DARKNESS: u8 = 63;

pub const TUNDRA: Palette = Palette {
    name: Cow::Borrowed("Tundra"), 

    color: [
        Palette::interpolate2(0xe6eaf2, 0xffffff, 0x0, 0x0),
        Palette::interpolate2(0x0b1221, 0x1a2130, 0x0, 0x0),
        Palette::interpolate(0xe6eaf2, 0xffffff, DARKNESS),
        Palette::interpolate(0xa8bbd4, 0x719bd3, DARKNESS),
        Palette::interpolate(0xe6eaf2, 0xffffff, DARKNESS),
        Palette::interpolate(0x0b1221, 0x1a2130, DARKNESS),
        Palette::interpolate(0x3e4554, 0x5f6675, DARKNESS),
        Palette::interpolate(0xfccaca, 0xfca5a5, DARKNESS),
        Palette::interpolate(0xfad9c5, 0xfbc19d, DARKNESS),
        Palette::interpolate(0xe8d7b7, 0xe8d4b0, DARKNESS),
        Palette::interpolate(0xbce8b7, 0xb5e8b0, DARKNESS),
        Palette::interpolate(0xbce8b7, 0xb5e8b0, DARKNESS),
        Palette::interpolate(0xa8bbd4, 0x719bd3, DARKNESS),
        Palette::interpolate(0xc8eafc, 0xbae6fd, DARKNESS),
        Palette::interpolate(0xc7d0fc, 0xa5b4fc, DARKNESS),
        Palette::interpolate(0xbfcaf2, 0x9baaf2, DARKNESS),
        Palette::interpolate(0xb7abd9, 0xb3a6da, DARKNESS),
        Palette::interpolate(0xffc9c9, 0xf98b8b, DARKNESS),
        Palette::interpolate(0xfffcad, 0xfecdd3, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 3),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Gray, 2),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("document-arrow.fg", Colors::None, 0),
        define_alias("document-base.bg", Colors::Black, 2),
        define_alias("document-border.fg", Colors::None, 0),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Blue, 3),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Blue, 3),
        define_alias("hover.bg", Colors::Secondary, 0),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Red, 3),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("menu-base.bg", Colors::Black, 1),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::White, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 1),
        define_alias("shadow.bg", Colors::TextDark, 0),
        define_alias("status-base.bg", Colors::Black, 1),
        define_alias("text-focus.bg", Colors::Primary, 1),
        define_alias("text-select.bg", Colors::Secondary, 1),
        define_alias("title.bg", Colors::Red, 0),
        define_alias("title.fg", Colors::TextDark, 3),
        define_alias("week-header.fg", Colors::BlueGreen, 0),
    ]),
};

