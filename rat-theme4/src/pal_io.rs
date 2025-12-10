//!
//! Allows load/store to an ini-style format.
//! Serde for Palette is supported as well.
//!

use crate::error::LoadPaletteErr;
use crate::palette;
use crate::palette::{Colors, Palette};
use ratatui::style::Color;
use std::borrow::Cow;
use std::{array, io};

/// Stora a Palette as a .pal file.
pub fn store_palette(pal: &Palette, mut buf: impl io::Write) -> Result<(), io::Error> {
    writeln!(buf, "[theme]")?;
    writeln!(buf, "name={}", pal.theme_name)?;
    writeln!(buf, "theme={}", pal.theme)?;
    writeln!(buf)?;
    writeln!(buf, "[palette]")?;
    writeln!(buf, "name={}", pal.name)?;
    writeln!(buf, "docs={}", pal.doc.replace('\n', "\\n"))?;
    writeln!(buf, "generator={}", pal.generator)?;
    writeln!(buf,)?;
    writeln!(buf, "[color]")?;
    if pal.generator.starts_with("light-dark") {
        for c in Colors::array() {
            writeln!(
                buf,
                "{}={}, {}",
                *c, pal.color[*c as usize][0], pal.color[*c as usize][3]
            )?;
        }
    } else if pal.generator.starts_with("color-1") {
        for c in Colors::array() {
            writeln!(buf, "{}={}", *c, pal.color[*c as usize][0])?;
        }
    } else if pal.generator.starts_with("color-2") {
        for c in Colors::array() {
            writeln!(
                buf,
                "{}={}, {}",
                *c, pal.color[*c as usize][0], pal.color[*c as usize][4]
            )?;
        }
    } else if pal.generator.starts_with("color-4") {
        for c in Colors::array() {
            writeln!(
                buf,
                "{}={}, {}, {}, {}",
                *c,
                pal.color[*c as usize][0],
                pal.color[*c as usize][1],
                pal.color[*c as usize][2],
                pal.color[*c as usize][3]
            )?;
        }
    } else if pal.generator.starts_with("color-4-dark") {
        for c in Colors::array() {
            writeln!(
                buf,
                "{}={}, {}, {}, {}",
                *c,
                pal.color[*c as usize][0],
                pal.color[*c as usize][1],
                pal.color[*c as usize][2],
                pal.color[*c as usize][3]
            )?;
        }
    } else if pal.generator.starts_with("color-8") {
        for c in Colors::array() {
            writeln!(
                buf,
                "{}={}, {}, {}, {}, {}, {}, {}, {}",
                *c,
                pal.color[*c as usize][0],
                pal.color[*c as usize][1],
                pal.color[*c as usize][2],
                pal.color[*c as usize][3],
                pal.color[*c as usize][4],
                pal.color[*c as usize][5],
                pal.color[*c as usize][6],
                pal.color[*c as usize][7],
            )?;
        }
    } else {
        return Err(io::Error::other(LoadPaletteErr(format!(
            "Invalid generator format {:?}",
            pal.generator
        ))));
    }
    writeln!(buf,)?;
    writeln!(buf, "[reference]")?;
    for (r, i) in pal.aliased.as_ref() {
        writeln!(buf, "{}={}", r, i)?;
    }
    Ok(())
}

/// Load a .pal file as a Palette.
pub fn load_palette(mut r: impl io::Read) -> Result<Palette, io::Error> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;

    enum S {
        Start,
        Theme,
        Palette,
        Color,
        Reference,
        Fail(String),
    }

    let mut pal = Palette::default();
    let mut dark = 63u8;

    let mut state = S::Start;
    'm: for l in buf.lines() {
        let l = l.trim();
        match state {
            S::Start => {
                if l == "[theme]" {
                    state = S::Theme;
                } else if l == "[palette]" {
                    state = S::Palette;
                } else {
                    state = S::Fail("No a valid pal-file".to_string());
                    break 'm;
                }
            }
            S::Theme => {
                if l == "[palette]" {
                    state = S::Palette;
                } else if l.is_empty() || l.starts_with("#") {
                    // ok
                } else if l.starts_with("name") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.theme_name = Cow::Owned(s.trim().to_string());
                    }
                } else if l.starts_with("theme") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.theme = Cow::Owned(s.trim().to_string());
                    }
                } else {
                    state = S::Fail(format!("Invalid theme property {:?}", l));
                    break 'm;
                }
            }
            S::Palette => {
                if l == "[color]" {
                    state = S::Color;
                } else if l.is_empty() || l.starts_with("#") {
                    // ok
                } else if l.starts_with("name") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.name = Cow::Owned(s.trim().to_string());
                    }
                } else if l.starts_with("docs") {
                    if let Some(s) = l.split('=').nth(1) {
                        let doc = s.trim().replace("\\n", "\n");
                        pal.doc = Cow::Owned(doc);
                    }
                } else if l.starts_with("generator") {
                    if let Some(s) = l.split('=').nth(1) {
                        pal.generator = Cow::Owned(s.trim().to_string());
                        if s.starts_with("light-dark") {
                            if let Some(s) = l.split(':').nth(1) {
                                dark = s.trim().parse::<u8>().unwrap_or(63);
                            }
                        } else if s.starts_with("color-1") {
                        } else if s.starts_with("color-2") {
                        } else if s.starts_with("color-4") {
                        } else if s.starts_with("color-4-dark") {
                            if let Some(s) = l.split(':').nth(1) {
                                dark = s.trim().parse::<u8>().unwrap_or(63);
                            }
                        } else if s.starts_with("color-8") {
                        } else {
                            state = S::Fail(format!("Unknown generator format {:?}", s));
                            break 'm;
                        }
                    }
                } else if l.starts_with("dark") {
                    if let Some(s) = l.split('=').nth(1) {
                        if let Ok(v) = s.trim().parse::<u8>() {
                            dark = v;
                        } else {
                            // skip
                        }
                    }
                } else {
                    state = S::Fail(format!("Invalid palette property {:?}", l));
                    break 'm;
                }
            }
            S::Color => {
                if l == "[reference]" {
                    state = S::Reference;
                } else if l.is_empty() || l.starts_with("#") {
                    // ok
                } else {
                    let mut kv = l.split('=');
                    let cn = if let Some(v) = kv.next() {
                        let Ok(c) = v.trim().parse::<Colors>() else {
                            state = S::Fail(format!("Invalid property format {:?}", l));
                            break 'm;
                        };
                        c
                    } else {
                        state = S::Fail(format!("Invalid property format {:?}", l));
                        break 'm;
                    };
                    if let Some(v) = kv.next() {
                        if pal.generator.starts_with("light-dark") {
                            let color = split_comma::<2>(v)?;
                            if cn == Colors::TextLight || cn == Colors::TextDark {
                                pal.color[cn as usize] = Palette::interpolatec2(
                                    color[0],
                                    color[1],
                                    Color::default(),
                                    Color::default(),
                                )
                            } else {
                                pal.color[cn as usize] =
                                    Palette::interpolatec(color[0], color[1], dark);
                            }
                        } else if pal.generator.starts_with("color-1") {
                            let color = split_comma::<1>(v)?;
                            pal.color[cn as usize] = array::from_fn(|_| color[0]);
                        } else if pal.generator.starts_with("color-2") {
                            let color = split_comma::<2>(v)?;
                            pal.color[cn as usize][0..=3]
                                .copy_from_slice(&array::from_fn::<_, 4, _>(|_| color[0]));
                            pal.color[cn as usize][4..=7]
                                .copy_from_slice(&array::from_fn::<_, 4, _>(|_| color[0]));
                        } else if pal.generator.starts_with("color-4") {
                            let color = split_comma::<4>(v)?;
                            pal.color[cn as usize][0..=3].copy_from_slice(&color);
                            pal.color[cn as usize][4..=7].copy_from_slice(&color);
                        } else if pal.generator.starts_with("color-4-dark") {
                            let color = split_comma::<4>(v)?;
                            pal.color[cn as usize][0..=3].copy_from_slice(&color);
                            pal.color[cn as usize][4] =
                                Palette::scale_color_to(pal.color[cn as usize][0], dark);
                            pal.color[cn as usize][5] =
                                Palette::scale_color_to(pal.color[cn as usize][1], dark);
                            pal.color[cn as usize][6] =
                                Palette::scale_color_to(pal.color[cn as usize][2], dark);
                            pal.color[cn as usize][7] =
                                Palette::scale_color_to(pal.color[cn as usize][3], dark);
                        } else if pal.generator.starts_with("color-8") {
                            let color = split_comma::<8>(v)?;
                            pal.color[cn as usize] = color;
                        }
                    } else {
                        state = S::Fail(format!("Invalid property format {:?}", l));
                        break 'm;
                    };
                }
            }
            S::Reference => {
                let mut kv = l.split('=');
                let rn = if let Some(v) = kv.next() {
                    v
                } else {
                    state = S::Fail(format!("Invalid property format {:?}", l));
                    break 'm;
                };
                let ci = if let Some(v) = kv.next() {
                    if let Ok(ci) = v.parse::<palette::ColorIdx>() {
                        ci
                    } else {
                        state = S::Fail(format!("Invalid color reference {:?}", l));
                        break 'm;
                    }
                } else {
                    state = S::Fail(format!("Invalid property format {:?}", l));
                    break 'm;
                };
                pal.add_aliased(rn, ci);
            }
            S::Fail(_) => {
                unreachable!()
            }
        }
    }

    match state {
        S::Fail(n) => Err(io::Error::other(LoadPaletteErr(n))),
        S::Start => Err(io::Error::other(LoadPaletteErr(
            "Missing [theme]. Invalid format or truncated.".to_string(),
        ))),
        S::Theme => Err(io::Error::other(LoadPaletteErr(
            "Missing [palette]. Invalid format or truncated.".to_string(),
        ))),
        S::Palette => Err(io::Error::other(LoadPaletteErr(
            "Missing [reference]. Invalid format or truncated.".to_string(),
        ))),
        S::Color | S::Reference => Ok(pal),
    }
}

fn split_comma<const N: usize>(s: &str) -> Result<[Color; N], io::Error> {
    let mut r: [Color; N] = array::from_fn(|_| Color::default());
    let mut vv = s.split(',');
    for i in 0..N {
        r[i] = if let Some(v) = vv.next() {
            let Ok(v) = v.trim().parse::<Color>() else {
                return Err(io::Error::other(LoadPaletteErr(
                    format!("Invalid color[{}] {:?}", i, s).to_string(),
                )));
            };
            v
        } else {
            return Err(io::Error::other(LoadPaletteErr(
                format!("Invalid color[{}] {:?}", i, s).to_string(),
            )));
        }
    }

    if let Some(v) = vv.next()
        && !v.trim().is_empty()
    {
        return Err(io::Error::other(LoadPaletteErr(
            format!("Too many colors (max {}) {:?}", N, s).to_string(),
        )));
    }

    Ok(r)
}
