use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Nord
/// Credits to original https://github.com/arcticicestudio/nord-vim
///
const DARKNESS: u8 = 128;

pub const NORD: Palette = {
    let mut p = Palette {
        name: "Nord",

        color: [
            Palette::interpolate2(0xe5e9f0, 0xe5e9f0, 0x0, 0x0),
            Palette::interpolate2(0x2e3440, 0x2e3440, 0x0, 0x0),
            Palette::interpolate(0xd8dee9, 0xd8dee9, DARKNESS),
            Palette::interpolate(0x9ab3d3, 0x9ab3d3, DARKNESS),
            Palette::interpolate(0xd8dee9, 0xd8dee9, DARKNESS),
            Palette::interpolate(0x2e3440, 0x2e3440, DARKNESS),
            Palette::interpolate(0x434c5e, 0x434c5e, DARKNESS),
            Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
            Palette::interpolate(0xd08770, 0xd08770, DARKNESS),
            Palette::interpolate(0xeadbbe, 0xeadbbe, DARKNESS),
            Palette::interpolate(0xa3be8c, 0xa3be8c, DARKNESS),
            Palette::interpolate(0x8fbcbb, 0x8fbcbb, DARKNESS),
            Palette::interpolate(0x88c0d0, 0x88c0d0, DARKNESS),
            Palette::interpolate(0x88c0d0, 0x88c0d0, DARKNESS),
            Palette::interpolate(0x81a1c1, 0x81a1c1, DARKNESS),
            Palette::interpolate(0x5e81ac, 0x5e81ac, DARKNESS),
            Palette::interpolate(0xb48ead, 0xb48ead, DARKNESS),
            Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
            Palette::interpolate(0xbf616a, 0xbf616a, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][0];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Gray as usize][0];

    p
};
