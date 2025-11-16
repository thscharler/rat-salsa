use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Monekai
const DARKNESS: u8 = 63;

pub const MONEKAI: Palette = {
    let mut p = Palette {
        name: "Monekai",

        color: [
            Palette::interpolate2(0xc9c9c0, 0xf5f4f1, 0x0, 0x0),
            Palette::interpolate2(0x272822, 0x464741, 0x0, 0x0),
            Palette::interpolate(0x80133a, 0xd12060, DARKNESS),
            Palette::interpolate(0x5e748c, 0x81a1c1, DARKNESS),
            Palette::interpolate(0xb0b2a8, 0xf5f4f1, DARKNESS),
            Palette::interpolate(0x272822, 0x464741, DARKNESS),
            Palette::interpolate(0x4d4e48, 0x64655f, DARKNESS),
            Palette::interpolate(0x804c10, 0xfd971f, DARKNESS),
            Palette::interpolate(0x584180, 0xae81ff, DARKNESS),
            Palette::interpolate(0x80643d, 0xf4bf75, DARKNESS),
            Palette::interpolate(0x5e801a, 0xa6e22e, DARKNESS),
            Palette::interpolate(0x628043, 0x96c367, DARKNESS),
            Palette::interpolate(0x207580, 0x34bfd0, DARKNESS),
            Palette::interpolate(0x235d80, 0x41afef, DARKNESS),
            Palette::interpolate(0x2f668c, 0x51afef, DARKNESS),
            Palette::interpolate(0x5e748c, 0x81a1c1, DARKNESS),
            Palette::interpolate(0x764980, 0xb26fc1, DARKNESS),
            Palette::interpolate(0x80133a, 0xf92672, DARKNESS),
            Palette::interpolate(0x804020, 0xcc6633, DARKNESS),
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
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::TextDark as usize][0];
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
