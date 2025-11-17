use crate::{ColorIdx, Colors, Palette};

/// Monekai
const DARKNESS: u8 = 63;

pub const MONEKAI: Palette = Palette {
    name: "Monekai", 

    color: [
        Palette::interpolate2(0xf5f4f1, 0xfffefc, 0x0, 0x0),
        Palette::interpolate2(0x272822, 0x464741, 0x0, 0x0),
        Palette::interpolate(0xc11f5a, 0xf92672, DARKNESS),
        Palette::interpolate(0x5c7289, 0x81a1c1, DARKNESS),
        Palette::interpolate(0xf5f4f1, 0xf5f4f1, DARKNESS),
        Palette::interpolate(0x22231d, 0x2f302a, DARKNESS),
        Palette::interpolate(0x4d4e48, 0x64655f, DARKNESS),
        Palette::interpolate(0xe36d76, 0xe36d76, DARKNESS),
        Palette::interpolate(0xd39467, 0xd39467, DARKNESS),
        Palette::interpolate(0xe6c181, 0xe6c181, DARKNESS),
        Palette::interpolate(0x96c367, 0x96c367, DARKNESS),
        Palette::interpolate(0x96c367, 0x96c367, DARKNESS),
        Palette::interpolate(0x34bfd0, 0x34bfd0, DARKNESS),
        Palette::interpolate(0x41afef, 0x41afef, DARKNESS),
        Palette::interpolate(0x51afef, 0x51afef, DARKNESS),
        Palette::interpolate(0x81a1c1, 0x81a1c1, DARKNESS),
        Palette::interpolate(0xae81ff, 0xae81ff, DARKNESS),
        Palette::interpolate(0xf92672, 0xf72270, DARKNESS),
        Palette::interpolate(0xf98385, 0xf98381, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base", ColorIdx(Colors::Gray, 0)),
        ("container-arrow", ColorIdx(Colors::Gray, 1)),
        ("container-base", ColorIdx(Colors::Black, 0)),
        ("container-border", ColorIdx(Colors::Gray, 1)),
        ("dialog-arrow", ColorIdx(Colors::Black, 3)),
        ("dialog-base", ColorIdx(Colors::Gray, 1)),
        ("dialog-border", ColorIdx(Colors::Black, 3)),
        ("disabled", ColorIdx(Colors::Gray, 0)),
        ("focus", ColorIdx(Colors::Primary, 1)),
        ("footer", ColorIdx(Colors::None, 0)),
        ("footer-fg", ColorIdx(Colors::DeepBlue, 0)),
        ("header", ColorIdx(Colors::None, 0)),
        ("header-fg", ColorIdx(Colors::DeepBlue, 0)),
        ("hover", ColorIdx(Colors::Purple, 0)),
        ("input", ColorIdx(Colors::Gray, 3)),
        ("invalid", ColorIdx(Colors::RedPink, 0)),
        ("key-binding", ColorIdx(Colors::BlueGreen, 0)),
        ("label", ColorIdx(Colors::White, 0)),
        ("menu-base", ColorIdx(Colors::Black, 0)),
        ("popup-arrow", ColorIdx(Colors::TextDark, 0)),
        ("popup-base", ColorIdx(Colors::Gray, 1)),
        ("popup-border", ColorIdx(Colors::TextDark, 0)),
        ("select", ColorIdx(Colors::Secondary, 3)),
        ("shadows", ColorIdx(Colors::Black, 0)),
        ("status-base", ColorIdx(Colors::Black, 0)),
        ("text-focus", ColorIdx(Colors::Primary, 1)),
        ("text-select", ColorIdx(Colors::Secondary, 1)),
        ("title", ColorIdx(Colors::None, 0)),
        ("title-fg", ColorIdx(Colors::Magenta, 0)),
    ],
};

