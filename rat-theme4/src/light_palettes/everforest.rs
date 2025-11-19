use std::borrow::Cow;
use crate::{Colors, Palette, define_alias};

/// EverForest
const DARKNESS: u8 = 63;

pub const EVERFOREST: Palette = Palette {
    name: Cow::Borrowed("EverForest"), 

    color: [
        Palette::interpolate2(0xd8d4cb, 0xfcf8ef, 0x0, 0x0),
        Palette::interpolate2(0x090a09, 0x2c2d2a, 0x0, 0x0),
        Palette::interpolate(0x5b738c, 0x8fa7bf, DARKNESS),
        Palette::interpolate(0x4e565c, 0x656d73, DARKNESS),
        Palette::interpolate(0xd3c6ab, 0xc4ac7b, DARKNESS),
        Palette::interpolate(0x272f35, 0x30383d, DARKNESS),
        Palette::interpolate(0x4e565c, 0x656d73, DARKNESS),
        Palette::interpolate(0xe67e80, 0xfc8f93, DARKNESS),
        Palette::interpolate(0xe69875, 0xf4ab8b, DARKNESS),
        Palette::interpolate(0xdbbc7f, 0xddc187, DARKNESS),
        Palette::interpolate(0xa7c080, 0xb8ce94, DARKNESS),
        Palette::interpolate(0x83c092, 0x83c092, DARKNESS),
        Palette::interpolate(0x69a59d, 0x77ada6, DARKNESS),
        Palette::interpolate(0x95d1c9, 0xafdbd5, DARKNESS),
        Palette::interpolate(0x7393b3, 0x8fa7bf, DARKNESS),
        Palette::interpolate(0x78b4ac, 0x8dc4bd, DARKNESS),
        Palette::interpolate(0xd699b6, 0xe5b7cd, DARKNESS),
        Palette::interpolate(0xff75a0, 0xf5608e, DARKNESS),
        Palette::interpolate(0xce8196, 0xbf7082, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Green, 4),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::BlueGreen, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::White, 0),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 2),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::DeepBlue, 7),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::DeepBlue, 7),
        define_alias("hover.bg", Colors::DeepBlue, 3),
        define_alias("input.bg", Colors::White, 0),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::Green, 0),
        define_alias("label.fg", Colors::TextDark, 1),
        define_alias("menu-base.bg", Colors::White, 3),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::White, 2),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 1),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::White, 3),
        define_alias("text-focus.bg", Colors::Primary, 2),
        define_alias("text-select.bg", Colors::Secondary, 1),
        define_alias("title.bg", Colors::None, 0),
        define_alias("title.fg", Colors::Green, 4),
        define_alias("week-header.fg", Colors::Gray, 0),
    ]),
};

