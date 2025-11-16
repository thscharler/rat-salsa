use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Radium
/// An adaption of nvchad's radium theme.
/// -- credits to original radium theme from <https://github.com/dharmx>
const DARKNESS: u8 = 63;

pub const RADIUM: Palette = {
    let mut p = Palette {
        name: "Radium",

        color: [
            Palette::interpolate2(0xc4c4c5, 0xaaaaaa, 0x0, 0x0),
            Palette::interpolate2(0x292c30, 0x292c30, 0x0, 0x0),
            Palette::interpolate(0x1fa372, 0x1b875d, DARKNESS),
            Palette::interpolate(0x9759b5, 0x7f4c99, DARKNESS),
            Palette::interpolate(0xc4c4c5, 0xaaaaaa, DARKNESS),
            Palette::interpolate(0x101317, 0x050607, DARKNESS),
            Palette::interpolate(0x3e4145, 0x27282b, DARKNESS),
            Palette::interpolate(0xf64b4b, 0xc13c3c, DARKNESS),
            Palette::interpolate(0xe6723d, 0xb25730, DARKNESS),
            Palette::interpolate(0xffc424, 0xcc9a1e, DARKNESS),
            Palette::interpolate(0x3cb77a, 0x2a8256, DARKNESS),
            Palette::interpolate(0x3cb77a, 0x2a8256, DARKNESS),
            Palette::interpolate(0x3cb77a, 0x2a8256, DARKNESS),
            Palette::interpolate(0x2b9fa5, 0x1e6d70, DARKNESS),
            Palette::interpolate(0x2b72b1, 0x1f517c, DARKNESS),
            Palette::interpolate(0x3476af, 0x24537a, DARKNESS),
            Palette::interpolate(0x9759b5, 0x6a3f7f, DARKNESS),
            Palette::interpolate(0xe03838, 0xaa2a2a, DARKNESS),
            Palette::interpolate(0xf43f3f, 0xbf3131, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][3];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::Cyan as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::Cyan as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::White as usize][3];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][0];

    p
};
