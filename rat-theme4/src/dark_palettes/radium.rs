use crate::{ColorIdx, Colors, Palette};

/// Radium
/// An adaption of nvchad's radium theme.
/// -- credits to original radium theme from <https://github.com/dharmx>
const DARKNESS: u8 = 63;

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
        ("button-base.bg", ColorIdx(Colors::White, 3)),
        ("container-arrow.fg", ColorIdx(Colors::None, 0)),
        ("container-base.bg", ColorIdx(Colors::Black, 3)),
        ("container-border.fg", ColorIdx(Colors::None, 0)),
        ("dialog-arrow.fg", ColorIdx(Colors::None, 0)),
        ("dialog-base.bg", ColorIdx(Colors::Gray, 2)),
        ("dialog-border.fg", ColorIdx(Colors::None, 0)),
        ("disabled.bg", ColorIdx(Colors::Gray, 0)),
        ("focus.bg", ColorIdx(Colors::Primary, 3)),
        ("footer.bg", ColorIdx(Colors::Black, 0)),
        ("footer.fg", ColorIdx(Colors::LimeGreen, 0)),
        ("header.bg", ColorIdx(Colors::Black, 0)),
        ("header.fg", ColorIdx(Colors::LimeGreen, 0)),
        ("hover.bg", ColorIdx(Colors::Green, 3)),
        ("input.bg", ColorIdx(Colors::Gray, 3)),
        ("invalid.bg", ColorIdx(Colors::Red, 0)),
        ("key-binding.bg", ColorIdx(Colors::BlueGreen, 0)),
        ("label.fg", ColorIdx(Colors::TextLight, 0)),
        ("menu-base.bg", ColorIdx(Colors::Black, 3)),
        ("month-header.fg", ColorIdx(Colors::None, 0)),
        ("popup-arrow.fg", ColorIdx(Colors::None, 0)),
        ("popup-base.bg", ColorIdx(Colors::Gray, 2)),
        ("popup-border.fg", ColorIdx(Colors::None, 0)),
        ("select.bg", ColorIdx(Colors::Secondary, 3)),
        ("shadow.bg", ColorIdx(Colors::Black, 0)),
        ("status-base.bg", ColorIdx(Colors::Black, 3)),
        ("text-focus.bg", ColorIdx(Colors::Primary, 3)),
        ("text-select.bg", ColorIdx(Colors::Secondary, 3)),
        ("title.bg", ColorIdx(Colors::Secondary, 0)),
        ("title.fg", ColorIdx(Colors::TextLight, 0)),
        ("week-header.fg", ColorIdx(Colors::BlueGreen, 0)),
    ],
};
