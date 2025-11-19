use std::borrow::Cow;
use crate::{Colors, Palette, define_alias};

/// VSCode
const DARKNESS: u8 = 63;

pub const VSCODE: Palette = Palette {
    name: Cow::Borrowed("VSCode"), 

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
    // must be sorted!
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::Black, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::Gray, 2),
        define_alias("dialog-border.fg", Colors::None, 0),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Cyan, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Cyan, 0),
        define_alias("hover.bg", Colors::Black, 0),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::BlueGreen, 0),
        define_alias("label.fg", Colors::White, 0),
        define_alias("menu-base.bg", Colors::Black, 0),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::White, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Secondary, 1),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Black, 0),
        define_alias("text-focus.bg", Colors::Primary, 1),
        define_alias("text-select.bg", Colors::Secondary, 1),
        define_alias("title.bg", Colors::Red, 0),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::Gray, 3),
    ]),
};

