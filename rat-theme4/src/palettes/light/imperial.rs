use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

/// Imperial Light
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
const DARKNESS: u8 = 63;

pub const IMPERIAL_LIGHT: Palette = Palette {
    name: Cow::Borrowed("Imperial Light"), 

    color: [
        Palette::interpolate2(0xdedfe3, 0xf6f6f3, 0x0, 0x0),
        Palette::interpolate2(0x0f1014, 0x2a2b37, 0x0, 0x0),
        Palette::interpolate(0x3d0070, 0x8900f9, DARKNESS),
        Palette::interpolate(0x444759, 0x959bc4, DARKNESS),
        Palette::interpolate(0xdedfe3, 0xf6f6f3, DARKNESS),
        Palette::interpolate(0x0f1014, 0x2a2b37, DARKNESS),
        Palette::interpolate(0x595c75, 0x7f84a8, DARKNESS),
        Palette::interpolate(0x601414, 0xd22d2d, DARKNESS),
        Palette::interpolate(0x5e3913, 0xd3802c, DARKNESS),
        Palette::interpolate(0x756600, 0xd6b900, DARKNESS),
        Palette::interpolate(0x3c5e17, 0x80ce31, DARKNESS),
        Palette::interpolate(0x186218, 0x32c932, DARKNESS),
        Palette::interpolate(0x1b5944, 0x3abc93, DARKNESS),
        Palette::interpolate(0x1b5184, 0x2bcece, DARKNESS),
        Palette::interpolate(0x234668, 0x2b81d4, DARKNESS),
        Palette::interpolate(0x202083, 0x3232cc, DARKNESS),
        Palette::interpolate(0x4b0089, 0x8c00ff, DARKNESS),
        Palette::interpolate(0x4f1b4f, 0xbd44c4, DARKNESS),
        Palette::interpolate(0x47101d, 0xc3425b, DARKNESS),
    ],
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::White, 0),
        define_alias("container-arrow.fg", Colors::Black, 3),
        define_alias("container-base.bg", Colors::Gray, 3),
        define_alias("container-border.fg", Colors::Black, 3),
        define_alias("dialog-arrow.fg", Colors::TextDark, 0),
        define_alias("dialog-base.bg", Colors::Gray, 3),
        define_alias("dialog-border.fg", Colors::TextDark, 0),
        define_alias("disabled.bg", Colors::Gray, 7),
        define_alias("document-arrow.fg", Colors::Black, 3),
        define_alias("document-base.bg", Colors::Gray, 0),
        define_alias("document-border.fg", Colors::Black, 3),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::BlueGreen, 0),
        define_alias("footer.fg", Colors::TextLight, 0),
        define_alias("header.bg", Colors::BlueGreen, 0),
        define_alias("header.fg", Colors::TextLight, 2),
        define_alias("hover.bg", Colors::Primary, 0),
        define_alias("input-focus.bg", Colors::Primary, 1),
        define_alias("input-select.bg", Colors::Secondary, 2),
        define_alias("input.bg", Colors::White, 0),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::TextLight, 0),
        define_alias("menu-base.bg", Colors::Gray, 2),
        define_alias("month-header.fg", Colors::White, 0),
        define_alias("popup-arrow.fg", Colors::Black, 3),
        define_alias("popup-base.bg", Colors::White, 2),
        define_alias("popup-border.fg", Colors::Black, 3),
        define_alias("select.bg", Colors::Secondary, 0),
        define_alias("shadow.bg", Colors::TextDark, 0),
        define_alias("status-base.bg", Colors::Gray, 2),
        define_alias("title.bg", Colors::Red, 0),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::Black, 0),
    ]),
};

