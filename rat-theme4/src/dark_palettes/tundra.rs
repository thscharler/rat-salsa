use crate::{ColorIdx, Colors, Palette};

/// Tundra
/// An adaption of nvchad's tundra theme.
/// -- Thanks to original theme for existing <https://github.com/sam4llis/nvim-tundra>
const DARKNESS: u8 = 63;

pub const TUNDRA: Palette = Palette {
    name: "Tundra",

    color: [
        Palette::interpolate2(0xe6eaf2, 0xffffff, 0x0, 0x0),
        Palette::interpolate2(0x0b1221, 0x1a2130, 0x0, 0x0),
        Palette::interpolate(0xe6eaf2, 0xffffff, DARKNESS),
        Palette::interpolate(0xa8bbd4, 0x719bd3, DARKNESS),
        Palette::interpolate(0xe6eaf2, 0xffffff, DARKNESS),
        Palette::interpolate(0x0b1221, 0x1a2130, DARKNESS),
        Palette::interpolate(0x3e4554, 0x5f6675, DARKNESS),
        Palette::interpolate(0xfccaca, 0xfca5a5, DARKNESS),
        Palette::interpolate(0xfad9c5, 0xfbc19d, DARKNESS),
        Palette::interpolate(0xe8d7b7, 0xe8d4b0, DARKNESS),
        Palette::interpolate(0xbce8b7, 0xb5e8b0, DARKNESS),
        Palette::interpolate(0xbce8b7, 0xb5e8b0, DARKNESS),
        Palette::interpolate(0xa8bbd4, 0x719bd3, DARKNESS),
        Palette::interpolate(0xc8eafc, 0xbae6fd, DARKNESS),
        Palette::interpolate(0xc7d0fc, 0xa5b4fc, DARKNESS),
        Palette::interpolate(0xbfcaf2, 0x9baaf2, DARKNESS),
        Palette::interpolate(0xb7abd9, 0xb3a6da, DARKNESS),
        Palette::interpolate(0xffc9c9, 0xf98b8b, DARKNESS),
        Palette::interpolate(0xfffcad, 0xfecdd3, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base.bg", ColorIdx(Colors::Gray, 0)),
        ("container-arrow.fg", ColorIdx(Colors::None, 0)),
        ("container-base.bg", ColorIdx(Colors::Black, 3)),
        ("container-border.fg", ColorIdx(Colors::None, 0)),
        ("dialog-arrow.fg", ColorIdx(Colors::None, 0)),
        ("dialog-base.bg", ColorIdx(Colors::Gray, 2)),
        ("dialog-border.fg", ColorIdx(Colors::None, 0)),
        ("disabled.bg", ColorIdx(Colors::Gray, 3)),
        ("focus.bg", ColorIdx(Colors::Primary, 1)),
        ("footer.bg", ColorIdx(Colors::None, 0)),
        ("footer.fg", ColorIdx(Colors::Blue, 3)),
        ("header.bg", ColorIdx(Colors::None, 0)),
        ("header.fg", ColorIdx(Colors::Blue, 3)),
        ("hover.bg", ColorIdx(Colors::Secondary, 0)),
        ("input.bg", ColorIdx(Colors::Gray, 3)),
        ("invalid.bg", ColorIdx(Colors::Red, 3)),
        ("key-binding.bg", ColorIdx(Colors::BlueGreen, 0)),
        ("label.fg", ColorIdx(Colors::White, 0)),
        ("menu-base.bg", ColorIdx(Colors::Black, 1)),
        ("month-header.fg", ColorIdx(Colors::None, 0)),
        ("popup-arrow.fg", ColorIdx(Colors::None, 0)),
        ("popup-base.bg", ColorIdx(Colors::White, 0)),
        ("popup-border.fg", ColorIdx(Colors::None, 0)),
        ("select.bg", ColorIdx(Colors::Secondary, 1)),
        ("shadow.bg", ColorIdx(Colors::TextDark, 0)),
        ("status-base.bg", ColorIdx(Colors::Black, 1)),
        ("text-focus.bg", ColorIdx(Colors::Primary, 1)),
        ("text-select.bg", ColorIdx(Colors::Secondary, 1)),
        ("title.bg", ColorIdx(Colors::Red, 0)),
        ("title.fg", ColorIdx(Colors::TextDark, 3)),
        ("week-header.fg", ColorIdx(Colors::BlueGreen, 0)),
    ],
};
