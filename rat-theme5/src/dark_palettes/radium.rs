use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Radium
/// An adaption of nvchad's radium theme.
/// -- credits to original radium theme from <https://github.com/dharmx>
const DARKNESS: u8 = 96;

pub const RADIUM: Palette = {
    let mut p = Palette {
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
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][3];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][3];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Green as usize][3];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::LimeGreen as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::LimeGreen as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][3];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][3];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::White as usize][3];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][0];

    p
};
