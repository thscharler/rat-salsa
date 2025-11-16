use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Monekai
const DARKNESS: u8 = 63;

pub const MONEKAI: Palette = {
    let mut p = Palette {
        name: "Monekai", 

        color: [
            Palette::interpolate2(0xf5f4f1, 0xfffefc, 0x0, 0x0),
            Palette::interpolate2(0x272822, 0x464741, 0x0, 0x0),
            Palette::interpolate(0xc11f5a, 0xf92672, DARKNESS),
            Palette::interpolate(0x5c7289, 0x81a1c1, DARKNESS),
            Palette::interpolate(0xf5f4f1, 0xf5f4f1, DARKNESS),
            Palette::interpolate(0x22231d, 0x2f302a, DARKNESS),
            Palette::interpolate(0x4d4e48, 0x64655f, DARKNESS),
            Palette::interpolate(0xe36d76, 0xe36d76, DARKNESS),
            Palette::interpolate(0xd39467, 0xd39467, DARKNESS),
            Palette::interpolate(0xe6c181, 0xe6c181, DARKNESS),
            Palette::interpolate(0x96c367, 0x96c367, DARKNESS),
            Palette::interpolate(0x96c367, 0x96c367, DARKNESS),
            Palette::interpolate(0x34bfd0, 0x34bfd0, DARKNESS),
            Palette::interpolate(0x41afef, 0x41afef, DARKNESS),
            Palette::interpolate(0x51afef, 0x51afef, DARKNESS),
            Palette::interpolate(0x81a1c1, 0x81a1c1, DARKNESS),
            Palette::interpolate(0xae81ff, 0xae81ff, DARKNESS),
            Palette::interpolate(0xf92672, 0xf72270, DARKNESS),
            Palette::interpolate(0xf98385, 0xf98385, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][3];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][1];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Primary as usize][4];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Magenta as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::DeepBlue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::DeepBlue as usize][0];
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
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::TextDark as usize][0];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][1];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::Black as usize][3];

    p
};
