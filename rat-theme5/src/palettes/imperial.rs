use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Imperial
///
/// Uses purple and gold for primary/secondary.
/// Other colors are bright, strong and slightly smudged.
const DARKNESS: u8 = 63;

pub const IMPERIAL: Palette = init_palette();

const fn init_palette() -> Palette {
    let mut p = Palette {
        name: "Imperial",

        color: [
            Palette::interpolate2(0xdedfe3, 0xf6f6f3, 0x0, 0x0),
            Palette::interpolate2(0x0f1014, 0x2a2b37, 0x0, 0x0),
            Palette::interpolate(0x3d0070, 0x8900f9, DARKNESS),
            Palette::interpolate(0x726100, 0xe0c200, DARKNESS),
            Palette::interpolate(0xdedfe3, 0xf6f6f3, DARKNESS),
            Palette::interpolate(0x0f1014, 0x2a2b37, DARKNESS),
            Palette::interpolate(0x3b3d4e, 0x6e7291, DARKNESS),
            Palette::interpolate(0x601414, 0xd22d2d, DARKNESS),
            Palette::interpolate(0x5e3913, 0xd3802c, DARKNESS),
            Palette::interpolate(0x756600, 0xd6b900, DARKNESS),
            Palette::interpolate(0x3c5e17, 0x80ce31, DARKNESS),
            Palette::interpolate(0x186218, 0x32c932, DARKNESS),
            Palette::interpolate(0x1b5944, 0x3abc93, DARKNESS),
            Palette::interpolate(0x1b5184, 0x2bcece, DARKNESS),
            Palette::interpolate(0x234668, 0x2b81d4, DARKNESS),
            Palette::interpolate(0x202083, 0x3232cc, DARKNESS),
            Palette::interpolate(0x4b0089, 0x8c00fd, DARKNESS),
            Palette::interpolate(0x4f1b4f, 0xbd42bd, DARKNESS),
            Palette::interpolate(0x47101d, 0xc33c5b, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][2];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][3];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Orange as usize][3];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Blue as usize][1];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Blue as usize][1];
    p.color_ext[ColorsExt::Shadow as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::ContainerBorder as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::ContainerArrow as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::PopupBorder as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupArrow as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::DialogBorder as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogArrow as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][2];

    p
}
