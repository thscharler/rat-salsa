use std::borrow::Cow;
use crate::{Colors, Palette, define_alias};

/// Rust
/// Rusty theme.
const DARKNESS: u8 = 63;

pub const RUST: Palette = Palette {
    name: Cow::Borrowed("Rust"), 

    color: [
        Palette::interpolate2(0xd1ccc8, 0xefe6e6, 0x0, 0x0),
        Palette::interpolate2(0x161514, 0x0f0e0d, 0x0, 0x0),
        Palette::interpolate(0x75311a, 0xd25a32, DARKNESS),
        Palette::interpolate(0x77551d, 0xcd9537, DARKNESS),
        Palette::interpolate(0xc4bfbb, 0xede3e3, DARKNESS),
        Palette::interpolate(0x101011, 0x464251, DARKNESS),
        Palette::interpolate(0x726e6b, 0xa39d99, DARKNESS),
        Palette::interpolate(0x75311a, 0xd25a32, DARKNESS),
        Palette::interpolate(0x75431a, 0xd27a32, DARKNESS),
        Palette::interpolate(0x77551d, 0xcd9537, DARKNESS),
        Palette::interpolate(0x44664d, 0x699b76, DARKNESS),
        Palette::interpolate(0x44664d, 0x699b76, DARKNESS),
        Palette::interpolate(0x1a7574, 0x32d2d1, DARKNESS),
        Palette::interpolate(0x1a7574, 0x32d2d1, DARKNESS),
        Palette::interpolate(0x005d94, 0x38b6ff, DARKNESS),
        Palette::interpolate(0x005d94, 0x38b6ff, DARKNESS),
        Palette::interpolate(0x722234, 0xc63f5d, DARKNESS),
        Palette::interpolate(0x7b1964, 0xc62fa3, DARKNESS),
        Palette::interpolate(0x7b1964, 0xd332ad, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 1),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Gray, 2),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Blue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Blue, 0),
        define_alias("hover.bg", Colors::Blue, 0),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Purple, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 1),
        define_alias("label.fg", Colors::White, 2),
        define_alias("menu-base.bg", Colors::Black, 0),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::White, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 1),
        define_alias("shadow.bg", Colors::TextDark, 0),
        define_alias("status-base.bg", Colors::Black, 0),
        define_alias("text-focus.bg", Colors::Primary, 1),
        define_alias("text-select.bg", Colors::Secondary, 1),
        define_alias("title.bg", Colors::Primary, 1),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::Gray, 1),
    ]),
};

