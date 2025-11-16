use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// EverForest
const DARKNESS: u8 = 63;

pub const EVERFOREST: Palette = {
    let mut p = Palette {
        name: "EverForest", 

        color: [
            Palette::interpolate2(0xd8d4cb, 0xf4f0e6, 0x0, 0x0),
            Palette::interpolate2(0x090a09, 0x2c2d2a, 0x0, 0x0),
            Palette::interpolate(0xa7c080, 0xb8ce94, DARKNESS),
            Palette::interpolate(0x78b4ac, 0x8dc4bd, DARKNESS),
            Palette::interpolate(0xd3c6aa, 0xc4ac7b, DARKNESS),
            Palette::interpolate(0x272f35, 0x30383d, DARKNESS),
            Palette::interpolate(0x4e565c, 0x656d73, DARKNESS),
            Palette::interpolate(0xe67e80, 0xfc8f93, DARKNESS),
            Palette::interpolate(0xe69875, 0xf4ab8b, DARKNESS),
            Palette::interpolate(0xdbbc7f, 0xddc187, DARKNESS),
            Palette::interpolate(0xa7c080, 0xb8ce94, DARKNESS),
            Palette::interpolate(0x83c092, 0x83c092, DARKNESS),
            Palette::interpolate(0x69a59d, 0x77ada6, DARKNESS),
            Palette::interpolate(0x95d1c9, 0xafdbd5, DARKNESS),
            Palette::interpolate(0x7393b3, 0x8fa7bf, DARKNESS),
            Palette::interpolate(0x78b4ac, 0x8dc4bd, DARKNESS),
            Palette::interpolate(0xd699b6, 0xe5b7cd, DARKNESS),
            Palette::interpolate(0xff75a0, 0xf2608e, DARKNESS),
            Palette::interpolate(0xce8196, 0xbf6b82, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::DeepBlue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::DeepBlue as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::Black as usize][0];
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
