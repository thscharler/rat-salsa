use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// OxoCarbon
const DARKNESS: u8 = 63;

pub const OXOCARBON: Palette = Palette {
    name: Cow::Borrowed("OxoCarbon"), 

    color: [
        Palette::interpolate2(0xf2f4f8, 0xf9fbff, 0x0, 0x0),
        Palette::interpolate2(0x0f0f0f, 0x202020, 0x0, 0x0),
        Palette::interpolate(0x78a9ff, 0x78a9ff, DARKNESS),
        Palette::interpolate(0xb5e8e0, 0xb5e8e0, DARKNESS),
        Palette::interpolate(0xdde1e6, 0xffffff, DARKNESS),
        Palette::interpolate(0x0f0f0f, 0x202020, DARKNESS),
        Palette::interpolate(0x464646, 0x5f5f5f, DARKNESS),
        Palette::interpolate(0xee5396, 0xee5396, DARKNESS),
        Palette::interpolate(0xf8bd96, 0xf8bd96, DARKNESS),
        Palette::interpolate(0xfae3b0, 0xfae3b0, DARKNESS),
        Palette::interpolate(0x08bdba, 0x08bdba, DARKNESS),
        Palette::interpolate(0x42be65, 0x42be65, DARKNESS),
        Palette::interpolate(0xb5e8e0, 0xb5e8e0, DARKNESS),
        Palette::interpolate(0x3ddbd9, 0x3ddbd9, DARKNESS),
        Palette::interpolate(0x33b1ff, 0x33b1ff, DARKNESS),
        Palette::interpolate(0x78a9ff, 0x78a9ff, DARKNESS),
        Palette::interpolate(0xbe95ff, 0xbe95ff, DARKNESS),
        Palette::interpolate(0xd0a9e5, 0xd0a9e5, DARKNESS),
        Palette::interpolate(0xff7eb6, 0xff77b4, DARKNESS),
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
        define_alias("document-base.bg", Colors::Black, 2),
        define_alias("document-border.fg", Colors::None, 0),
        define_alias("focus.bg", Colors::Primary, 0),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::DeepBlue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::DeepBlue, 0),
        define_alias("hover.bg", Colors::Secondary, 0),
        define_alias("input-focus.bg", Colors::Primary, 1),
        define_alias("input-select.bg", Colors::Secondary, 1),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("menu-base.bg", Colors::Black, 0),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::Gray, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 0),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Black, 0),
        define_alias("title.bg", Colors::Red, 0),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::BlueGreen, 0),
    ]),
};

