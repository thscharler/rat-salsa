use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Rust
/// Rusty theme.
const DARKNESS: u8 = 64;

pub const RUST: Palette = {
    let mut p = Palette {
        name: "Rust", 

        color: [
            Palette::interpolate2(0xd1ccc8, 0xefe6e6, 0x0, 0x0),
            Palette::interpolate2(0x161514, 0x0f0e0d, 0x0, 0x0),
            Palette::interpolate(0x75311a, 0xd25a32, DARKNESS),
            Palette::interpolate(0x77551d, 0xcd9537, DARKNESS),
            Palette::interpolate(0xc4bfbb, 0xede3e3, DARKNESS),
            Palette::interpolate(0x101011, 0x464251, DARKNESS),
            Palette::interpolate(0x726e6b, 0xa39d99, DARKNESS),
            Palette::interpolate(0x75311a, 0xd25a32, DARKNESS),
            Palette::interpolate(0x75431a, 0xd27a32, DARKNESS),
            Palette::interpolate(0x77551d, 0xcd9537, DARKNESS),
            Palette::interpolate(0x44664d, 0x699b76, DARKNESS),
            Palette::interpolate(0x44664d, 0x699b76, DARKNESS),
            Palette::interpolate(0x1a7574, 0x32d2d1, DARKNESS),
            Palette::interpolate(0x1a7574, 0x32d2d1, DARKNESS),
            Palette::interpolate(0x005d94, 0x38b6ff, DARKNESS),
            Palette::interpolate(0x005d94, 0x38b6ff, DARKNESS),
            Palette::interpolate(0x722234, 0xc63f5d, DARKNESS),
            Palette::interpolate(0x7b1964, 0xc62fa3, DARKNESS),
            Palette::interpolate(0x7b1964, 0xd332ad, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][2];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::BlueGreen as usize][2];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Blue as usize][0];
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
