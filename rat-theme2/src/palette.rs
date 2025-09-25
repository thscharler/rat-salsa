use ratatui::style::{Color, Style};

/// Color palette.
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
/// * Background colors need extra considerations. Extend to 8.
///
#[derive(Debug, Default, Clone)]
pub struct Palette {
    pub white: [Color; 8],
    pub black: [Color; 8],
    pub gray: [Color; 8],

    pub red: [Color; 8],
    pub orange: [Color; 8],
    pub yellow: [Color; 8],
    pub limegreen: [Color; 8],
    pub green: [Color; 8],
    pub bluegreen: [Color; 8],
    pub cyan: [Color; 8],
    pub blue: [Color; 8],
    pub deepblue: [Color; 8],
    pub purple: [Color; 8],
    pub magenta: [Color; 8],
    pub redpink: [Color; 8],

    pub primary: [Color; 8],
    pub secondary: [Color; 8],
}

/// Contrast rating for the text-color that should be used.
#[derive(Debug)]
pub enum TextColorRating {
    /// Use light/white text for the given background.
    Light,
    /// Use dark/black text for the given background.
    Dark,
}

/// Used to create a high contrast or normal contrast style.
#[derive(Debug)]
pub enum Contrast {
    High,
    Normal,
}

impl Palette {
    /// Color index for a bright variant of the base color.
    /// Brightness increases with the number.
    pub const BRIGHT_0: usize = 0;
    /// Color index for a bright variant of the base color.
    /// Brightness increases with the number.
    pub const BRIGHT_1: usize = 1;
    /// Color index for a bright variant of the base color.
    /// Brightness increases with the number.
    pub const BRIGHT_2: usize = 2;
    /// Color index for a bright variant of the base color.
    /// Brightness increases with the number.
    pub const BRIGHT_3: usize = 3;
    /// Color index for a dark variant of the base color.
    /// Brightness increases with the number.
    pub const DARK_0: usize = 4;
    /// Color index for a dark variant of the base color.
    /// Brightness increases with the number.
    pub const DARK_1: usize = 5;
    /// Color index for a dark variant of the base color.
    /// Brightness increases with the number.
    pub const DARK_2: usize = 6;
    /// Color index for a dark variant of the base color.
    /// Brightness increases with the number.
    pub const DARK_3: usize = 7;

    /// Create a style from the given white shade.
    /// n is `0..=3`
    pub fn white(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.white[n], contrast)
    }

    /// Create a style from the given black shade.
    /// n is `0..=3`
    pub fn black(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.black[n], contrast)
    }

    /// Create a style from the given gray shade.
    /// n is `0..=3`
    pub fn gray(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.gray[n], contrast)
    }

    /// Create a style from the given red shade.
    /// n is `0..=3`
    pub fn red(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.red[n], contrast)
    }

    /// Create a style from the given orange shade.
    /// n is `0..=3`
    pub fn orange(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.orange[n], contrast)
    }

    /// Create a style from the given yellow shade.
    /// n is `0..=3`
    pub fn yellow(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.yellow[n], contrast)
    }

    /// Create a style from the given limegreen shade.
    /// n is `0..=3`
    pub fn limegreen(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.limegreen[n], contrast)
    }

    /// Create a style from the given green shade.
    /// n is `0..=3`
    pub fn green(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.green[n], contrast)
    }

    /// Create a style from the given bluegreen shade.
    /// n is `0..=3`
    pub fn bluegreen(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.bluegreen[n], contrast)
    }

    /// Create a style from the given cyan shade.
    /// n is `0..=3`
    pub fn cyan(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.cyan[n], contrast)
    }

    /// Create a style from the given blue shade.
    /// n is `0..=3`
    pub fn blue(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.blue[n], contrast)
    }

    /// Create a style from the given deepblue shade.
    /// n is `0..=3`
    pub fn deepblue(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.deepblue[n], contrast)
    }

    /// Create a style from the given purple shade.
    /// n is `0..=3`
    pub fn purple(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.purple[n], contrast)
    }

    /// Create a style from the given magenta shade.
    /// n is `0..=3`
    pub fn magenta(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.magenta[n], contrast)
    }

    /// Create a style from the given redpink shade.
    /// n is `0..=3`
    pub fn redpink(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.redpink[n], contrast)
    }

    /// Create a style from the given primary shade.
    /// n is `0..=3`
    pub fn primary(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.primary[n], contrast)
    }

    /// Create a style from the given secondary shade.
    /// n is `0..=3`
    pub fn secondary(&self, n: usize, contrast: Contrast) -> Style {
        self.style(self.secondary[n], contrast)
    }
}

impl Palette {
    /// Create a style with the given background color and
    /// contrast.
    pub fn style(&self, color: Color, contrast: Contrast) -> Style {
        match contrast {
            Contrast::High => self.high_contrast(color),
            Contrast::Normal => self.normal_contrast(color),
        }
    }

    /// Create a style with the given background color.
    /// Uses `white[3]` or `black[0]` for the foreground,
    /// based on `rate_text_color`.
    pub fn high_contrast(&self, color: Color) -> Style {
        match Self::rate_text_color(color) {
            None => Style::reset(),
            Some(TextColorRating::Light) => Style::new().bg(color).fg(self.white[3]),
            Some(TextColorRating::Dark) => Style::new().bg(color).fg(self.black[0]),
        }
    }

    /// Create a style with the given background color.
    /// Uses `white[0]` or `black[3]` for the foreground,
    /// based on `rate_text_color`.
    pub fn normal_contrast(&self, color: Color) -> Style {
        match Self::rate_text_color(color) {
            None => Style::reset(),
            Some(TextColorRating::Light) => Style::new().bg(color).fg(self.white[0]),
            Some(TextColorRating::Dark) => Style::new().bg(color).fg(self.black[3]),
        }
    }

    /// This gives back a [TextColorRating] for the given background.
    ///
    /// This converts RGB to grayscale and takes the grayscale value
    /// of VGA cyan as threshold, which is about 105 out of 255.
    /// This point is a bit arbitrary, just based on what I
    /// perceive as acceptable. But it produces a good reading
    /// contrast in my experience.
    ///
    /// For the named colors it takes the VGA equivalent as a base.
    /// For indexed colors it splits the range in half as an estimate.
    pub const fn rate_text_color(color: Color) -> Option<TextColorRating> {
        match color {
            Color::Reset => None,
            Color::Black => Some(TextColorRating::Light), //0
            Color::Red => Some(TextColorRating::Light),   //1
            Color::Green => Some(TextColorRating::Light), //2
            Color::Yellow => Some(TextColorRating::Light), //3
            Color::Blue => Some(TextColorRating::Light),  //4
            Color::Magenta => Some(TextColorRating::Light), //5
            Color::Cyan => Some(TextColorRating::Light),  //6
            Color::Gray => Some(TextColorRating::Dark),   //7
            Color::DarkGray => Some(TextColorRating::Light), //8
            Color::LightRed => Some(TextColorRating::Dark), //9
            Color::LightGreen => Some(TextColorRating::Dark), //10
            Color::LightYellow => Some(TextColorRating::Dark), //11
            Color::LightBlue => Some(TextColorRating::Light), //12
            Color::LightMagenta => Some(TextColorRating::Dark), //13
            Color::LightCyan => Some(TextColorRating::Dark), //14
            Color::White => Some(TextColorRating::Dark),  //15
            Color::Rgb(r, g, b) => {
                // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
                // That's also Rec.ITU-R BT.601-7
                // let grey = r as f32 * 0.2989f32 + g as f32 * 0.5870f32 + b as f32 * 0.1140f32;

                // Another standard BT.709. Works better with green.
                let grey = r as f32 * 0.2126f32 + g as f32 * 0.7152f32 + b as f32 * 0.0722f32;
                if grey >= 105f32 {
                    Some(TextColorRating::Dark)
                } else {
                    Some(TextColorRating::Light)
                }
            }
            Color::Indexed(n) => match n {
                0..=6 => Some(TextColorRating::Light),
                7 => Some(TextColorRating::Dark),
                8 => Some(TextColorRating::Light),
                9..=11 => Some(TextColorRating::Dark),
                12 => Some(TextColorRating::Light),
                13..=15 => Some(TextColorRating::Dark),
                v @ 16..=231 => {
                    if (v - 16) % 36 < 18 {
                        Some(TextColorRating::Light)
                    } else {
                        Some(TextColorRating::Dark)
                    }
                }
                v @ 232..=255 => {
                    if (v - 232) % 24 < 12 {
                        Some(TextColorRating::Light)
                    } else {
                        Some(TextColorRating::Dark)
                    }
                }
            },
        }
    }

    /// Reduces the range of the given color from 0..255
    /// to 0..scale_to.
    ///
    /// This gives a true dark equivalent which can be used
    /// as a background for a dark theme.
    pub const fn darken(color: Color, scale_to: u8) -> Color {
        let (r, g, b) = Self::color2rgb(color);
        Color::Rgb(
            Self::scale_to(r, scale_to),
            Self::scale_to(g, scale_to),
            Self::scale_to(b, scale_to),
        )
    }

    /// Converts the given color to an equivalent grayscale.
    pub const fn grayscale(color: Color) -> Color {
        let (r, g, b) = Self::color2rgb(color);
        // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
        let gray = r as f32 * 0.3f32 + g as f32 * 0.59f32 + b as f32 * 0.11f32;
        Color::Rgb(gray as u8, gray as u8, gray as u8)
    }

    /// Calculates a linear interpolation for the two colors
    /// and fills the first 4 colors with it.
    /// The next 4 colors are scaled down versions using dark_scale_to.
    pub const fn interpolate(c0: u32, c1: u32, dark_scale_to: u8) -> [Color; 8] {
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

        // dark
        let r4 = Self::scale_to(r0, dark_scale_to);
        let g4 = Self::scale_to(g0, dark_scale_to);
        let b4 = Self::scale_to(b0, dark_scale_to);

        let r5 = Self::scale_to(r1, dark_scale_to);
        let g5 = Self::scale_to(g1, dark_scale_to);
        let b5 = Self::scale_to(b1, dark_scale_to);

        let r6 = Self::scale_to(r2, dark_scale_to);
        let g6 = Self::scale_to(g2, dark_scale_to);
        let b6 = Self::scale_to(b2, dark_scale_to);

        let r7 = Self::scale_to(r3, dark_scale_to);
        let g7 = Self::scale_to(g3, dark_scale_to);
        let b7 = Self::scale_to(b3, dark_scale_to);

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
    pub const fn color2rgb(color: Color) -> (u8, u8, u8) {
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
