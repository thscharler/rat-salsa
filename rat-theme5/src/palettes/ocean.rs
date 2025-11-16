use crate::{Colors, ColorsExt, Palette};
use ratatui::style::Color;

/// Ocean
/// My take on an ocean theme.
const DARKNESS: u8 = 63;

pub const OCEAN: Palette = {
    let mut p = Palette {
        name: "Ocean", 

        color: [
            Palette::interpolate2(0xe5e5dd, 0xf2f2ee, 0x0, 0x0),
            Palette::interpolate2(0x030305, 0x0c092c, 0x0, 0x0),
            Palette::interpolate(0xff8d3c, 0xffbf3c, DARKNESS),
            Palette::interpolate(0x2b4779, 0x6688cc, DARKNESS),
            Palette::interpolate(0xe5e5dd, 0xf2f2ee, DARKNESS),
            Palette::interpolate(0x030305, 0x0c092c, DARKNESS),
            Palette::interpolate(0x4f6167, 0xbcc7cc, DARKNESS),
            Palette::interpolate(0xff5e7f, 0xff9276, DARKNESS),
            Palette::interpolate(0xff9f5b, 0xffdc94, DARKNESS),
            Palette::interpolate(0xffda5d, 0xfff675, DARKNESS),
            Palette::interpolate(0x7d8447, 0xe1e5b9, DARKNESS),
            Palette::interpolate(0x658362, 0x99c794, DARKNESS),
            Palette::interpolate(0x3a615c, 0x5b9c90, DARKNESS),
            Palette::interpolate(0x24adbc, 0xb8dade, DARKNESS),
            Palette::interpolate(0x4f86ca, 0xbfdcff, DARKNESS),
            Palette::interpolate(0x2b4779, 0x6688cc, DARKNESS),
            Palette::interpolate(0x5068d7, 0xc7c4ff, DARKNESS),
            Palette::interpolate(0x7952d6, 0xc9bde4, DARKNESS),
            Palette::interpolate(0x9752d6, 0xcebde7, DARKNESS),
        ],
        color_ext: [Color::Reset; ColorsExt::LEN],
    };

    p.color_ext[ColorsExt::LabelFg as usize] = p.color[Colors::White as usize][0];
    p.color_ext[ColorsExt::Input as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Focus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::Select as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::Disabled as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::Invalid as usize] = p.color[Colors::Red as usize][0];
    p.color_ext[ColorsExt::Hover as usize] = p.color[Colors::Primary as usize][2];
    p.color_ext[ColorsExt::TitleFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::Title as usize] = p.color[Colors::Yellow as usize][0];
    p.color_ext[ColorsExt::HeaderFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Header as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::FooterFg as usize] = p.color[Colors::TextLight as usize][0];
    p.color_ext[ColorsExt::Footer as usize] = p.color[Colors::Blue as usize][0];
    p.color_ext[ColorsExt::Shadow as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::TextFocus as usize] = p.color[Colors::Primary as usize][1];
    p.color_ext[ColorsExt::TextSelect as usize] = p.color[Colors::Secondary as usize][1];
    p.color_ext[ColorsExt::ButtonBase as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::MenuBase as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::KeyBinding as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::StatusBase as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::ContainerBase as usize] = p.color[Colors::Black as usize][3];
    p.color_ext[ColorsExt::ContainerBorderFg as usize] = p.color[Colors::Gray as usize][0];
    p.color_ext[ColorsExt::ContainerArrowFg as usize] = p.color[Colors::BlueGreen as usize][0];
    p.color_ext[ColorsExt::PopupBase as usize] = p.color[Colors::Gray as usize][3];
    p.color_ext[ColorsExt::PopupBorderFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::PopupArrowFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::DialogBase as usize] = p.color[Colors::Gray as usize][2];
    p.color_ext[ColorsExt::DialogBorderFg as usize] = p.color[Colors::TextDark as usize][3];
    p.color_ext[ColorsExt::DialogArrowFg as usize] = p.color[Colors::TextDark as usize][3];

    p
};
