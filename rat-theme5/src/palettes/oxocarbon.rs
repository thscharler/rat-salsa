use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// OxoCarbon
const DARKNESS: u8 = 63;

pub const OXOCARBON: Palette = {
    let mut p = Palette {
        name: "OxoCarbon", 

        color: [
            Palette::interpolate2(0xf2f4f8, 0xf9fbff, 0x0, 0x0),
            Palette::interpolate2(0x0f0f0f, 0x202020, 0x0, 0x0),
            Palette::interpolate(0x78a9ff, 0x78a9ff, DARKNESS),
            Palette::interpolate(0xb5e8e0, 0xb5e8e0, DARKNESS),
            Palette::interpolate(0xdde1e6, 0xffffff, DARKNESS),
            Palette::interpolate(0x0f0f0f, 0x202020, DARKNESS),
            Palette::interpolate(0x464646, 0x5f5f5f, DARKNESS),
            Palette::interpolate(0xee5396, 0xee5396, DARKNESS),
            Palette::interpolate(0xf8bd96, 0xf8bd96, DARKNESS),
            Palette::interpolate(0xfae3b0, 0xfae3b0, DARKNESS),
            Palette::interpolate(0x08bdba, 0x08bdba, DARKNESS),
            Palette::interpolate(0x42be65, 0x42be65, DARKNESS),
            Palette::interpolate(0xb5e8e0, 0xb5e8e0, DARKNESS),
            Palette::interpolate(0x3ddbd9, 0x3ddbd9, DARKNESS),
            Palette::interpolate(0x33b1ff, 0x33b1ff, DARKNESS),
            Palette::interpolate(0x78a9ff, 0x78a9ff, DARKNESS),
            Palette::interpolate(0xbe95ff, 0xbe95ff, DARKNESS),
            Palette::interpolate(0xd0a9e5, 0xd0a9e5, DARKNESS),
            Palette::interpolate(0xff7eb6, 0xff77b4, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
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
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::White as usize][0];

    p
};
