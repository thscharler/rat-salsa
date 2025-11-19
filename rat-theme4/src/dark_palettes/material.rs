use crate::{ColorIdx, Colors, Palette};

/// Material
/// Credits to original theme https://github.com/marko-cerovac/material.nvim for existing
/// -
const DARKNESS: u8 = 63;

pub const MATERIAL: Palette = Palette {
    name: "Material", 

    color: [
        Palette::interpolate2(0xeeffff, 0xeeffff, 0x0, 0x0),
        Palette::interpolate2(0x191919, 0x191919, 0x0, 0x0),
        Palette::interpolate(0xcca055, 0xffcb6b, DARKNESS),
        Palette::interpolate(0x96b26c, 0xc3e88d, DARKNESS),
        Palette::interpolate(0xeeffff, 0xeeffff, DARKNESS),
        Palette::interpolate(0x191919, 0x292929, DARKNESS),
        Palette::interpolate(0x4a4a4a, 0x6b6b6b, DARKNESS),
        Palette::interpolate(0xf07178, 0xf07178, DARKNESS),
        Palette::interpolate(0xf78c6c, 0xf78c6c, DARKNESS),
        Palette::interpolate(0xffcb6b, 0xffcb6b, DARKNESS),
        Palette::interpolate(0xc3e88d, 0xc3e88d, DARKNESS),
        Palette::interpolate(0xc3e88d, 0xc3e88d, DARKNESS),
        Palette::interpolate(0xabcf76, 0xabcf76, DARKNESS),
        Palette::interpolate(0x89ddff, 0x89ddff, DARKNESS),
        Palette::interpolate(0x82aaff, 0x82aaff, DARKNESS),
        Palette::interpolate(0x6e98eb, 0x6e98eb, DARKNESS),
        Palette::interpolate(0xb480d6, 0xb480d6, DARKNESS),
        Palette::interpolate(0xda70ca, 0xda70ca, DARKNESS),
        Palette::interpolate(0xffadff, 0xffadff, DARKNESS),
    ],
    // must be sorted!
    aliased: &[
        ("button-base.bg", ColorIdx(Colors::Gray, 0)),
        ("container-arrow.fg", ColorIdx(Colors::None, 0)),
        ("container-base.bg", ColorIdx(Colors::Black, 1)),
        ("container-border.fg", ColorIdx(Colors::None, 0)),
        ("dialog-arrow.fg", ColorIdx(Colors::None, 0)),
        ("dialog-base.bg", ColorIdx(Colors::Gray, 2)),
        ("dialog-border.fg", ColorIdx(Colors::None, 0)),
        ("disabled.bg", ColorIdx(Colors::Gray, 3)),
        ("document", ColorIdx(Colors::None, 0)),
        ("focus.bg", ColorIdx(Colors::Primary, 1)),
        ("footer.bg", ColorIdx(Colors::None, 0)),
        ("footer.fg", ColorIdx(Colors::Blue, 0)),
        ("header.bg", ColorIdx(Colors::None, 0)),
        ("header.fg", ColorIdx(Colors::Blue, 0)),
        ("heading-1", ColorIdx(Colors::None, 0)),
        ("heading-2", ColorIdx(Colors::None, 0)),
        ("hover.bg", ColorIdx(Colors::Gray, 2)),
        ("input.bg", ColorIdx(Colors::Gray, 2)),
        ("invalid.bg", ColorIdx(Colors::Red, 1)),
        ("key-binding.bg", ColorIdx(Colors::BlueGreen, 0)),
        ("label.fg", ColorIdx(Colors::White, 0)),
        ("menu-base.bg", ColorIdx(Colors::Black, 1)),
        ("month-header.fg", ColorIdx(Colors::None, 0)),
        ("popup-arrow.fg", ColorIdx(Colors::None, 0)),
        ("popup-base.bg", ColorIdx(Colors::Gray, 0)),
        ("popup-border.fg", ColorIdx(Colors::None, 0)),
        ("select.bg", ColorIdx(Colors::Secondary, 1)),
        ("shadow.bg", ColorIdx(Colors::Black, 0)),
        ("status-base.bg", ColorIdx(Colors::Black, 1)),
        ("sub-form", ColorIdx(Colors::None, 0)),
        ("text-focus.bg", ColorIdx(Colors::Primary, 1)),
        ("text-select.bg", ColorIdx(Colors::Secondary, 1)),
        ("title.bg", ColorIdx(Colors::Red, 0)),
        ("title.fg", ColorIdx(Colors::TextLight, 0)),
        ("week-header.fg", ColorIdx(Colors::Gray, 3)),
    ],
};

