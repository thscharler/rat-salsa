//!
//! [Palette] is the color palette for salsa-themes.
//!
//! It has graduations for white, black, gray, red, orange,
//! yellow, limegreen, green, bluegreen, cyan, blue, deepblue,
//! purple, magenta and a redpink.
//! And it has a primary and a secondary highlight color.
//! And it has graduations for light/dark text.
//!
//! There is an algorithm that chooses the text-color for a
//! given background.
//!
//! And there is a semantic layer, that can give names to
//! specific colors. It's these names/aliases that are primarily
//! used when composing everything into a theme.
//!
//! This way salsa-theme can have one dark theme that works
//! with multiple palettes.
//!

use ratatui::style::{Color, Style};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::mem;
use std::str::FromStr;

/// Refers to a color in the Palette.
/// This is used for color-aliases.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorIdx(pub Colors, pub usize);

/// Names/Indexes into the color-table of the palette.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Colors {
    /// Colors for light text.
    TextLight = 0,
    /// Colors for dark text.
    TextDark,
    /// Primary highlight color.
    Primary,
    /// Secondary highlight color.
    Secondary,
    White,
    Black,
    Gray,
    Red,
    Orange,
    Yellow,
    LimeGreen,
    Green,
    BlueGreen,
    Cyan,
    Blue,
    DeepBlue,
    Purple,
    Magenta,
    RedPink,
    /// Undefined color.
    ///
    /// Maps to Color::Reset when queried as a color.
    /// When used to define a style fg or bg it sets them to [Option::None].
    #[default]
    None,
}

impl Display for ColorIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl FromStr for ColorIdx {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ss = s.split(':');
        let Some(name) = ss.next() else {
            return Err(());
        };
        let Ok(c) = Colors::from_str(name) else {
            return Err(());
        };
        let Some(idx) = ss.next() else { return Err(()) };
        let Ok(idx) = idx.parse::<usize>() else {
            return Err(());
        };
        Ok(ColorIdx(c, idx))
    }
}

impl Display for Colors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Colors::TextLight => "text-light",
            Colors::TextDark => "text-dark",
            Colors::Primary => "primary",
            Colors::Secondary => "secondary",
            Colors::White => "white",
            Colors::Black => "black",
            Colors::Gray => "gray",
            Colors::Red => "red",
            Colors::Orange => "orange",
            Colors::Yellow => "yellow",
            Colors::LimeGreen => "lime-green",
            Colors::Green => "green",
            Colors::BlueGreen => "blue-green",
            Colors::Cyan => "cyan",
            Colors::Blue => "blue",
            Colors::DeepBlue => "deep-blue",
            Colors::Purple => "purple",
            Colors::Magenta => "magenta",
            Colors::RedPink => "red-pink",
            Colors::None => "none",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Colors {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text-light" => Ok(Colors::TextLight),
            "text-dark" => Ok(Colors::TextDark),
            "primary" => Ok(Colors::Primary),
            "secondary" => Ok(Colors::Secondary),
            "white" => Ok(Colors::White),
            "black" => Ok(Colors::Black),
            "gray" => Ok(Colors::Gray),
            "red" => Ok(Colors::Red),
            "orange" => Ok(Colors::Orange),
            "yellow" => Ok(Colors::Yellow),
            "lime-green" => Ok(Colors::LimeGreen),
            "green" => Ok(Colors::Green),
            "blue-green" => Ok(Colors::BlueGreen),
            "cyan" => Ok(Colors::Cyan),
            "blue" => Ok(Colors::Blue),
            "deep-blue" => Ok(Colors::DeepBlue),
            "purple" => Ok(Colors::Purple),
            "magenta" => Ok(Colors::Magenta),
            "red-pink" => Ok(Colors::RedPink),
            "none" => Ok(Colors::None),
            _ => Err(()),
        }
    }
}

impl Colors {
    pub const LEN: usize = 19;

    pub fn array() -> &'static [Colors] {
        use Colors::*;
        // don't include None!
        &[
            TextLight, TextDark, Primary, Secondary, White, Black, Gray, Red, Orange, Yellow,
            LimeGreen, Green, BlueGreen, Cyan, Blue, DeepBlue, Purple, Magenta, RedPink,
        ]
    }
}

/// Color palette.
///
/// This provides the palette used for a theme.
#[derive(Debug, Clone)]
pub struct Palette {
    /// Name of the theme.
    pub theme_name: Cow<'static, str>,
    /// Which theme should be created.
    /// Known themes:
    /// * Dark
    /// * Light
    /// * Shell
    /// There are 3 more special cased themes for fallback/testing
    /// purposes.
    /// * Core - Fallback theme if something fails.
    /// * Blackout - Testing theme that blacks everything.
    /// * Fallback - Testing theme that relies on each widget's own defaults.
    pub theme: Cow<'static, str>,
    /// Name of the color palette.
    pub name: Cow<'static, str>,
    /// Doc string.
    pub doc: Cow<'static, str>,
    /// Generator function.
    /// Determines how the palette is stored.
    /// Currently:
    /// * light-dark:N
    ///   stores color[0] and color[3] and interpolates between them.
    ///   color[4]..=color[7] are generated by scaling the rgb values
    ///   down to N.
    pub generator: Cow<'static, str>,
    /// Color palette. Use [Colors] for indexing.
    pub color: [[Color; 8]; Colors::LEN],
    /// **Sorted** list of aliases.
    /// Must be pre-sorted for binary-search.
    pub aliased: Cow<'static, [(Cow<'static, str>, ColorIdx)]>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            theme_name: Default::default(),
            theme: Default::default(),
            name: Default::default(),
            doc: Default::default(),
            generator: Default::default(),
            color: Default::default(),
            aliased: Default::default(),
        }
    }
}

/// Contrast rating for the text-color that should be used.
#[derive(Debug)]
pub(crate) enum Rating {
    /// Use light/white text for the given background.
    Light,
    /// Use dark/black text for the given background.
    Dark,
}

/// Create a color alias. Useful when creating a static Palette.
pub const fn define_alias(
    alias: &'static str,
    color: Colors,
    n: usize,
) -> (Cow<'static, str>, ColorIdx) {
    (Cow::Borrowed(alias), ColorIdx(color, n))
}

/// Create a color alias. This function is useful when
/// modifying a Palette at runtime.
pub fn define_rt_alias(
    alias: impl Into<String>,
    color: Colors,
    n: usize,
) -> (Cow<'static, str>, ColorIdx) {
    let alias = alias.into();
    (Cow::Owned(alias), ColorIdx(color, n))
}

impl Palette {
    /// Create a style from the given white shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn white(&self, n: usize) -> Style {
        self.style(Colors::White, n)
    }

    /// Create a style from the given black shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn black(&self, n: usize) -> Style {
        self.style(Colors::Black, n)
    }

    /// Create a style from the given gray shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn gray(&self, n: usize) -> Style {
        self.style(Colors::Gray, n)
    }

    /// Create a style from the given red shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn red(&self, n: usize) -> Style {
        self.style(Colors::Red, n)
    }

    /// Create a style from the given orange shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn orange(&self, n: usize) -> Style {
        self.style(Colors::Orange, n)
    }

    /// Create a style from the given yellow shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn yellow(&self, n: usize) -> Style {
        self.style(Colors::Yellow, n)
    }

    /// Create a style from the given limegreen shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn limegreen(&self, n: usize) -> Style {
        self.style(Colors::LimeGreen, n)
    }

    /// Create a style from the given green shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn green(&self, n: usize) -> Style {
        self.style(Colors::Green, n)
    }

    /// Create a style from the given bluegreen shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn bluegreen(&self, n: usize) -> Style {
        self.style(Colors::BlueGreen, n)
    }

    /// Create a style from the given cyan shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn cyan(&self, n: usize) -> Style {
        self.style(Colors::Cyan, n)
    }

    /// Create a style from the given blue shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn blue(&self, n: usize) -> Style {
        self.style(Colors::Blue, n)
    }

    /// Create a style from the given deepblue shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn deepblue(&self, n: usize) -> Style {
        self.style(Colors::DeepBlue, n)
    }

    /// Create a style from the given purple shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn purple(&self, n: usize) -> Style {
        self.style(Colors::Purple, n)
    }

    /// Create a style from the given magenta shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn magenta(&self, n: usize) -> Style {
        self.style(Colors::Magenta, n)
    }

    /// Create a style from the given redpink shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn redpink(&self, n: usize) -> Style {
        self.style(Colors::RedPink, n)
    }

    /// Create a style from the given primary shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn primary(&self, n: usize) -> Style {
        self.style(Colors::Primary, n)
    }

    /// Create a style from the given secondary shade.
    /// n is 0..=7 with 4..=7 as darker variants of the first 3.
    pub fn secondary(&self, n: usize) -> Style {
        self.style(Colors::Secondary, n)
    }
}

impl Palette {
    /// The Color for the id + index n.
    pub fn color(&self, id: Colors, n: usize) -> Color {
        if id == Colors::None {
            Color::Reset
        } else {
            self.color[id as usize][n]
        }
    }

    /// Create a style with the given background color.
    /// The foreground is chosen from the text-colors for a
    /// normal contrast.
    pub fn style(&self, id: Colors, n: usize) -> Style {
        let color = self.color(id, n);
        self.normal_contrast(color)
    }

    /// Create a style with the given background color.
    /// The foreground is chosen from the text-colors for
    /// high contrast.
    pub fn high_style(&self, id: Colors, n: usize) -> Style {
        let color = self.color(id, n);
        self.high_contrast(color)
    }

    /// Create a style with the given fg/bg.
    pub fn fg_bg_style(&self, fg: Colors, n: usize, bg: Colors, m: usize) -> Style {
        let color = self.color(fg, n);
        let color_bg = self.color(bg, m);
        let mut style = Style::new();
        if color != Color::Reset {
            style = style.fg(color);
        }
        if color_bg != Color::Reset {
            style = style.bg(color_bg);
        }
        style
    }

    /// Create a style with only fg set.
    pub fn fg_style(&self, id: Colors, n: usize) -> Style {
        let color = self.color(id, n);
        let mut style = Style::new();
        if color != Color::Reset {
            style = style.fg(color);
        }
        style
    }

    /// Create a style with only bg set.
    pub fn bg_style(&self, id: Colors, n: usize) -> Style {
        let color = self.color(id, n);
        let mut style = Style::new();
        if color != Color::Reset {
            style = style.bg(color);
        }
        style
    }

    /// Add an alias.
    pub fn add_aliased(&mut self, id: &str, color_idx: ColorIdx) {
        if matches!(self.aliased, Cow::Borrowed(_)) {
            self.aliased = Cow::Owned(mem::take(&mut self.aliased).into_owned());
        }
        match &mut self.aliased {
            Cow::Borrowed(_) => {
                unreachable!()
            }
            Cow::Owned(aliased) => {
                aliased.push((Cow::Owned(id.to_string()), color_idx));
                aliased.sort();
            }
        }
    }

    /// Try to find an alias.
    pub fn try_aliased(&self, id: &str) -> Option<ColorIdx> {
        match self.aliased.binary_search_by_key(&id, |v| v.0.as_ref()) {
            Ok(n) => Some(self.aliased[n].1),
            Err(_) => None,
        }
    }

    /// Get the ColorIdx of an aliased color.
    ///
    /// __Panic__
    ///
    /// With debug_assertions this panics if the alias is not found.
    /// Otherwise, it returns a default.
    pub fn aliased(&self, id: &str) -> ColorIdx {
        match self.try_aliased(id) {
            Some(c) => c,
            None => {
                if cfg!(debug_assertions) {
                    panic!("unknown aliased color {:?} in palette {:?}", id, self.name);
                } else {
                    ColorIdx::default()
                }
            }
        }
    }

    /// Get an aliased color.
    pub fn color_alias(&self, id: &str) -> Color {
        match self.try_aliased(id) {
            Some(ColorIdx { 0: c, 1: idx }) => {
                if c != Colors::None {
                    self.color[c as usize][idx]
                } else {
                    Color::default()
                }
            }
            None => {
                if cfg!(debug_assertions) {
                    panic!("unknown aliased color {:?} in palette {:?}", id, self.name);
                } else {
                    Color::default()
                }
            }
        }
    }

    /// Get a Style for a color-alias.
    /// Uses the color as bg() and finds the matching text-color
    /// for normal contrast.
    pub fn style_alias(&self, bg: &str) -> Style {
        let color = self.color_alias(bg);
        self.normal_contrast(color)
    }

    /// Get a Style for a color-alias.
    /// Uses the color as bg() and finds the matching text-color
    /// for high contrast.
    pub fn high_style_alias(&self, bg: &str) -> Style {
        let color = self.color_alias(bg);
        self.high_contrast(color)
    }

    /// Get a Style for a color-alias.
    /// Uses explicit aliases for fg() and bg()
    pub fn fg_bg_style_alias(&self, fg: &str, bg: &str) -> Style {
        let color = self.color_alias(fg);
        let color_bg = self.color_alias(bg);
        let mut style = Style::new();
        if color != Color::Reset {
            style = style.fg(color);
        }
        if color_bg != Color::Reset {
            style = style.bg(color_bg);
        }
        style
    }

    /// Get a Style for a color-alias.
    /// This creates a style with only the fg() color set.
    pub fn fg_style_alias(&self, fg: &str) -> Style {
        let color = self.color_alias(fg);
        let mut style = Style::new();
        if color != Color::Reset {
            style = style.fg(color);
        }
        style
    }

    /// Get a Style for a color-alias.
    /// This creates a style with only the bg() color set.
    pub fn bg_style_alias(&self, bg: &str) -> Style {
        let color = self.color_alias(bg);
        let mut style = Style::new();
        if color != Color::Reset {
            style = style.bg(color);
        }
        style
    }
}

impl Palette {
    /// Create a style with the given background color.
    /// Uses `white[3]` or `black[0]` for the foreground,
    /// based on `rate_text_color`.
    pub fn high_contrast(&self, color: Color) -> Style {
        match Self::rate_text_color(color) {
            None => Style::new(),
            Some(Rating::Light) => Style::new().bg(color).fg(self.color(Colors::TextLight, 3)),
            Some(Rating::Dark) => Style::new().bg(color).fg(self.color(Colors::TextDark, 3)),
        }
    }

    /// Create a style with the given background color.
    /// Uses text_light or text_dark for the foreground,
    /// based on `rate_text_color`.
    pub fn normal_contrast(&self, color: Color) -> Style {
        match Self::rate_text_color(color) {
            None => Style::new(),
            Some(Rating::Light) => Style::new().bg(color).fg(self.color(Colors::TextLight, 0)),
            Some(Rating::Dark) => Style::new().bg(color).fg(self.color(Colors::TextDark, 0)),
        }
    }

    /// Pick a color from the choice with a good contrast to the
    /// given background.
    pub fn normal_contrast_color(&self, bg: Color, text: &[Color]) -> Style {
        if bg == Color::Reset {
            return Style::new();
        }
        let mut color0 = text[0];
        let mut color1 = text[0];
        let mut contrast1 = Self::contrast_bt_srgb(color1, bg);

        for text_color in text {
            let test = Self::contrast_bt_srgb(*text_color, bg);
            if test > contrast1 {
                color0 = color1;
                color1 = *text_color;
                contrast1 = test;
            }
        }

        Style::new().bg(bg).fg(color0)
    }

    /// Pick a color from the choice with the best contrast to the
    /// given background.
    pub fn high_contrast_color(&self, bg: Color, text: &[Color]) -> Style {
        if bg == Color::Reset {
            return Style::new();
        }
        let mut color0 = text[0];
        let mut color1 = text[0];
        let mut contrast1 = Self::contrast_bt_srgb(color1, bg);

        for text_color in text {
            let test = Self::contrast_bt_srgb(*text_color, bg);
            if test > contrast1 {
                color0 = color1;
                color1 = *text_color;
                contrast1 = test;
            }
        }
        // don't use the second brightest.
        _ = color0;

        Style::new().bg(bg).fg(color1)
    }

    // /// Gives the luminance according to Rec.ITU-R BT.601-7.
    // const fn luminance_itu(color: Color) -> f32 {
    //     let (r, g, b) = Self::color2rgb(color);
    //     0.2989f32 * (r as f32) / 255f32
    //         + 0.5870f32 * (g as f32) / 255f32
    //         + 0.1140f32 * (b as f32) / 255f32
    // }
    //
    // /// Gives the luminance according to Rec.ITU-R BT.601-7.
    // fn luminance_itu_srgb(color: Color) -> f32 {
    //     let (r, g, b) = Self::color2rgb(color);
    //     0.2989f32 * (r as f32) / 255f32
    //         + 0.5870f32 * (g as f32) / 255f32
    //         + 0.1140f32 * (b as f32) / 255f32
    // }
    //
    // /// Contrast between two colors.
    // fn contrast_itu_srgb(color: Color, color2: Color) -> f32 {
    //     let lum1 = Self::luminance_itu_srgb(color);
    //     let lum2 = Self::luminance_itu_srgb(color2);
    //     (lum1 + 0.05f32) / (lum2 + 0.05f32)
    // }

    /// Gives the luminance according to BT.709.
    pub(crate) const fn luminance_bt(color: Color) -> f32 {
        let (r, g, b) = Self::color_to_rgb(color);
        0.2126f32 * ((r as f32) / 255f32)
            + 0.7152f32 * ((g as f32) / 255f32)
            + 0.0722f32 * ((b as f32) / 255f32)
    }

    /// Gives the luminance according to BT.709.
    pub(crate) fn luminance_bt_srgb(color: Color) -> f32 {
        let (r, g, b) = Self::color_to_rgb(color);
        0.2126f32 * ((r as f32) / 255f32).powf(2.2f32)
            + 0.7152f32 * ((g as f32) / 255f32).powf(2.2f32)
            + 0.0722f32 * ((b as f32) / 255f32).powf(2.2f32)
    }

    /// Contrast between two colors.
    pub(crate) fn contrast_bt_srgb(color: Color, color2: Color) -> f32 {
        let lum1 = Self::luminance_bt_srgb(color);
        let lum2 = Self::luminance_bt_srgb(color2);
        (lum1 - lum2).abs()
        // Don't use this prescribed method.
        // The abs diff comes out better.
        // (lum1 + 0.05f32) / (lum2 + 0.05f32)
    }

    /// This gives back a [Rating] for the given background.
    ///
    /// This converts RGB to grayscale and takes the grayscale value
    /// of VGA cyan as threshold, which is about 105 out of 255.
    /// This point is a bit arbitrary, just based on what I
    /// perceive as acceptable. But it produces a good reading
    /// contrast in my experience.
    ///
    /// For the named colors it takes the VGA equivalent as a base.
    /// For indexed colors it splits the range in half as an estimate.
    pub(crate) fn rate_text_color(color: Color) -> Option<Rating> {
        match color {
            Color::Reset => None,
            Color::Black => Some(Rating::Light),       //0
            Color::Red => Some(Rating::Light),         //1
            Color::Green => Some(Rating::Light),       //2
            Color::Yellow => Some(Rating::Light),      //3
            Color::Blue => Some(Rating::Light),        //4
            Color::Magenta => Some(Rating::Light),     //5
            Color::Cyan => Some(Rating::Light),        //6
            Color::Gray => Some(Rating::Dark),         //7
            Color::DarkGray => Some(Rating::Light),    //8
            Color::LightRed => Some(Rating::Dark),     //9
            Color::LightGreen => Some(Rating::Dark),   //10
            Color::LightYellow => Some(Rating::Dark),  //11
            Color::LightBlue => Some(Rating::Light),   //12
            Color::LightMagenta => Some(Rating::Dark), //13
            Color::LightCyan => Some(Rating::Dark),    //14
            Color::White => Some(Rating::Dark),        //15
            c => {
                let lum = Self::luminance_bt(c);
                if lum >= 0.4117f32 {
                    Some(Rating::Dark)
                } else {
                    Some(Rating::Light)
                }
            }
        }
    }

    /// Converts the given color to an equivalent grayscale.
    pub const fn grayscale(color: Color) -> Color {
        let lum = Self::luminance_bt(color);
        let gray = lum * 255f32;
        Color::Rgb(gray as u8, gray as u8, gray as u8)
    }

    /// Color from u32
    pub const fn color_from_u32(c: u32) -> Color {
        let r0 = (c >> 16) as u8;
        let g0 = (c >> 8) as u8;
        let b0 = c as u8;
        Color::Rgb(r0, g0, b0)
    }

    /// Color to u32
    pub const fn color_to_u32(color: Color) -> u32 {
        let (r, g, b) = Self::color_to_rgb(color);
        ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
    }

    /// Calculates a linear interpolation for the two colors
    /// and fills the first 4 colors with it.
    /// The next 4 colors are scaled down versions using dark_scale_to.
    pub const fn interpolatec(c0: Color, c3: Color, dark_scale_to: u8) -> [Color; 8] {
        Self::interpolate(
            Self::color_to_u32(c0),
            Self::color_to_u32(c3),
            dark_scale_to,
        )
    }

    /// Calculates a linear interpolation for the two colors
    /// and fills the first 4 colors with it.
    /// The next 4 colors are scaled down versions using dark_scale_to.
    pub const fn interpolate(c0: u32, c3: u32, dark_scale_to: u8) -> [Color; 8] {
        // dark
        let mut c4 = Self::color_to_rgb(Self::color_from_u32(c0));
        c4.0 = Self::scale_to(c4.0, dark_scale_to);
        c4.1 = Self::scale_to(c4.1, dark_scale_to);
        c4.2 = Self::scale_to(c4.2, dark_scale_to);
        let c4 = ((c4.0 as u32) << 16) + ((c4.1 as u32) << 8) + (c4.2 as u32);

        let mut c7 = Self::color_to_rgb(Self::color_from_u32(c3));
        c7.0 = Self::scale_to(c7.0, dark_scale_to);
        c7.1 = Self::scale_to(c7.1, dark_scale_to);
        c7.2 = Self::scale_to(c7.2, dark_scale_to);
        let c7 = ((c7.0 as u32) << 16) + ((c7.1 as u32) << 8) + (c7.2 as u32);

        Self::interpolate2(c0, c3, c4, c7)
    }

    /// Calculates a linear interpolation for the two colors
    /// and fills the first 4 colors with it.
    /// The next 4 colors are scaled down versions using dark_scale_to.
    pub const fn interpolatec2(c0: Color, c3: Color, c4: Color, c7: Color) -> [Color; 8] {
        Self::interpolate2(
            Self::color_to_u32(c0),
            Self::color_to_u32(c3),
            Self::color_to_u32(c4),
            Self::color_to_u32(c7),
        )
    }

    /// Calculates a linear interpolation for the two colors
    /// and fills the first 4 colors with it.
    /// The next 4 colors are scaled down versions using dark_scale_to.
    pub const fn interpolate2(c0: u32, c3: u32, c4: u32, c7: u32) -> [Color; 8] {
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

        let r3 = (c3 >> 16) as u8;
        let g3 = (c3 >> 8) as u8;
        let b3 = c3 as u8;

        let r1 = i1(r0, r3);
        let g1 = i1(g0, g3);
        let b1 = i1(b0, b3);

        let r2 = i2(r0, r3);
        let g2 = i2(g0, g3);
        let b2 = i2(b0, b3);

        // dark
        let r4 = (c4 >> 16) as u8;
        let g4 = (c4 >> 8) as u8;
        let b4 = c4 as u8;

        let r7 = (c7 >> 16) as u8;
        let g7 = (c7 >> 8) as u8;
        let b7 = c7 as u8;

        let r5 = i1(r4, r7);
        let g5 = i1(g4, g7);
        let b5 = i1(b4, b7);

        let r6 = i2(r4, r7);
        let g6 = i2(g4, g7);
        let b6 = i2(b4, b7);

        [
            Color::Rgb(r0, g0, b0),
            Color::Rgb(r1, g1, b1),
            Color::Rgb(r2, g2, b2),
            Color::Rgb(r3, g3, b3),
            Color::Rgb(r4, g4, b4),
            Color::Rgb(r5, g5, b5),
            Color::Rgb(r6, g6, b6),
            Color::Rgb(r7, g7, b7),
        ]
    }

    /// Scale the u8 down to scale_to.
    pub const fn scale_to(v: u8, scale_to: u8) -> u8 {
        (((v as u16) * scale_to as u16) / 255u16) as u8
    }

    /// Gives back the rgb for any ratatui Color.
    /// Has the indexed and the named colors too.
    pub const fn color_to_rgb(color: Color) -> (u8, u8, u8) {
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
}
