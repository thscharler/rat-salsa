use std::borrow::Cow;
use crate::palette::{Colors, Palette, define_alias};

const DARKNESS: u8 = 64;

/// VSCode
/// VSCode color palette.
pub const VSCODE_SHELL: Palette = Palette {
    theme_name: Cow::Borrowed("VSCode Shell"), 
    theme: Cow::Borrowed("Shell"), 
    name: Cow::Borrowed("VSCode"), 
    doc: Cow::Borrowed("VSCode color palette."), 
    generator: Cow::Borrowed("light-dark:64"), 

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
        define_alias("container-arrow.fg", Colors::Gray, 1),
        define_alias("container-base.bg", Colors::None, 0),
        define_alias("container-border.fg", Colors::Gray, 1),
        define_alias("dialog-arrow.fg", Colors::White, 0),
        define_alias("dialog-base.bg", Colors::None, 0),
        define_alias("dialog-border.fg", Colors::White, 0),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("document-arrow.fg", Colors::Gray, 1),
        define_alias("document-base.bg", Colors::None, 0),
        define_alias("document-border.fg", Colors::Gray, 1),
        define_alias("focus.bg", Colors::Primary, 1),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Cyan, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Cyan, 0),
        define_alias("hover.bg", Colors::Black, 0),
        define_alias("input-focus.bg", Colors::Primary, 1),
        define_alias("input-select.bg", Colors::Secondary, 1),
        define_alias("input.bg", Colors::Gray, 3),
        define_alias("invalid.bg", Colors::Red, 1),
        define_alias("key-binding.bg", Colors::Secondary, 1),
        define_alias("label.fg", Colors::White, 0),
        define_alias("menu-base.bg", Colors::Black, 1),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::Primary, 1),
        define_alias("popup-base.bg", Colors::Gray, 3),
        define_alias("popup-border.fg", Colors::Primary, 1),
        define_alias("select.bg", Colors::Secondary, 1),
        define_alias("shadow.bg", Colors::Black, 0),
        define_alias("status-base.bg", Colors::Black, 1),
        define_alias("title.bg", Colors::Red, 0),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::Gray, 3),
    ]),
};

