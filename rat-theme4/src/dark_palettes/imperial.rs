use crate::{ColorIdx, Colors, Palette};

/// Imperial
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
const DARKNESS: u8 = 63;

pub const IMPERIAL: Palette = Palette {
    name: "Imperial", 

    color: [
        Palette::interpolate2(0xdedfe3, 0xf6f6f3, 0x0, 0x0),
        Palette::interpolate2(0x0f1014, 0x2a2b37, 0x0, 0x0),
        Palette::interpolate(0x3d0070, 0x8900f9, DARKNESS),
        Palette::interpolate(0x726100, 0xe0c200, DARKNESS),
        Palette::interpolate(0xdedfe3, 0xf6f6f3, DARKNESS),
        Palette::interpolate(0x0f1014, 0x2a2b37, DARKNESS),
        Palette::interpolate(0x3b3d4e, 0x6e7291, DARKNESS),
        Palette::interpolate(0x601414, 0xd22d2d, DARKNESS),
        Palette::interpolate(0x5e3913, 0xd3802c, DARKNESS),
        Palette::interpolate(0x756600, 0xd6b900, DARKNESS),
        Palette::interpolate(0x3c5e17, 0x80ce31, DARKNESS),
        Palette::interpolate(0x186218, 0x32c932, DARKNESS),
        Palette::interpolate(0x1b5944, 0x3abc93, DARKNESS),
        Palette::interpolate(0x1b5184, 0x2bcece, DARKNESS),
        Palette::interpolate(0x234668, 0x2b81d4, DARKNESS),
        Palette::interpolate(0x202083, 0x3232cc, DARKNESS),
        Palette::interpolate(0x4b0089, 0x8c00fd, DARKNESS),
        Palette::interpolate(0x4f1b4f, 0xbd44bd, DARKNESS),
        Palette::interpolate(0x47101d, 0xc3425b, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base.bg", ColorIdx(Colors::Gray, 2)),
        ("container-arrow.fg", ColorIdx(Colors::None, 0)),
        ("container-base.bg", ColorIdx(Colors::Black, 3)),
        ("container-border.fg", ColorIdx(Colors::None, 0)),
        ("dialog-arrow.fg", ColorIdx(Colors::None, 0)),
        ("dialog-base.bg", ColorIdx(Colors::Gray, 3)),
        ("dialog-border.fg", ColorIdx(Colors::None, 0)),
        ("disabled.bg", ColorIdx(Colors::Gray, 3)),
        ("document", ColorIdx(Colors::None, 0)),
        ("focus.bg", ColorIdx(Colors::Primary, 1)),
        ("footer.bg", ColorIdx(Colors::None, 0)),
        ("footer.fg", ColorIdx(Colors::BlueGreen, 2)),
        ("header.bg", ColorIdx(Colors::None, 0)),
        ("header.fg", ColorIdx(Colors::BlueGreen, 2)),
        ("heading-1", ColorIdx(Colors::BlueGreen, 0)),
        ("heading-2", ColorIdx(Colors::BlueGreen, 0)),
        ("hover.bg", ColorIdx(Colors::Gray, 1)),
        ("input.bg", ColorIdx(Colors::Gray, 2)),
        ("invalid.bg", ColorIdx(Colors::Red, 1)),
        ("key-binding.bg", ColorIdx(Colors::BlueGreen, 0)),
        ("label.fg", ColorIdx(Colors::White, 0)),
        ("menu-base.bg", ColorIdx(Colors::Black, 1)),
        ("month-header.fg", ColorIdx(Colors::White, 0)),
        ("popup-arrow.fg", ColorIdx(Colors::None, 0)),
        ("popup-base.bg", ColorIdx(Colors::Gray, 3)),
        ("popup-border.fg", ColorIdx(Colors::None, 0)),
        ("select.bg", ColorIdx(Colors::Secondary, 1)),
        ("shadow.bg", ColorIdx(Colors::TextDark, 0)),
        ("status-base.bg", ColorIdx(Colors::Black, 1)),
        ("sub-form", ColorIdx(Colors::Black, 2)),
        ("text-focus.bg", ColorIdx(Colors::Primary, 1)),
        ("text-select.bg", ColorIdx(Colors::Secondary, 1)),
        ("title.bg", ColorIdx(Colors::Red, 0)),
        ("title.fg", ColorIdx(Colors::TextLight, 0)),
        ("week-header.fg", ColorIdx(Colors::Gray, 3)),
    ],
};

