use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Reds
const DARKNESS: u8 = 63;

pub const REDS: Palette = {
    let mut p = Palette {
        name: "Reds", 

        color: [
            Palette::interpolate2(0xcfafaf, 0xe2cfcf, 0x0, 0x0),
            Palette::interpolate2(0x160d0d, 0x3a2323, 0x0, 0x0),
            Palette::interpolate(0xa50000, 0xa55454, DARKNESS),
            Palette::interpolate(0xa52300, 0xa56654, DARKNESS),
            Palette::interpolate(0xcfafaf, 0xe2cfcf, DARKNESS),
            Palette::interpolate(0x160d0d, 0x3a2323, DARKNESS),
            Palette::interpolate(0x6f4343, 0xbd8f8f, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
            Palette::interpolate(0xff7f7f, 0xffcccc, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][3];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::Blue as usize][2];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::Blue as usize][2];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][1];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][3];

    p
};
