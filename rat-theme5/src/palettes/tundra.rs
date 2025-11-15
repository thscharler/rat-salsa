use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Tundra
const DARKNESS: u8 = 64;

pub const TUNDRA: Palette = {
    let mut p = Palette {
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
            Palette::interpolate(0xffc9c9, 0xff8e8e, DARKNESS),
            Palette::interpolate(0xfffcad, 0xfecdd3, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][3];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::Blue as usize][2];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::Blue as usize][2];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Shadow as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][2];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][3];

    p
};
