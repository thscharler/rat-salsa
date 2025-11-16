use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Solarized
/// credit https://github.com/altercation/solarized/tree/master/vim-colors-solarized
const DARKNESS: u8 = 64;

pub const SOLARIZED: Palette = {
    let mut p = Palette {
        name: "Solarized", 

        color: [
            Palette::interpolate2(0xeee8d5, 0xfdf6e3, 0x0, 0x0),
            Palette::interpolate2(0x002b36, 0x073642, 0x0, 0x0),
            Palette::interpolate(0xcb4b16, 0xcb4b16, DARKNESS),
            Palette::interpolate(0x586e75, 0x839496, DARKNESS),
            Palette::interpolate(0xeee8d5, 0xfdf6e3, DARKNESS),
            Palette::interpolate(0x002b36, 0x073642, DARKNESS),
            Palette::interpolate(0x586e75, 0x839496, DARKNESS),
            Palette::interpolate(0xdc322f, 0xdc322f, DARKNESS),
            Palette::interpolate(0xcb4b16, 0xcb4b16, DARKNESS),
            Palette::interpolate(0xb58900, 0xb58900, DARKNESS),
            Palette::interpolate(0x859900, 0x859900, DARKNESS),
            Palette::interpolate(0x859900, 0x859900, DARKNESS),
            Palette::interpolate(0x2aa198, 0x2aa198, DARKNESS),
            Palette::interpolate(0x2aa198, 0x2aa198, DARKNESS),
            Palette::interpolate(0x268bd2, 0x268bd2, DARKNESS),
            Palette::interpolate(0x268bd2, 0x268bd2, DARKNESS),
            Palette::interpolate(0x6c71c4, 0x6c71c4, DARKNESS),
            Palette::interpolate(0xd33682, 0xd33682, DARKNESS),
            Palette::interpolate(0xd33682, 0xd33c82, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][0];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Secondary as usize][3];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::TextDark as usize][0];

    p
};
