use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Monekai
const DARKNESS: u8 = 63;

pub const MONEKAI: Palette = Palette {
    name: Cow::Borrowed("Monekai"), 

    color: [
        Palette::interpolate2(0xf5f4f1, 0xfffefc, 0x0, 0x0),
        Palette::interpolate2(0x272822, 0x464741, 0x0, 0x0),
        Palette::interpolate(0xc11f5a, 0xf92672, DARKNESS),
        Palette::interpolate(0x5c7289, 0x81a1c1, DARKNESS),
        Palette::interpolate(0xf5f4f1, 0xf5f4f1, DARKNESS),
        Palette::interpolate(0x22231d, 0x2f302a, DARKNESS),
        Palette::interpolate(0x4d4e48, 0x64655f, DARKNESS),
        Palette::interpolate(0xe36d76, 0xe36d76, DARKNESS),
        Palette::interpolate(0xd39467, 0xd39467, DARKNESS),
        Palette::interpolate(0xe6c181, 0xe6c181, DARKNESS),
        Palette::interpolate(0x96c367, 0x96c367, DARKNESS),
        Palette::interpolate(0x96c367, 0x96c367, DARKNESS),
        Palette::interpolate(0x34bfd0, 0x34bfd0, DARKNESS),
        Palette::interpolate(0x41afef, 0x41afef, DARKNESS),
        Palette::interpolate(0x51afef, 0x51afef, DARKNESS),
        Palette::interpolate(0x81a1c1, 0x81a1c1, DARKNESS),
        Palette::interpolate(0xae81ff, 0xae81ff, DARKNESS),
        Palette::interpolate(0xf92672, 0xf72270, DARKNESS),
        Palette::interpolate(0xf98385, 0xf98381, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::Gray, 2),
        define_alias("container-base.bg", Colors::Black, 0),
        define_alias("container-border.fg", Colors::Gray, 2),
        define_alias("dialog-arrow.fg", Colors::Gray, 3),
        define_alias("dialog-base.bg", Colors::Black, 2),
        define_alias("dialog-border.fg", Colors::Gray, 3),
        define_alias("disabled.bg", Colors::Gray, 0),
        define_alias("focus.bg", Colors::Primary, 0),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::DeepBlue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::DeepBlue, 0),
        define_alias("hover.bg", Colors::Purple, 0),
        define_alias("input.bg", Colors::Gray, 0),
        define_alias("invalid.bg", Colors::RedPink, 0),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("md+hidden", Colors::None, 0),
        define_alias("menu-base.bg", Colors::Black, 1),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::Gray, 1),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 3),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Black, 1),
        define_alias("text-focus.bg", Colors::Primary, 1),
        define_alias("text-select.bg", Colors::Secondary, 1),
        define_alias("title.bg", Colors::None, 0),
        define_alias("title.fg", Colors::Magenta, 0),
        define_alias("week-header.fg", Colors::Gray, 3),
    ]),
};

