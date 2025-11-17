use crate::{ColorIdx, Colors, Palette};

/// Monochrome
const DARKNESS: u8 = 63;

pub const MONOCHROME: Palette = Palette {
    name: "Monochrome", 

    color: [
        Palette::interpolate2(0xd8dee9, 0xd8dee9, 0x0, 0x0),
        Palette::interpolate2(0x101010, 0x202020, 0x0, 0x0),
        Palette::interpolate(0x708187, 0x9ab2ba, DARKNESS),
        Palette::interpolate(0x424242, 0x677777, DARKNESS),
        Palette::interpolate(0xd8dee9, 0xd7dde8, DARKNESS),
        Palette::interpolate(0x1a1a1a, 0x202020, DARKNESS),
        Palette::interpolate(0x424242, 0x677777, DARKNESS),
        Palette::interpolate(0xec8989, 0xec8989, DARKNESS),
        Palette::interpolate(0xefb6a0, 0xefb6a0, DARKNESS),
        Palette::interpolate(0xffe6b5, 0xffe6b5, DARKNESS),
        Palette::interpolate(0xeff6ab, 0xeff6ab, DARKNESS),
        Palette::interpolate(0xc9d36a, 0xc9d36a, DARKNESS),
        Palette::interpolate(0x6484a4, 0x6484a4, DARKNESS),
        Palette::interpolate(0x9aafe6, 0x9aafe6, DARKNESS),
        Palette::interpolate(0x8abae1, 0x8abae1, DARKNESS),
        Palette::interpolate(0xa5c6e1, 0xa5c6e1, DARKNESS),
        Palette::interpolate(0xdb9fe9, 0xdb9fe9, DARKNESS),
        Palette::interpolate(0xda838b, 0xda838b, DARKNESS),
        Palette::interpolate(0xeca8a8, 0xeca8a8, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base", ColorIdx(Colors::Gray, 0)),
        ("container-arrow", ColorIdx(Colors::Gray, 1)),
        ("container-base", ColorIdx(Colors::Black, 0)),
        ("container-border", ColorIdx(Colors::Gray, 1)),
        ("dialog-arrow", ColorIdx(Colors::Black, 0)),
        ("dialog-base", ColorIdx(Colors::Gray, 2)),
        ("dialog-border", ColorIdx(Colors::Black, 0)),
        ("disabled", ColorIdx(Colors::Gray, 3)),
        ("focus", ColorIdx(Colors::Primary, 1)),
        ("footer", ColorIdx(Colors::None, 0)),
        ("footer-fg", ColorIdx(Colors::DeepBlue, 0)),
        ("header", ColorIdx(Colors::None, 0)),
        ("header-fg", ColorIdx(Colors::DeepBlue, 0)),
        ("hover", ColorIdx(Colors::Secondary, 2)),
        ("input", ColorIdx(Colors::Gray, 2)),
        ("invalid", ColorIdx(Colors::Red, 1)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::White, 0)),
        ("menu-base", ColorIdx(Colors::Black, 1)),
        ("popup-arrow", ColorIdx(Colors::Gray, 3)),
        ("popup-base", ColorIdx(Colors::Gray, 0)),
        ("popup-border", ColorIdx(Colors::Gray, 3)),
        ("select", ColorIdx(Colors::Secondary, 0)),
        ("shadows", ColorIdx(Colors::None, 0)),
        ("status-base", ColorIdx(Colors::Black, 0)),
        ("text-focus", ColorIdx(Colors::Primary, 1)),
        ("text-select", ColorIdx(Colors::Secondary, 0)),
        ("title", ColorIdx(Colors::LimeGreen, 0)),
        ("title-fg", ColorIdx(Colors::TextDark, 3)),
    ],
};

