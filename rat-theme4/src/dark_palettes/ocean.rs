use crate::{ColorIdx, Colors, Palette};

/// Ocean
/// My take on an ocean theme.
const DARKNESS: u8 = 63;

pub const OCEAN: Palette = Palette {
    name: "Ocean", 

    color: [
        Palette::interpolate2(0xe5e5dd, 0xf2f2ee, 0x0, 0x0),
        Palette::interpolate2(0x030305, 0x0c092c, 0x0, 0x0),
        Palette::interpolate(0xff8d3c, 0xffbf3c, DARKNESS),
        Palette::interpolate(0x2b4779, 0x6688cc, DARKNESS),
        Palette::interpolate(0xe5e5dd, 0xf2f2ee, DARKNESS),
        Palette::interpolate(0x030305, 0x0c092c, DARKNESS),
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
    aliased: &[
        ("button-base", ColorIdx(Colors::Gray, 0)),
        ("container-arrow", ColorIdx(Colors::BlueGreen, 0)),
        ("container-base", ColorIdx(Colors::Black, 3)),
        ("container-border", ColorIdx(Colors::Gray, 0)),
        ("dialog-arrow", ColorIdx(Colors::TextDark, 3)),
        ("dialog-base", ColorIdx(Colors::Gray, 2)),
        ("dialog-border", ColorIdx(Colors::TextDark, 3)),
        ("disabled", ColorIdx(Colors::Gray, 3)),
        ("focus", ColorIdx(Colors::Primary, 1)),
        ("footer", ColorIdx(Colors::None, 0)),
        ("footer-fg", ColorIdx(Colors::Blue, 0)),
        ("header", ColorIdx(Colors::None, 0)),
        ("header-fg", ColorIdx(Colors::Blue, 0)),
        ("hover", ColorIdx(Colors::Primary, 2)),
        ("input", ColorIdx(Colors::Gray, 3)),
        ("invalid", ColorIdx(Colors::Red, 0)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::White, 0)),
        ("menu-base", ColorIdx(Colors::Black, 3)),
        ("month-header-fg", ColorIdx(Colors::None, 0)),
        ("popup-arrow", ColorIdx(Colors::TextDark, 3)),
        ("popup-base", ColorIdx(Colors::Gray, 3)),
        ("popup-border", ColorIdx(Colors::TextDark, 3)),
        ("select", ColorIdx(Colors::Secondary, 1)),
        ("shadows", ColorIdx(Colors::Black, 3)),
        ("status-base", ColorIdx(Colors::Black, 3)),
        ("text-focus", ColorIdx(Colors::Primary, 1)),
        ("text-select", ColorIdx(Colors::Secondary, 1)),
        ("title", ColorIdx(Colors::Yellow, 0)),
        ("title-fg", ColorIdx(Colors::TextDark, 3)),
        ("week-header-fg", ColorIdx(Colors::Gray, 2)),
    ],
};

