use crate::RatWidgetColor;
use crate::palette::{Colors, Palette, define_alias};
use ratatui::style::Color;
use std::borrow::Cow;

pub const SHELL: Palette = Palette {
    name: Cow::Borrowed("Shell"),
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
        [Color::Black; 8],
        [
            Color::Gray,
            Color::Gray,
            Color::DarkGray,
            Color::DarkGray,
            Color::Gray,
            Color::Gray,
            Color::DarkGray,
            Color::DarkGray,
        ],
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
        define_alias(Color::BUTTON_BASE_BG, Colors::Gray, 0),
        define_alias(Color::CONTAINER_ARROW_FG, Colors::Gray, 0),
        define_alias(Color::CONTAINER_BASE_BG, Colors::Black, 0),
        define_alias(Color::CONTAINER_BORDER_FG, Colors::Gray, 0),
        define_alias(Color::DIALOG_ARROW_FG, Colors::Black, 0),
        define_alias(Color::DIALOG_BASE_BG, Colors::Gray, 3),
        define_alias(Color::DIALOG_BORDER_FG, Colors::Black, 0),
        define_alias(Color::DISABLED_BG, Colors::Gray, 0),
        define_alias(Color::FOCUS_BG, Colors::Primary, 0),
        define_alias(Color::FOOTER_BG, Colors::Blue, 0),
        define_alias(Color::FOOTER_FG, Colors::TextLight, 0),
        define_alias(Color::HEADER_BG, Colors::Blue, 0),
        define_alias(Color::HEADER_FG, Colors::TextLight, 0),
        define_alias(Color::HOVER_BG, Colors::Gray, 3),
        define_alias(Color::INPUT_BG, Colors::Gray, 3),
        define_alias(Color::INVALID_BG, Colors::Red, 0),
        define_alias(Color::KEY_BINDING_BG, Colors::BlueGreen, 0),
        define_alias(Color::LABEL_FG, Colors::White, 0),
        define_alias(Color::MENU_BASE_BG, Colors::Black, 0),
        define_alias(Color::MONTH_HEADER_FG, Colors::TextDark, 0),
        define_alias(Color::POPUP_ARROW_FG, Colors::Gray, 3),
        define_alias(Color::POPUP_BASE_BG, Colors::White, 0),
        define_alias(Color::POPUP_BORDER_FG, Colors::Gray, 3),
        define_alias(Color::SELECT_BG, Colors::Secondary, 0),
        define_alias(Color::SHADOW_BG, Colors::TextDark, 0),
        define_alias(Color::STATUS_BASE_BG, Colors::Black, 0),
        define_alias(Color::TEXT_FOCUS_BG, Colors::Primary, 0),
        define_alias(Color::TEXT_SELECT_BG, Colors::Secondary, 0),
        define_alias(Color::TITLE_BG, Colors::Red, 0),
        define_alias(Color::TITLE_FG, Colors::TextLight, 0),
        define_alias(Color::WEEK_HEADER_FG, Colors::TextDark, 0),
    ]),
};
