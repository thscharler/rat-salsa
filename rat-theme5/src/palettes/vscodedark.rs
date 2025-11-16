use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// VSCodeDark
const DARKNESS: u8 = 63;

pub const VSCODEDARK: Palette = {
    let mut p = Palette {
        name: "VSCodeDark", 

        color: [
            Palette::interpolate2(0xd4d4d4, 0xffffff, 0x0, 0x0),
            Palette::interpolate2(0x1a1a1a, 0x3a3a3a, 0x0, 0x0),
            Palette::interpolate(0xd4d4d4, 0xffffff, DARKNESS),
            Palette::interpolate(0x444444, 0x878787, DARKNESS),
            Palette::interpolate(0xd4d4d4, 0xffffff, DARKNESS),
            Palette::interpolate(0x1a1a1a, 0x3a3a3a, DARKNESS),
            Palette::interpolate(0x444444, 0x878787, DARKNESS),
            Palette::interpolate(0xd16969, 0xd16969, DARKNESS),
            Palette::interpolate(0xd57e62, 0xd3967d, DARKNESS),
            Palette::interpolate(0xd7ba7d, 0xd7ba7d, DARKNESS),
            Palette::interpolate(0x9cda80, 0x9cda80, DARKNESS),
            Palette::interpolate(0x80daba, 0x80daba, DARKNESS),
            Palette::interpolate(0xb5cea8, 0xb5cea8, DARKNESS),
            Palette::interpolate(0x9cdcfe, 0x9cdcfe, DARKNESS),
            Palette::interpolate(0x89beec, 0x89beec, DARKNESS),
            Palette::interpolate(0x85bae6, 0x85bae6, DARKNESS),
            Palette::interpolate(0xbd88ed, 0xbd88ed, DARKNESS),
            Palette::interpolate(0xbb7cb6, 0xbb7cb6, DARKNESS),
            Palette::interpolate(0xe98691, 0xe98691, DARKNESS),
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
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::Cyan as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::Cyan as usize][0];
    p.color_ext[ColorsExt::Shadows as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][0];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][0];
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
