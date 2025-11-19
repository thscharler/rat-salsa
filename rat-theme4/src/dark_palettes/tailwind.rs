use std::borrow::Cow;
use crate::{Colors, Palette, define_alias};

/// Tailwind
const DARKNESS: u8 = 92;

pub const TAILWIND: Palette = Palette {
    name: Cow::Borrowed("Tailwind"), 

    color: [
        Palette::interpolate2(0xccc9c7, 0xeaeaff, 0x0, 0x0),
        Palette::interpolate2(0x202021, 0x1c1c1c, 0x0, 0x0),
        Palette::interpolate(0xffb86a, 0xa07343, DARKNESS),
        Palette::interpolate(0xcad5e2, 0x62748e, DARKNESS),
        Palette::interpolate(0xc2c2d3, 0xfffbf9, DARKNESS),
        Palette::interpolate(0x252526, 0x111111, DARKNESS),
        Palette::interpolate(0xd6d3d1, 0x79716b, DARKNESS),
        Palette::interpolate(0xffa2a2, 0xfb2c36, DARKNESS),
        Palette::interpolate(0xffb86a, 0xff6900, DARKNESS),
        Palette::interpolate(0xffdf20, 0xf0b100, DARKNESS),
        Palette::interpolate(0xbbf451, 0x7ccf00, DARKNESS),
        Palette::interpolate(0x7bf1a8, 0x00c950, DARKNESS),
        Palette::interpolate(0x5ee9b5, 0x00bba7, DARKNESS),
        Palette::interpolate(0x46ecd5, 0x00b8db, DARKNESS),
        Palette::interpolate(0x74d4ff, 0x2b7fff, DARKNESS),
        Palette::interpolate(0x8ec5ff, 0x615fff, DARKNESS),
        Palette::interpolate(0xc4b4ff, 0x8e51ff, DARKNESS),
        Palette::interpolate(0xf4a8ff, 0xe12afb, DARKNESS),
        Palette::interpolate(0xfda5d5, 0xff2056, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Secondary, 1),
        define_alias("container-arrow.fg", Colors::Gray, 3),
        define_alias("container-base.bg", Colors::Secondary, 7),
        define_alias("container-border.fg", Colors::Gray, 4),
        define_alias("dialog-arrow.fg", Colors::Secondary, 2),
        define_alias("dialog-base.bg", Colors::Secondary, 5),
        define_alias("dialog-border.fg", Colors::Secondary, 2),
        define_alias("disabled.bg", Colors::Secondary, 3),
        define_alias("focus.bg", Colors::Primary, 0),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Secondary, 2),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Secondary, 2),
        define_alias("hover.bg", Colors::Secondary, 3),
        define_alias("input.bg", Colors::Secondary, 1),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::LimeGreen, 4),
        define_alias("label.fg", Colors::TextLight, 0),
        define_alias("menu-base.bg", Colors::Secondary, 6),
        define_alias("month-header.fg", Colors::Secondary, 2),
        define_alias("popup-arrow.fg", Colors::Secondary, 3),
        define_alias("popup-base.bg", Colors::Secondary, 6),
        define_alias("popup-border.fg", Colors::Secondary, 3),
        define_alias("select.bg", Colors::Primary, 2),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Secondary, 6),
        define_alias("text-focus.bg", Colors::Primary, 0),
        define_alias("text-select.bg", Colors::Primary, 2),
        define_alias("title.bg", Colors::None, 0),
        define_alias("title.fg", Colors::Secondary, 0),
        define_alias("week-header.fg", Colors::Secondary, 2),
    ]),
};

