use crate::{ColorIdx, Colors, Palette};

/// Radium
/// An adaption of nvchad's radium theme.
/// -- credits to original radium theme from <https://github.com/dharmx>
const DARKNESS: u8 = 96;

pub const RADIUM: Palette = Palette {
    name: "Radium", 

    color: [
        Palette::interpolate2(0xd4d4d5, 0xffffff, 0x0, 0x0),
        Palette::interpolate2(0x101317, 0x191d22, 0x0, 0x0),
        Palette::interpolate(0x2ba578, 0x37d99e, DARKNESS),
        Palette::interpolate(0x866696, 0xb68acb, DARKNESS),
        Palette::interpolate(0xc4c4c5, 0xaaaaaa, DARKNESS),
        Palette::interpolate(0x101317, 0x191d22, DARKNESS),
        Palette::interpolate(0x3e4145, 0x525559, DARKNESS),
        Palette::interpolate(0xf87070, 0xf87070, DARKNESS),
        Palette::interpolate(0xf0a988, 0xf0a988, DARKNESS),
        Palette::interpolate(0xffe59e, 0xffe59e, DARKNESS),
        Palette::interpolate(0x79dcaa, 0x79dcaa, DARKNESS),
        Palette::interpolate(0x37d99e, 0x37d99e, DARKNESS),
        Palette::interpolate(0x63b3ad, 0x63b3ad, DARKNESS),
        Palette::interpolate(0x50cad2, 0x50cad2, DARKNESS),
        Palette::interpolate(0x7ab0df, 0x7ab0df, DARKNESS),
        Palette::interpolate(0x87bdec, 0x87bdec, DARKNESS),
        Palette::interpolate(0xb68acb, 0xb284c9, DARKNESS),
        Palette::interpolate(0xffa7a7, 0xffb7b7, DARKNESS),
        Palette::interpolate(0xff8e8e, 0xff8e8e, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base", ColorIdx(Colors::White, 3)),
        ("container-arrow", ColorIdx(Colors::Gray, 2)),
        ("container-base", ColorIdx(Colors::Black, 3)),
        ("container-border", ColorIdx(Colors::Gray, 0)),
        ("dialog-arrow", ColorIdx(Colors::Black, 0)),
        ("dialog-base", ColorIdx(Colors::Gray, 2)),
        ("dialog-border", ColorIdx(Colors::Black, 0)),
        ("disabled", ColorIdx(Colors::Gray, 0)),
        ("focus", ColorIdx(Colors::Primary, 3)),
        ("footer", ColorIdx(Colors::Black, 0)),
        ("footer-fg", ColorIdx(Colors::LimeGreen, 0)),
        ("header", ColorIdx(Colors::Black, 0)),
        ("header-fg", ColorIdx(Colors::LimeGreen, 0)),
        ("hover", ColorIdx(Colors::Green, 3)),
        ("input", ColorIdx(Colors::Gray, 3)),
        ("invalid", ColorIdx(Colors::Red, 0)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::TextLight, 0)),
        ("menu-base", ColorIdx(Colors::Black, 3)),
        ("popup-arrow", ColorIdx(Colors::Black, 0)),
        ("popup-base", ColorIdx(Colors::Gray, 2)),
        ("popup-border", ColorIdx(Colors::Black, 0)),
        ("select", ColorIdx(Colors::Secondary, 3)),
        ("shadows", ColorIdx(Colors::Black, 0)),
        ("status-base", ColorIdx(Colors::Black, 3)),
        ("text-focus", ColorIdx(Colors::Primary, 3)),
        ("text-select", ColorIdx(Colors::Secondary, 3)),
        ("title", ColorIdx(Colors::Secondary, 0)),
        ("title-fg", ColorIdx(Colors::TextLight, 0)),
    ],
};

