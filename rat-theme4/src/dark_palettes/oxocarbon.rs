use crate::{ColorIdx, Colors, Palette};

/// OxoCarbon
const DARKNESS: u8 = 63;

pub const OXOCARBON: Palette = Palette {
    name: "OxoCarbon", 

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
    aliased: &[
        ("button-base", ColorIdx(Colors::Gray, 0)),
        ("container-arrow", ColorIdx(Colors::Gray, 1)),
        ("container-base", ColorIdx(Colors::Black, 2)),
        ("container-border", ColorIdx(Colors::Gray, 1)),
        ("dialog-arrow", ColorIdx(Colors::White, 0)),
        ("dialog-base", ColorIdx(Colors::Gray, 2)),
        ("dialog-border", ColorIdx(Colors::White, 0)),
        ("disabled", ColorIdx(Colors::Gray, 3)),
        ("focus", ColorIdx(Colors::Primary, 0)),
        ("footer", ColorIdx(Colors::None, 0)),
        ("footer-fg", ColorIdx(Colors::DeepBlue, 0)),
        ("header", ColorIdx(Colors::None, 0)),
        ("header-fg", ColorIdx(Colors::DeepBlue, 0)),
        ("hover", ColorIdx(Colors::Secondary, 0)),
        ("input", ColorIdx(Colors::Gray, 3)),
        ("invalid", ColorIdx(Colors::Red, 1)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::White, 0)),
        ("menu-base", ColorIdx(Colors::Black, 1)),
        ("popup-arrow", ColorIdx(Colors::White, 0)),
        ("popup-base", ColorIdx(Colors::Gray, 0)),
        ("popup-border", ColorIdx(Colors::White, 0)),
        ("select", ColorIdx(Colors::Secondary, 0)),
        ("shadows", ColorIdx(Colors::Black, 0)),
        ("status-base", ColorIdx(Colors::Black, 1)),
        ("text-focus", ColorIdx(Colors::Primary, 1)),
        ("text-select", ColorIdx(Colors::Secondary, 1)),
        ("title", ColorIdx(Colors::Red, 0)),
        ("title-fg", ColorIdx(Colors::TextLight, 0)),
    ],
};

