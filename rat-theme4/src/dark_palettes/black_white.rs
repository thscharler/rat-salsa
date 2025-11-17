use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Black&White
const DARKNESS: u8 = 63;

pub const BLACK_WHITE: Palette = {
    let mut p = Palette {
        name: "Black&White", 

        color: [
            Palette::interpolate2(0xffffff, 0xffffff, 0x0, 0x0),
            Palette::interpolate2(0x000000, 0x000000, 0x0, 0x0),
            Palette::interpolate(0xffffff, 0x000000, DARKNESS),
            Palette::interpolate(0xffffff, 0x000000, DARKNESS),
            Palette::interpolate(0xffffff, 0xffffff, DARKNESS),
            Palette::interpolate(0x000000, 0x000000, DARKNESS),
            Palette::interpolate(0xffffff, 0x000000, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
            Palette::interpolate(0xffffff, 0x7f7f7f, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][2];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][3];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::Blue as usize][3];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::Blue as usize][3];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][0];

    p
};
