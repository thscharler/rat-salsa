use crate::{ColorIdx, Colors, Palette};

/// Black&White
const DARKNESS: u8 = 63;

pub const BLACK_WHITE: Palette = Palette {
    name: "Black&White", 

    color: [
        Palette::interpolate2(0xffffff, 0xffffff, 0x0, 0x0),
        Palette::interpolate2(0x000000, 0x000000, 0x0, 0x0),
        Palette::interpolate(0xffffff, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0xffffff, DARKNESS),
        Palette::interpolate(0x000000, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0x000000, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base", ColorIdx(Colors::Gray, 1)),
        ("container-arrow", ColorIdx(Colors::Gray, 1)),
        ("container-base", ColorIdx(Colors::Black, 0)),
        ("container-border", ColorIdx(Colors::Gray, 0)),
        ("dialog-arrow", ColorIdx(Colors::Black, 0)),
        ("dialog-base", ColorIdx(Colors::Gray, 2)),
        ("dialog-border", ColorIdx(Colors::Gray, 0)),
        ("disabled", ColorIdx(Colors::Gray, 2)),
        ("focus", ColorIdx(Colors::Primary, 0)),
        ("footer", ColorIdx(Colors::None, 0)),
        ("footer-fg", ColorIdx(Colors::Blue, 3)),
        ("header", ColorIdx(Colors::None, 0)),
        ("header-fg", ColorIdx(Colors::Blue, 3)),
        ("hover", ColorIdx(Colors::Gray, 0)),
        ("input", ColorIdx(Colors::Gray, 1)),
        ("invalid", ColorIdx(Colors::Red, 1)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::White, 0)),
        ("menu-base", ColorIdx(Colors::Black, 0)),
        ("popup-arrow", ColorIdx(Colors::Gray, 2)),
        ("popup-base", ColorIdx(Colors::White, 0)),
        ("popup-border", ColorIdx(Colors::Gray, 2)),
        ("select", ColorIdx(Colors::Secondary, 2)),
        ("shadows", ColorIdx(Colors::Black, 0)),
        ("status-base", ColorIdx(Colors::Black, 0)),
        ("text-focus", ColorIdx(Colors::Primary, 0)),
        ("text-select", ColorIdx(Colors::Secondary, 1)),
        ("title", ColorIdx(Colors::Red, 3)),
        ("title-fg", ColorIdx(Colors::TextLight, 0)),
    ],
};

