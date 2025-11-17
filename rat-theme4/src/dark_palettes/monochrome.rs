use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Monochrome
const DARKNESS: u8 = 63;

pub const MONOCHROME: Palette = {
    let mut p = Palette {
        name: "Monochrome", 

        color: [
            Palette::interpolate2(0xd8dee9, 0xd8dee9, 0x0, 0x0),
            Palette::interpolate2(0x101010, 0x202020, 0x0, 0x0),
            Palette::interpolate(0x708187, 0x9ab2ba, DARKNESS),
            Palette::interpolate(0x424242, 0x677777, DARKNESS),
            Palette::interpolate(0xd8dee9, 0xd7dde8, DARKNESS),
            Palette::interpolate(0x1a1a1a, 0x202020, DARKNESS),
            Palette::interpolate(0x424242, 0x677777, DARKNESS),
            Palette::interpolate(0xec8989, 0xec8989, DARKNESS),
            Palette::interpolate(0xefb6a0, 0xefb6a0, DARKNESS),
            Palette::interpolate(0xffe6b5, 0xffe6b5, DARKNESS),
            Palette::interpolate(0xeff6ab, 0xeff6ab, DARKNESS),
            Palette::interpolate(0xc9d36a, 0xc9d36a, DARKNESS),
            Palette::interpolate(0x6484a4, 0x6484a4, DARKNESS),
            Palette::interpolate(0x9aafe6, 0x9aafe6, DARKNESS),
            Palette::interpolate(0x8abae1, 0x8abae1, DARKNESS),
            Palette::interpolate(0xa5c6e1, 0xa5c6e1, DARKNESS),
            Palette::interpolate(0xdb9fe9, 0xdb9fe9, DARKNESS),
            Palette::interpolate(0xda838b, 0xda838b, DARKNESS),
            Palette::interpolate(0xeca8a8, 0xeca8a8, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Secondary as usize][2];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::LimeGreen as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::DeepBlue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::DeepBlue as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][0];

    p
};
