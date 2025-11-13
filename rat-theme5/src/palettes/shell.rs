use crate::Palette;
use ratatui::style::Color;

/// Shell
///
/// Use the default terminal colors.
pub const SHELL: Palette = Palette {
    name: "Shell",

    text_light: Color::Gray,
    text_bright: Color::White,
    text_dark: Color::DarkGray,
    text_black: Color::Black,

    primary: [Color::Cyan; 8],
    secondary: [Color::Yellow; 8],

    white: [Color::White; 8],
    gray: [Color::Gray; 8],
    black: [Color::Black; 8],

    red: [Color::Red; 8],
    orange: [Color::Yellow; 8],
    yellow: [Color::LightYellow; 8],
    limegreen: [Color::LightGreen; 8],
    green: [Color::Green; 8],
    bluegreen: [Color::Cyan; 8],
    cyan: [Color::LightCyan; 8],
    blue: [Color::LightBlue; 8],
    deepblue: [Color::Blue; 8],
    purple: [Color::Magenta; 8],
    magenta: [Color::LightMagenta; 8],
    redpink: [Color::LightRed; 8],
};
