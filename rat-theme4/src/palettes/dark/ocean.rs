use crate::palette::{Colors, Palette, define_alias};
use std::borrow::Cow;

/// Ocean
/// My take on an ocean theme.
const DARKNESS: u8 = 85;

pub const OCEAN: Palette = Palette {
    name: Cow::Borrowed("Ocean"),

    color: [
        Palette::interpolate2(0xe5e5dd, 0xf2f2ee, 0x0, 0x0),
        Palette::interpolate2(0x030305, 0x0c092c, 0x0, 0x0),
        Palette::interpolate(0xea7f38, 0xffbf3c, DARKNESS),
        Palette::interpolate(0x2b4779, 0x6688cc, DARKNESS),
        Palette::interpolate(0xe5e5dd, 0xf2f2ee, DARKNESS),
        Palette::interpolate(0x181828, 0x25233a, DARKNESS),
        Palette::interpolate(0x4f6167, 0xbcc7cc, DARKNESS),
        Palette::interpolate(0xff5e7f, 0xff9276, DARKNESS),
        Palette::interpolate(0xff9f5b, 0xffdc94, DARKNESS),
        Palette::interpolate(0xffda5d, 0xfff675, DARKNESS),
        Palette::interpolate(0x7d8447, 0xe1e5b9, DARKNESS),
        Palette::interpolate(0x658362, 0x99c794, DARKNESS),
        Palette::interpolate(0x3a615c, 0x5b9c90, DARKNESS),
        Palette::interpolate(0x24adbc, 0xb8dade, DARKNESS),
        Palette::interpolate(0x4f86ca, 0xbfdcff, DARKNESS),
        Palette::interpolate(0x2b4779, 0x6688cc, DARKNESS),
        Palette::interpolate(0x5068d7, 0xc7c4ff, DARKNESS),
        Palette::interpolate(0x7952d6, 0xc9bde4, DARKNESS),
        Palette::interpolate(0x9752d6, 0xcebde7, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 1),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Black, 3),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("document-arrow.fg", Colors::None, 0),
        define_alias("document-base.bg", Colors::Black, 1),
        define_alias("document-border.fg", Colors::None, 0),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::Blue, 0),
        define_alias("footer.fg", Colors::TextLight, 0),
        define_alias("header.bg", Colors::Blue, 0),
        define_alias("header.fg", Colors::TextLight, 0),
        define_alias("hover.bg", Colors::Primary, 2),
        define_alias("input-focus.bg", Colors::Primary, 1),
        define_alias("input-select.bg", Colors::Secondary, 1),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Red, 0),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("menu-base.bg", Colors::Black, 1),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::Gray, 2),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 1),
        define_alias("shadow.bg", Colors::Black, 3),
        define_alias("status-base.bg", Colors::Black, 1),
        define_alias("title.bg", Colors::BlueGreen, 0),
        define_alias("title.fg", Colors::TextLight, 1),
        define_alias("week-header.fg", Colors::Gray, 2),
    ]),
};
