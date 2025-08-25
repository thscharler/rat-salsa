#![doc = include_str!("../readme.md")]

use crate::dark_theme::DarkTheme;
use crate::scheme::*;
use map_range_int::MapRange;
use ratatui::style::Color;
use ratatui::style::Style;

mod base16;
mod base16r;
pub mod dark_theme;
mod imperial;
mod monekai;
mod monochrome;
mod ocean;
mod oxocarbon;
mod radium;
mod tundra;
mod vscode_dark;

/// Color palettes
pub mod scheme {
    pub use crate::base16::BASE16;
    pub use crate::base16r::BASE16_RELAXED;
    pub use crate::imperial::IMPERIAL;
    pub use crate::monekai::MONEKAI;
    pub use crate::monochrome::MONOCHROME;
    pub use crate::ocean::OCEAN;
    pub use crate::oxocarbon::OXOCARBON;
    pub use crate::radium::RADIUM;
    pub use crate::tundra::TUNDRA;
    pub use crate::vscode_dark::VSCODE_DARK;
}

/// Color scheme.
///
/// This provides the palette used for a theme.
///
/// The ideas packed in here are
/// * provide two colors for highlighting and accents.
/// * I always want some white, black and gray.
/// * I don't want to miss out anything, so go once
///   round the hue in HSV. Take steps of 30° then we
///   hit pretty much anything interesting.
/// * Just one variant of each color is not enough, make it 4.
///
#[derive(Debug, Default, Clone)]
pub struct Scheme {
    pub white: [Color; 4],
    pub black: [Color; 4],
    pub gray: [Color; 4],

    pub red: [Color; 4],
    pub orange: [Color; 4],
    pub yellow: [Color; 4],
    pub limegreen: [Color; 4],
    pub green: [Color; 4],
    pub bluegreen: [Color; 4],
    pub cyan: [Color; 4],
    pub blue: [Color; 4],
    pub deepblue: [Color; 4],
    pub purple: [Color; 4],
    pub magenta: [Color; 4],
    pub redpink: [Color; 4],

    pub primary: [Color; 4],
    pub secondary: [Color; 4],
}

impl Scheme {
    /// Create a style from the given white shade.
    /// n is `0..=3`
    pub fn white(&self, n: usize) -> Style {
        self.style(self.white[n])
    }

    /// Create a style from the given black shade.
    /// n is `0..=3`
    pub fn black(&self, n: usize) -> Style {
        self.style(self.black[n])
    }

    /// Create a style from the given gray shade.
    /// n is `0..=3`
    pub fn gray(&self, n: usize) -> Style {
        self.style(self.gray[n])
    }

    /// Create a style from the given red shade.
    /// n is `0..=3`
    pub fn red(&self, n: usize) -> Style {
        self.style(self.red[n])
    }

    /// Create a style from the given orange shade.
    /// n is `0..=3`
    pub fn orange(&self, n: usize) -> Style {
        self.style(self.orange[n])
    }

    /// Create a style from the given yellow shade.
    /// n is `0..=3`
    pub fn yellow(&self, n: usize) -> Style {
        self.style(self.yellow[n])
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..=3`
    pub fn limegreen(&self, n: usize) -> Style {
        self.style(self.limegreen[n])
    }

    /// Create a style from the given green shade.
    /// n is `0..=3`
    pub fn green(&self, n: usize) -> Style {
        self.style(self.green[n])
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..=3`
    pub fn bluegreen(&self, n: usize) -> Style {
        self.style(self.bluegreen[n])
    }

    /// Create a style from the given cyan shade.
    /// n is `0..=3`
    pub fn cyan(&self, n: usize) -> Style {
        self.style(self.cyan[n])
    }

    /// Create a style from the given blue shade.
    /// n is `0..=3`
    pub fn blue(&self, n: usize) -> Style {
        self.style(self.blue[n])
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..=3`
    pub fn deepblue(&self, n: usize) -> Style {
        self.style(self.deepblue[n])
    }

    /// Create a style from the given purple shade.
    /// n is `0..=3`
    pub fn purple(&self, n: usize) -> Style {
        self.style(self.purple[n])
    }

    /// Create a style from the given magenta shade.
    /// n is `0..=3`
    pub fn magenta(&self, n: usize) -> Style {
        self.style(self.magenta[n])
    }

    /// Create a style from the given redpink shade.
    /// n is `0..=3`
    pub fn redpink(&self, n: usize) -> Style {
        self.style(self.redpink[n])
    }

    /// Create a style from the given primary shade.
    /// n is `0..=3`
    pub fn primary(&self, n: usize) -> Style {
        self.style(self.primary[n])
    }

    /// Create a style from the given secondary shade.
    /// n is `0..=3`
    pub fn secondary(&self, n: usize) -> Style {
        self.style(self.secondary[n])
    }
}

impl Scheme {
    /// Create a style from the given white shade.
    /// n is `0..=3`
    pub fn true_dark_white(&self, n: usize) -> Style {
        self.true_dark_style(self.white[n])
    }

    /// Create a style from the given black shade.
    /// n is `0..=3`
    pub fn true_dark_black(&self, n: usize) -> Style {
        self.true_dark_style(self.black[n])
    }

    /// Create a style from the given gray shade.
    /// n is `0..=3`
    pub fn true_dark_gray(&self, n: usize) -> Style {
        self.true_dark_style(self.gray[n])
    }

    /// Create a style from the given red shade.
    /// n is `0..=3`
    pub fn true_dark_red(&self, n: usize) -> Style {
        self.true_dark_style(self.red[n])
    }

    /// Create a style from the given orange shade.
    /// n is `0..=3`
    pub fn true_dark_orange(&self, n: usize) -> Style {
        self.true_dark_style(self.orange[n])
    }

    /// Create a style from the given yellow shade.
    /// n is `0..=3`
    pub fn true_dark_yellow(&self, n: usize) -> Style {
        self.true_dark_style(self.yellow[n])
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..=3`
    pub fn true_dark_limegreen(&self, n: usize) -> Style {
        self.true_dark_style(self.limegreen[n])
    }

    /// Create a style from the given green shade.
    /// n is `0..=3`
    pub fn true_dark_green(&self, n: usize) -> Style {
        self.true_dark_style(self.green[n])
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..=3`
    pub fn true_dark_bluegreen(&self, n: usize) -> Style {
        self.true_dark_style(self.bluegreen[n])
    }

    /// Create a style from the given cyan shade.
    /// n is `0..=3`
    pub fn true_dark_cyan(&self, n: usize) -> Style {
        self.true_dark_style(self.cyan[n])
    }

    /// Create a style from the given blue shade.
    /// n is `0..=3`
    pub fn true_dark_blue(&self, n: usize) -> Style {
        self.true_dark_style(self.blue[n])
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..=3`
    pub fn true_dark_deepblue(&self, n: usize) -> Style {
        self.true_dark_style(self.deepblue[n])
    }

    /// Create a style from the given purple shade.
    /// n is `0..=3`
    pub fn true_dark_purple(&self, n: usize) -> Style {
        self.true_dark_style(self.purple[n])
    }

    /// Create a style from the given magenta shade.
    /// n is `0..=3`
    pub fn true_dark_magenta(&self, n: usize) -> Style {
        self.true_dark_style(self.magenta[n])
    }

    /// Create a style from the given redpink shade.
    /// n is `0..=3`
    pub fn true_dark_redpink(&self, n: usize) -> Style {
        self.true_dark_style(self.redpink[n])
    }

    /// Create a style from the given primary shade.
    /// n is `0..=3`
    pub fn true_dark_primary(&self, n: usize) -> Style {
        self.true_dark_style(self.primary[n])
    }

    /// Create a style from the given secondary shade.
    /// n is `0..=3`
    pub fn true_dark_secondary(&self, n: usize) -> Style {
        self.true_dark_style(self.secondary[n])
    }
}

impl Scheme {
    /// Create a style from the given white shade.
    /// n is `0..=3`
    pub fn reduced_white(&self, n: usize) -> Style {
        self.reduced_style(self.white[n])
    }

    /// Create a style from the given black shade.
    /// n is `0..=3`
    pub fn reduced_black(&self, n: usize) -> Style {
        self.reduced_style(self.black[n])
    }

    /// Create a style from the given gray shade.
    /// n is `0..=3`
    pub fn reduced_gray(&self, n: usize) -> Style {
        self.reduced_style(self.gray[n])
    }

    /// Create a style from the given red shade.
    /// n is `0..=3`
    pub fn reduced_red(&self, n: usize) -> Style {
        self.reduced_style(self.red[n])
    }

    /// Create a style from the given orange shade.
    /// n is `0..=3`
    pub fn reduced_orange(&self, n: usize) -> Style {
        self.reduced_style(self.orange[n])
    }

    /// Create a style from the given yellow shade.
    /// n is `0..=3`
    pub fn reduced_yellow(&self, n: usize) -> Style {
        self.reduced_style(self.yellow[n])
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..=3`
    pub fn reduced_limegreen(&self, n: usize) -> Style {
        self.reduced_style(self.limegreen[n])
    }

    /// Create a style from the given green shade.
    /// n is `0..=3`
    pub fn reduced_green(&self, n: usize) -> Style {
        self.reduced_style(self.green[n])
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..=3`
    pub fn reduced_bluegreen(&self, n: usize) -> Style {
        self.reduced_style(self.bluegreen[n])
    }

    /// Create a style from the given cyan shade.
    /// n is `0..=3`
    pub fn reduced_cyan(&self, n: usize) -> Style {
        self.reduced_style(self.cyan[n])
    }

    /// Create a style from the given blue shade.
    /// n is `0..=3`
    pub fn reduced_blue(&self, n: usize) -> Style {
        self.reduced_style(self.blue[n])
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..=3`
    pub fn reduced_deepblue(&self, n: usize) -> Style {
        self.reduced_style(self.deepblue[n])
    }

    /// Create a style from the given purple shade.
    /// n is `0..=3`
    pub fn reduced_purple(&self, n: usize) -> Style {
        self.reduced_style(self.purple[n])
    }

    /// Create a style from the given magenta shade.
    /// n is `0..=3`
    pub fn reduced_magenta(&self, n: usize) -> Style {
        self.reduced_style(self.magenta[n])
    }

    /// Create a style from the given redpink shade.
    /// n is `0..=3`
    pub fn reduced_redpink(&self, n: usize) -> Style {
        self.reduced_style(self.redpink[n])
    }

    /// Create a style from the given primary shade.
    /// n is `0..=3`
    pub fn reduced_primary(&self, n: usize) -> Style {
        self.reduced_style(self.primary[n])
    }

    /// Create a style from the given secondary shade.
    /// n is `0..=3`
    pub fn reduced_secondary(&self, n: usize) -> Style {
        self.reduced_style(self.secondary[n])
    }
}

impl Scheme {
    /// Create a style with the given background color.
    /// Foreground is calculated with `text_color`.
    pub fn style(&self, color: Color) -> Style {
        Style::new().bg(color).fg(self.text_color(color))
    }

    /// Create a style with the given background color.
    /// Foreground is calculated with `reduced_text_color`.
    pub fn reduced_style(&self, color: Color) -> Style {
        Style::new().bg(color).fg(self.reduced_text_color(color))
    }

    /// Create a style with the given background color
    /// converted with true_dark_color().
    /// Foreground is calculated with `text_color`.
    pub fn true_dark_style(&self, color: Color) -> Style {
        let dark = self.true_dark_color(color);
        Style::new().bg(dark).fg(self.text_color(dark))
    }

    /// Create a style with the given background color.
    /// converted with true_dark_color().
    /// Foreground is calculated with `reduced_text_color`.
    pub fn reduced_dark_style(&self, color: Color) -> Style {
        let dark = self.true_dark_color(color);
        Style::new().bg(dark).fg(self.reduced_text_color(dark))
    }

    /// Linear interpolation between the two colors.
    pub const fn linear4(c0: u32, c1: u32) -> [Color; 4] {
        // 1/3
        const fn i1(a: u8, b: u8) -> u8 {
            if a < b {
                a + (b - a) / 3
            } else {
                a - (a - b) / 3
            }
        }
        // 2/3
        const fn i2(a: u8, b: u8) -> u8 {
            if a < b {
                b - (b - a) / 3
            } else {
                b + (a - b) / 3
            }
        }

        let r0 = (c0 >> 16) as u8;
        let g0 = (c0 >> 8) as u8;
        let b0 = c0 as u8;

        let r3 = (c1 >> 16) as u8;
        let g3 = (c1 >> 8) as u8;
        let b3 = c1 as u8;

        let r1 = i1(r0, r3);
        let g1 = i1(g0, g3);
        let b1 = i1(b0, b3);

        let r2 = i2(r0, r3);
        let g2 = i2(g0, g3);
        let b2 = i2(b0, b3);

        [
            Color::Rgb(r0, g0, b0),
            Color::Rgb(r1, g1, b1),
            Color::Rgb(r2, g2, b2),
            Color::Rgb(r3, g3, b3),
        ]
    }

    /// Reduces the range of the given color from 0..255 to 0..63.
    ///
    /// This gives a true dark equivalent which can be used
    /// as a background for a dark theme.
    pub fn true_dark_color(&self, color: Color) -> Color {
        let (r, g, b) = as_rgb(color);
        Color::Rgb(
            r.map_range_unchecked((0, 255), (0, 63)),
            g.map_range_unchecked((0, 255), (0, 63)),
            b.map_range_unchecked((0, 255), (0, 63)),
        )
    }

    /// Converts the given color to an equivalent grayscale.
    pub fn grey_color(&self, color: Color) -> Color {
        let (r, g, b) = as_rgb(color);
        // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
        let grey = r as f32 * 0.3f32 + g as f32 * 0.59f32 + b as f32 * 0.11f32;
        Color::Rgb(grey as u8, grey as u8, grey as u8)
    }

    /// This gives back `white[3]` or `black[0]` for text foreground
    /// providing good contrast to the given background.
    ///
    /// This converts RGB to grayscale and takes the grayscale value
    /// of VGA cyan as threshold, which is about 105 out of 255.
    /// This point is a bit arbitrary, just based on what I
    /// perceive as acceptable. But it produces a good reading
    /// contrast in my experience.
    ///
    /// For the named colors it takes the VGA equivalent as a base.
    /// For indexed colors it splits the range in half as an estimate.
    pub fn text_color(&self, color: Color) -> Color {
        match self.rate_text_color(color) {
            None => Color::Reset,
            Some(true) => self.white[3],
            Some(false) => self.black[0],
        }
    }

    /// This gives back `white[3]` or `black[0]` for text foreground
    /// providing good contrast to the given background.
    ///
    /// This converts RGB to grayscale and takes the grayscale value
    /// of VGA cyan as threshold, which is about 105 out of 255.
    /// This point is a bit arbitrary, just based on what I
    /// perceive as acceptable. But it produces a good reading
    /// contrast in my experience.
    ///
    /// For the named colors it takes the VGA equivalent as a base.
    /// For indexed colors it splits the range in half as an estimate.
    pub fn reduced_text_color(&self, color: Color) -> Color {
        match self.rate_text_color(color) {
            None => Color::Reset,
            Some(true) => self.white[0],
            Some(false) => self.black[3],
        }
    }

    /// This gives back `true` or `false` for text foreground
    /// where true means light and false means a dark text-color
    /// providing good contrast to the given background.
    ///
    /// This converts RGB to grayscale and takes the grayscale value
    /// of VGA cyan as threshold, which is about 105 out of 255.
    /// This point is a bit arbitrary, just based on what I
    /// perceive as acceptable. But it produces a good reading
    /// contrast in my experience.
    ///
    /// For the named colors it takes the VGA equivalent as a base.
    /// For indexed colors it splits the range in half as an estimate.
    pub fn rate_text_color(&self, color: Color) -> Option<bool> {
        match color {
            Color::Reset => None,
            Color::Black => Some(true),         //0
            Color::Red => Some(true),           //1
            Color::Green => Some(true),         //2
            Color::Yellow => Some(true),        //3
            Color::Blue => Some(true),          //4
            Color::Magenta => Some(true),       //5
            Color::Cyan => Some(true),          //6
            Color::Gray => Some(false),         //7
            Color::DarkGray => Some(true),      //8
            Color::LightRed => Some(false),     //9
            Color::LightGreen => Some(false),   //10
            Color::LightYellow => Some(false),  //11
            Color::LightBlue => Some(true),     //12
            Color::LightMagenta => Some(false), //13
            Color::LightCyan => Some(false),    //14
            Color::White => Some(false),        //15
            Color::Rgb(r, g, b) => {
                // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
                let grey = r as f32 * 0.3f32 + g as f32 * 0.59f32 + b as f32 * 0.11f32;
                if grey >= 105f32 {
                    Some(false)
                } else {
                    Some(true)
                }
            }
            Color::Indexed(n) => match n {
                0..=6 => Some(true),
                7 => Some(false),
                8 => Some(true),
                9..=11 => Some(false),
                12 => Some(true),
                13..=15 => Some(false),
                v @ 16..=231 => {
                    if (v - 16) % 36 < 18 {
                        Some(true)
                    } else {
                        Some(false)
                    }
                }
                v @ 232..=255 => {
                    if (v - 232) % 24 < 12 {
                        Some(true)
                    } else {
                        Some(false)
                    }
                }
            },
        }
    }
}

/// All currently existing color palettes.
pub fn color_schemes() -> Vec<(String, Scheme)> {
    vec![
        ("Imperial".to_string(), IMPERIAL),
        ("Radium".to_string(), RADIUM),
        ("Tundra".to_string(), TUNDRA),
        ("Monochrome".to_string(), MONOCHROME),
        ("Monekai".to_string(), MONEKAI),
        ("OxoCarbon".to_string(), OXOCARBON),
        ("VSCodeDark".to_string(), VSCODE_DARK),
        ("Ocean".to_string(), OCEAN),
        ("Base16".to_string(), BASE16),
        ("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}

/// A list of DarkTheme for all color palettes.
pub fn dark_themes() -> Vec<DarkTheme> {
    vec![
        DarkTheme::new("Imperial".to_string(), IMPERIAL),
        DarkTheme::new("Radium".to_string(), RADIUM),
        DarkTheme::new("Tundra".to_string(), TUNDRA),
        DarkTheme::new("Monochrome".to_string(), MONOCHROME),
        DarkTheme::new("Monekai".to_string(), MONEKAI),
        DarkTheme::new("Oxocarbon".to_string(), OXOCARBON),
        DarkTheme::new("VSCodeDark".to_string(), VSCODE_DARK),
        DarkTheme::new("Ocean".to_string(), OCEAN),
        DarkTheme::new("Base16".to_string(), BASE16),
        DarkTheme::new("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}

const fn as_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0x00, 0x00, 0x00),
        Color::Red => (0xaa, 0x00, 0x00),
        Color::Green => (0x00, 0xaa, 0x00),
        Color::Yellow => (0xaa, 0x55, 0x00),
        Color::Blue => (0x00, 0x00, 0xaa),
        Color::Magenta => (0xaa, 0x00, 0xaa),
        Color::Cyan => (0x00, 0xaa, 0xaa),
        Color::Gray => (0xaa, 0xaa, 0xaa),
        Color::DarkGray => (0x55, 0x55, 0x55),
        Color::LightRed => (0xff, 0x55, 0x55),
        Color::LightGreen => (0x55, 0xff, 0x55),
        Color::LightYellow => (0xff, 0xff, 0x55),
        Color::LightBlue => (0x55, 0x55, 0xff),
        Color::LightMagenta => (0xff, 0x55, 0xff),
        Color::LightCyan => (0x55, 0xff, 0xff),
        Color::White => (0xff, 0xff, 0xff),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(i) => {
            const VGA256: [(u8, u8, u8); 256] = [
                (0x00, 0x00, 0x00),
                (0x80, 0x00, 0x00),
                (0x00, 0x80, 0x00),
                (0x80, 0x80, 0x00),
                (0x00, 0x00, 0x80),
                (0x80, 0x00, 0x80),
                (0x00, 0x80, 0x80),
                (0xc0, 0xc0, 0xc0),
                (0x80, 0x80, 0x80),
                (0xff, 0x00, 0x00),
                (0x00, 0xff, 0x00),
                (0xff, 0xff, 0x00),
                (0x00, 0x00, 0xff),
                (0xff, 0x00, 0xff),
                (0x00, 0xff, 0xff),
                (0xff, 0xff, 0xff),
                (0x00, 0x00, 0x00),
                (0x00, 0x00, 0x5f),
                (0x00, 0x00, 0x87),
                (0x00, 0x00, 0xaf),
                (0x00, 0x00, 0xd7),
                (0x00, 0x00, 0xff),
                (0x00, 0x5f, 0x00),
                (0x00, 0x5f, 0x5f),
                (0x00, 0x5f, 0x87),
                (0x00, 0x5f, 0xaf),
                (0x00, 0x5f, 0xd7),
                (0x00, 0x5f, 0xff),
                (0x00, 0x87, 0x00),
                (0x00, 0x87, 0x5f),
                (0x00, 0x87, 0x87),
                (0x00, 0x87, 0xaf),
                (0x00, 0x87, 0xd7),
                (0x00, 0x87, 0xff),
                (0x00, 0xaf, 0x00),
                (0x00, 0xaf, 0x5f),
                (0x00, 0xaf, 0x87),
                (0x00, 0xaf, 0xaf),
                (0x00, 0xaf, 0xd7),
                (0x00, 0xaf, 0xff),
                (0x00, 0xd7, 0x00),
                (0x00, 0xd7, 0x5f),
                (0x00, 0xd7, 0x87),
                (0x00, 0xd7, 0xaf),
                (0x00, 0xd7, 0xd7),
                (0x00, 0xd7, 0xff),
                (0x00, 0xff, 0x00),
                (0x00, 0xff, 0x5f),
                (0x00, 0xff, 0x87),
                (0x00, 0xff, 0xaf),
                (0x00, 0xff, 0xd7),
                (0x00, 0xff, 0xff),
                (0x5f, 0x00, 0x00),
                (0x5f, 0x00, 0x5f),
                (0x5f, 0x00, 0x87),
                (0x5f, 0x00, 0xaf),
                (0x5f, 0x00, 0xd7),
                (0x5f, 0x00, 0xff),
                (0x5f, 0x5f, 0x00),
                (0x5f, 0x5f, 0x5f),
                (0x5f, 0x5f, 0x87),
                (0x5f, 0x5f, 0xaf),
                (0x5f, 0x5f, 0xd7),
                (0x5f, 0x5f, 0xff),
                (0x5f, 0x87, 0x00),
                (0x5f, 0x87, 0x5f),
                (0x5f, 0x87, 0x87),
                (0x5f, 0x87, 0xaf),
                (0x5f, 0x87, 0xd7),
                (0x5f, 0x87, 0xff),
                (0x5f, 0xaf, 0x00),
                (0x5f, 0xaf, 0x5f),
                (0x5f, 0xaf, 0x87),
                (0x5f, 0xaf, 0xaf),
                (0x5f, 0xaf, 0xd7),
                (0x5f, 0xaf, 0xff),
                (0x5f, 0xd7, 0x00),
                (0x5f, 0xd7, 0x5f),
                (0x5f, 0xd7, 0x87),
                (0x5f, 0xd7, 0xaf),
                (0x5f, 0xd7, 0xd7),
                (0x5f, 0xd7, 0xff),
                (0x5f, 0xff, 0x00),
                (0x5f, 0xff, 0x5f),
                (0x5f, 0xff, 0x87),
                (0x5f, 0xff, 0xaf),
                (0x5f, 0xff, 0xd7),
                (0x5f, 0xff, 0xff),
                (0x87, 0x00, 0x00),
                (0x87, 0x00, 0x5f),
                (0x87, 0x00, 0x87),
                (0x87, 0x00, 0xaf),
                (0x87, 0x00, 0xd7),
                (0x87, 0x00, 0xff),
                (0x87, 0x5f, 0x00),
                (0x87, 0x5f, 0x5f),
                (0x87, 0x5f, 0x87),
                (0x87, 0x5f, 0xaf),
                (0x87, 0x5f, 0xd7),
                (0x87, 0x5f, 0xff),
                (0x87, 0x87, 0x00),
                (0x87, 0x87, 0x5f),
                (0x87, 0x87, 0x87),
                (0x87, 0x87, 0xaf),
                (0x87, 0x87, 0xd7),
                (0x87, 0x87, 0xff),
                (0x87, 0xaf, 0x00),
                (0x87, 0xaf, 0x5f),
                (0x87, 0xaf, 0x87),
                (0x87, 0xaf, 0xaf),
                (0x87, 0xaf, 0xd7),
                (0x87, 0xaf, 0xff),
                (0x87, 0xd7, 0x00),
                (0x87, 0xd7, 0x5f),
                (0x87, 0xd7, 0x87),
                (0x87, 0xd7, 0xaf),
                (0x87, 0xd7, 0xd7),
                (0x87, 0xd7, 0xff),
                (0x87, 0xff, 0x00),
                (0x87, 0xff, 0x5f),
                (0x87, 0xff, 0x87),
                (0x87, 0xff, 0xaf),
                (0x87, 0xff, 0xd7),
                (0x87, 0xff, 0xff),
                (0xaf, 0x00, 0x00),
                (0xaf, 0x00, 0x5f),
                (0xaf, 0x00, 0x87),
                (0xaf, 0x00, 0xaf),
                (0xaf, 0x00, 0xd7),
                (0xaf, 0x00, 0xff),
                (0xaf, 0x5f, 0x00),
                (0xaf, 0x5f, 0x5f),
                (0xaf, 0x5f, 0x87),
                (0xaf, 0x5f, 0xaf),
                (0xaf, 0x5f, 0xd7),
                (0xaf, 0x5f, 0xff),
                (0xaf, 0x87, 0x00),
                (0xaf, 0x87, 0x5f),
                (0xaf, 0x87, 0x87),
                (0xaf, 0x87, 0xaf),
                (0xaf, 0x87, 0xd7),
                (0xaf, 0x87, 0xff),
                (0xaf, 0xaf, 0x00),
                (0xaf, 0xaf, 0x5f),
                (0xaf, 0xaf, 0x87),
                (0xaf, 0xaf, 0xaf),
                (0xaf, 0xaf, 0xd7),
                (0xaf, 0xaf, 0xff),
                (0xaf, 0xd7, 0x00),
                (0xaf, 0xd7, 0x5f),
                (0xaf, 0xd7, 0x87),
                (0xaf, 0xd7, 0xaf),
                (0xaf, 0xd7, 0xd7),
                (0xaf, 0xd7, 0xff),
                (0xaf, 0xff, 0x00),
                (0xaf, 0xff, 0x5f),
                (0xaf, 0xff, 0x87),
                (0xaf, 0xff, 0xaf),
                (0xaf, 0xff, 0xd7),
                (0xaf, 0xff, 0xff),
                (0xd7, 0x00, 0x00),
                (0xd7, 0x00, 0x5f),
                (0xd7, 0x00, 0x87),
                (0xd7, 0x00, 0xaf),
                (0xd7, 0x00, 0xd7),
                (0xd7, 0x00, 0xff),
                (0xd7, 0x5f, 0x00),
                (0xd7, 0x5f, 0x5f),
                (0xd7, 0x5f, 0x87),
                (0xd7, 0x5f, 0xaf),
                (0xd7, 0x5f, 0xd7),
                (0xd7, 0x5f, 0xff),
                (0xd7, 0x87, 0x00),
                (0xd7, 0x87, 0x5f),
                (0xd7, 0x87, 0x87),
                (0xd7, 0x87, 0xaf),
                (0xd7, 0x87, 0xd7),
                (0xd7, 0x87, 0xff),
                (0xd7, 0xaf, 0x00),
                (0xd7, 0xaf, 0x5f),
                (0xd7, 0xaf, 0x87),
                (0xd7, 0xaf, 0xaf),
                (0xd7, 0xaf, 0xd7),
                (0xd7, 0xaf, 0xff),
                (0xd7, 0xd7, 0x00),
                (0xd7, 0xd7, 0x5f),
                (0xd7, 0xd7, 0x87),
                (0xd7, 0xd7, 0xaf),
                (0xd7, 0xd7, 0xd7),
                (0xd7, 0xd7, 0xff),
                (0xd7, 0xff, 0x00),
                (0xd7, 0xff, 0x5f),
                (0xd7, 0xff, 0x87),
                (0xd7, 0xff, 0xaf),
                (0xd7, 0xff, 0xd7),
                (0xd7, 0xff, 0xff),
                (0xff, 0x00, 0x00),
                (0xff, 0x00, 0x5f),
                (0xff, 0x00, 0x87),
                (0xff, 0x00, 0xaf),
                (0xff, 0x00, 0xd7),
                (0xff, 0x00, 0xff),
                (0xff, 0x5f, 0x00),
                (0xff, 0x5f, 0x5f),
                (0xff, 0x5f, 0x87),
                (0xff, 0x5f, 0xaf),
                (0xff, 0x5f, 0xd7),
                (0xff, 0x5f, 0xff),
                (0xff, 0x87, 0x00),
                (0xff, 0x87, 0x5f),
                (0xff, 0x87, 0x87),
                (0xff, 0x87, 0xaf),
                (0xff, 0x87, 0xd7),
                (0xff, 0x87, 0xff),
                (0xff, 0xaf, 0x00),
                (0xff, 0xaf, 0x5f),
                (0xff, 0xaf, 0x87),
                (0xff, 0xaf, 0xaf),
                (0xff, 0xaf, 0xd7),
                (0xff, 0xaf, 0xff),
                (0xff, 0xd7, 0x00),
                (0xff, 0xd7, 0x5f),
                (0xff, 0xd7, 0x87),
                (0xff, 0xd7, 0xaf),
                (0xff, 0xd7, 0xd7),
                (0xff, 0xd7, 0xff),
                (0xff, 0xff, 0x00),
                (0xff, 0xff, 0x5f),
                (0xff, 0xff, 0x87),
                (0xff, 0xff, 0xaf),
                (0xff, 0xff, 0xd7),
                (0xff, 0xff, 0xff),
                (0x08, 0x08, 0x08),
                (0x12, 0x12, 0x12),
                (0x1c, 0x1c, 0x1c),
                (0x26, 0x26, 0x26),
                (0x30, 0x30, 0x30),
                (0x3a, 0x3a, 0x3a),
                (0x44, 0x44, 0x44),
                (0x4e, 0x4e, 0x4e),
                (0x58, 0x58, 0x58),
                (0x62, 0x62, 0x62),
                (0x6c, 0x6c, 0x6c),
                (0x76, 0x76, 0x76),
                (0x80, 0x80, 0x80),
                (0x8a, 0x8a, 0x8a),
                (0x94, 0x94, 0x94),
                (0x9e, 0x9e, 0x9e),
                (0xa8, 0xa8, 0xa8),
                (0xb2, 0xb2, 0xb2),
                (0xbc, 0xbc, 0xbc),
                (0xc6, 0xc6, 0xc6),
                (0xd0, 0xd0, 0xd0),
                (0xda, 0xda, 0xda),
                (0xe4, 0xe4, 0xe4),
                (0xee, 0xee, 0xee),
            ];
            VGA256[i as usize]
        }
        Color::Reset => (0, 0, 0),
    }
}
