use crate::{ColorIdx, Colors, Palette};

/// Nord
/// Credits to original https://github.com/arcticicestudio/nord-vim
/// 
const DARKNESS: u8 = 128;

pub const NORD: Palette = Palette {
    name: "Nord", 

    color: [
        Palette::interpolate2(0xe5e9f0, 0xe5e9f0, 0x0, 0x0),
        Palette::interpolate2(0x2e3440, 0x2e3440, 0x0, 0x0),
        Palette::interpolate(0xd8dee9, 0xd8dee9, DARKNESS),
        Palette::interpolate(0x9ab3d3, 0x9ab3d3, DARKNESS),
        Palette::interpolate(0xd8dee9, 0xd8dee9, DARKNESS),
        Palette::interpolate(0x2e3440, 0x2e3440, DARKNESS),
        Palette::interpolate(0x434c5e, 0x434c5e, DARKNESS),
        Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
        Palette::interpolate(0xd08770, 0xd08770, DARKNESS),
        Palette::interpolate(0xeadbbe, 0xeadbbe, DARKNESS),
        Palette::interpolate(0xa3be8c, 0xa3be8c, DARKNESS),
        Palette::interpolate(0x8fbcbb, 0x8fbcbb, DARKNESS),
        Palette::interpolate(0x88c0d0, 0x88c0d0, DARKNESS),
        Palette::interpolate(0x88c0d0, 0x88c0d0, DARKNESS),
        Palette::interpolate(0x81a1c1, 0x81a1c1, DARKNESS),
        Palette::interpolate(0x5e81ac, 0x5e81ac, DARKNESS),
        Palette::interpolate(0xb48ead, 0xb48ead, DARKNESS),
        Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
        Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base", ColorIdx(Colors::Gray, 0)),
        ("container-arrow", ColorIdx(Colors::Gray, 0)),
        ("container-base", ColorIdx(Colors::Black, 0)),
        ("container-border", ColorIdx(Colors::Gray, 0)),
        ("dialog-arrow", ColorIdx(Colors::Gray, 0)),
        ("dialog-base", ColorIdx(Colors::Blue, 0)),
        ("dialog-border", ColorIdx(Colors::Gray, 0)),
        ("disabled", ColorIdx(Colors::Gray, 0)),
        ("focus", ColorIdx(Colors::Primary, 0)),
        ("footer", ColorIdx(Colors::None, 0)),
        ("footer-fg", ColorIdx(Colors::Blue, 0)),
        ("header", ColorIdx(Colors::None, 0)),
        ("header-fg", ColorIdx(Colors::Blue, 0)),
        ("hover", ColorIdx(Colors::Primary, 0)),
        ("input", ColorIdx(Colors::Gray, 0)),
        ("invalid", ColorIdx(Colors::Red, 1)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::White, 0)),
        ("menu-base", ColorIdx(Colors::Black, 0)),
        ("month-header-fg", ColorIdx(Colors::None, 0)),
        ("popup-arrow", ColorIdx(Colors::Gray, 0)),
        ("popup-base", ColorIdx(Colors::White, 0)),
        ("popup-border", ColorIdx(Colors::Gray, 0)),
        ("select", ColorIdx(Colors::Secondary, 0)),
        ("shadows", ColorIdx(Colors::TextDark, 0)),
        ("status-base", ColorIdx(Colors::Black, 0)),
        ("text-focus", ColorIdx(Colors::Primary, 0)),
        ("text-select", ColorIdx(Colors::Secondary, 0)),
        ("title", ColorIdx(Colors::Red, 0)),
        ("title-fg", ColorIdx(Colors::TextLight, 0)),
        ("week-header-fg", ColorIdx(Colors::Yellow, 0)),
    ],
};

