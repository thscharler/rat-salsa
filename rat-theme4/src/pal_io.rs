//!
//! Allows load/store to an ini-style format.
//! Serde for Palette is supported as well.
//!

use crate::error::LoadPaletteErr;
use crate::palette;
use crate::palette::{Colors, Palette};
use ratatui::style::Color;
use std::borrow::Cow;
use std::io;

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
    for c in Colors::array() {
        writeln!(
            buf,
            "{}={}, {}",
            *c, pal.color[*c as usize][0], pal.color[*c as usize][3]
        )?;
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
        Fail(u8),
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
                    state = S::Fail(1);
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
                    state = S::Fail(2);
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
                    state = S::Fail(3);
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
                            state = S::Fail(4);
                            break 'm;
                        };
                        c
                    } else {
                        state = S::Fail(5);
                        break 'm;
                    };
                    let (c0, c3) = if let Some(v) = kv.next() {
                        let mut vv = v.split(',');
                        let c0 = if let Some(v) = vv.next() {
                            let Ok(v) = v.trim().parse::<Color>() else {
                                state = S::Fail(6);
                                break 'm;
                            };
                            v
                        } else {
                            state = S::Fail(7);
                            break 'm;
                        };
                        let c3 = if let Some(v) = vv.next() {
                            let Ok(v) = v.trim().parse::<Color>() else {
                                state = S::Fail(8);
                                break 'm;
                            };
                            v
                        } else {
                            state = S::Fail(9);
                            break 'm;
                        };
                        (c0, c3)
                    } else {
                        state = S::Fail(10);
                        break 'm;
                    };

                    if cn == Colors::TextLight || cn == Colors::TextDark {
                        pal.color[cn as usize] =
                            Palette::interpolatec2(c0, c3, Color::default(), Color::default())
                    } else {
                        pal.color[cn as usize] = Palette::interpolatec(c0, c3, dark);
                    }
                }
            }
            S::Reference => {
                let mut kv = l.split('=');
                let rn = if let Some(v) = kv.next() {
                    v
                } else {
                    state = S::Fail(11);
                    break 'm;
                };
                let ci = if let Some(v) = kv.next() {
                    if let Ok(ci) = v.parse::<palette::ColorIdx>() {
                        ci
                    } else {
                        state = S::Fail(12);
                        break 'm;
                    }
                } else {
                    state = S::Fail(13);
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
        S::Start => Err(io::Error::other(LoadPaletteErr(100))),
        S::Theme => Err(io::Error::other(LoadPaletteErr(101))),
        S::Palette => Err(io::Error::other(LoadPaletteErr(102))),
        S::Color | S::Reference => Ok(pal),
    }
}
