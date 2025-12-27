//!
//! Defines the SHELL palette.
//!
use crate::palette::{Colors, Palette, define_alias};
use ratatui_core::style::Color;
use std::borrow::Cow;

/// A `Shell` palette that uses only named [Color]s.
///
/// This is useful if you want to let the terminal decide
/// what exact color 'red' really is.
pub const SHELL: Palette = Palette {
    theme_name: Cow::Borrowed("Shell"),
    theme: Cow::Borrowed("Shell"),
    name: Cow::Borrowed("Shell"),
    doc: Cow::Borrowed(""),
    generator: Cow::Borrowed("spectrum"),
    color: [
        [
            Color::Gray,
            Color::Gray,
            Color::White,
            Color::White,
            Color::Gray,
            Color::Gray,
            Color::White,
            Color::White,
        ], // text light
        [
            Color::DarkGray,
            Color::DarkGray,
            Color::Black,
            Color::Black,
            Color::DarkGray,
            Color::DarkGray,
            Color::Black,
            Color::Black,
        ], // text dark
        [Color::Cyan; 8],   // primary
        [Color::Yellow; 8], // secondary
        [Color::White; 8],  // white
        [Color::Black; 8],  // black
        [
            Color::Gray,
            Color::Gray,
            Color::DarkGray,
            Color::DarkGray,
            Color::Gray,
            Color::Gray,
            Color::DarkGray,
            Color::DarkGray,
        ], // gray
        [Color::Red; 8],
        [Color::Yellow; 8],
        [Color::LightYellow; 8],
        [Color::LightGreen; 8],
        [Color::Green; 8],
        [Color::Cyan; 8],
        [Color::LightCyan; 8],
        [Color::LightBlue; 8],
        [Color::Blue; 8],
        [Color::Magenta; 8],
        [Color::LightMagenta; 8],
        [Color::LightRed; 8],
    ],
    aliased: Cow::Borrowed(&[
        define_alias("button-base.bg", Colors::Gray, 0),
        define_alias("container-arrow.fg", Colors::None, 0),
        define_alias("container-base.bg", Colors::None, 0),
        define_alias("container-border.fg", Colors::None, 0),
        define_alias("dialog-arrow.fg", Colors::None, 0),
        define_alias("dialog-base.bg", Colors::None, 0),
        define_alias("dialog-border.fg", Colors::TextLight, 3),
        define_alias("disabled.bg", Colors::Gray, 3),
        define_alias("document-arrow.fg", Colors::None, 0),
        define_alias("document-base.bg", Colors::None, 0),
        define_alias("document-border.fg", Colors::None, 0),
        define_alias("focus.bg", Colors::Primary, 0),
        define_alias("footer.bg", Colors::None, 0),
        define_alias("footer.fg", Colors::Blue, 0),
        define_alias("header.bg", Colors::None, 0),
        define_alias("header.fg", Colors::Blue, 0),
        define_alias("hover.bg", Colors::Cyan, 0),
        define_alias("input-focus.bg", Colors::Primary, 0),
        define_alias("input-select.bg", Colors::Gray, 3),
        define_alias("input.bg", Colors::Gray, 0),
        define_alias("invalid.bg", Colors::Red, 0),
        define_alias("key-binding.bg", Colors::None, 0),
        define_alias("label.fg", Colors::TextLight, 0),
        define_alias("md+hidden", Colors::None, 0),
        define_alias("menu-base.bg", Colors::Gray, 3),
        define_alias("month-header.fg", Colors::None, 0),
        define_alias("popup-arrow.fg", Colors::None, 0),
        define_alias("popup-base.bg", Colors::None, 0),
        define_alias("popup-border.fg", Colors::None, 0),
        define_alias("select.bg", Colors::Gray, 3),
        define_alias("shadow.bg", Colors::None, 0),
        define_alias("status-base.bg", Colors::Gray, 3),
        define_alias("title.bg", Colors::Blue, 0),
        define_alias("title.fg", Colors::TextLight, 0),
        define_alias("week-header.fg", Colors::None, 0),
    ]),
};
