use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Black&White
const DARKNESS: u8 = 63;

pub const BLACK_WHITE: Palette = Palette {
    name: Cow::Borrowed("Black&White"), 

    color: [
        Palette::interpolate2(0xffffff, 0xffffff, 0x0, 0x0),
        Palette::interpolate2(0x000000, 0x000000, 0x0, 0x0),
        Palette::interpolate(0xffffff, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0xffffff, DARKNESS),
        Palette::interpolate(0x000000, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 1),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Black, 0),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 2),
        define_alias("focus.bg", Colors::Primary, 0),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Blue, 3),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Blue, 3),
        define_alias("hover.bg", Colors::Gray, 0),
        define_alias("input.bg", Colors::Gray, 1),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("md+hidden", Colors::None, 0),
        define_alias("menu-base.bg", Colors::Gray, 2),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::White, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 2),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Gray, 2),
        define_alias("text-focus.bg", Colors::Primary, 0),
        define_alias("text-select.bg", Colors::Secondary, 1),
        define_alias("title.bg", Colors::Red, 3),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::Gray, 1),
    ]),
};

