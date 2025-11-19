use crate::{ColorIdx, Colors, Palette};

/// Nord
/// Credits to original https://github.com/arcticicestudio/nord-vim
///
const DARKNESS: u8 = 63;

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
        ("button-base.bg", ColorIdx(Colors::Gray, 0)),
        ("container-arrow.fg", ColorIdx(Colors::None, 0)),
        ("container-base.bg", ColorIdx(Colors::Black, 0)),
        ("container-border.fg", ColorIdx(Colors::None, 0)),
        ("dialog-arrow.fg", ColorIdx(Colors::None, 0)),
        ("dialog-base.bg", ColorIdx(Colors::Blue, 0)),
        ("dialog-border.fg", ColorIdx(Colors::None, 0)),
        ("disabled.bg", ColorIdx(Colors::Gray, 0)),
        ("focus.bg", ColorIdx(Colors::Primary, 0)),
        ("footer.bg", ColorIdx(Colors::None, 0)),
        ("footer.fg", ColorIdx(Colors::Blue, 0)),
        ("header.bg", ColorIdx(Colors::None, 0)),
        ("header.fg", ColorIdx(Colors::Blue, 0)),
        ("hover.bg", ColorIdx(Colors::Primary, 0)),
        ("input.bg", ColorIdx(Colors::Gray, 0)),
        ("invalid.bg", ColorIdx(Colors::Red, 1)),
        ("key-binding.bg", ColorIdx(Colors::BlueGreen, 0)),
        ("label.fg", ColorIdx(Colors::White, 0)),
        ("menu-base.bg", ColorIdx(Colors::Black, 0)),
        ("month-header.fg", ColorIdx(Colors::None, 0)),
        ("popup-arrow.fg", ColorIdx(Colors::None, 0)),
        ("popup-base.bg", ColorIdx(Colors::White, 0)),
        ("popup-border.fg", ColorIdx(Colors::None, 0)),
        ("select.bg", ColorIdx(Colors::Secondary, 0)),
        ("shadow.bg", ColorIdx(Colors::TextDark, 0)),
        ("status-base.bg", ColorIdx(Colors::Black, 0)),
        ("text-focus.bg", ColorIdx(Colors::Primary, 0)),
        ("text-select.bg", ColorIdx(Colors::Secondary, 0)),
        ("title.bg", ColorIdx(Colors::Red, 0)),
        ("title.fg", ColorIdx(Colors::TextLight, 0)),
        ("week-header.fg", ColorIdx(Colors::Yellow, 0)),
    ],
};
