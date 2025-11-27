use configparser::ini::{IniDefault, WriteOptions};
use std::fmt::Debug;
use std::mem;
use std::mem::MaybeUninit;
use std::path::Path;
use std::str::FromStr;

/// Extensions to configparser for ease of use.
#[allow(dead_code)]
pub(crate) trait ConfigParserExt {
    fn new_std() -> Self;

    /// Get multiline text.
    fn get_text<S: AsRef<str>, D: Into<String>>(
        &self, //
        sec: S,
        key: &str,
        default: D,
    ) -> String;

    fn parse_array_v4<const N: usize, T: Copy + FromStr, S: AsRef<str>>(
        &mut self,
        sec: S,
        key: &str,
        default: T,
    ) -> [T; N];

    fn parse_array<const N: usize, T: Copy + FromStr + Debug, S: AsRef<str>>(
        &mut self,
        sec: S,
        key: &str,
        default: T,
    ) -> [T; N];

    /// Call parse() for the value.
    fn parse_val<T: FromStr, S: AsRef<str>>(
        &self, //
        sec: S,
        key: &str,
        default: T,
    ) -> T;

    /// Set from some type.
    fn set_val<T: ToString, S: AsRef<str>>(
        &mut self, //
        sec: S,
        key: &str,
        val: T,
    );

    fn set_array<const N: usize, T: Copy + ToString, S: AsRef<str>>(
        &mut self,
        sec: S,
        key: &str,
        val: [T; N],
    );

    fn set_text<T: ToString, S: AsRef<str>>(
        &mut self, //
        sec: S,
        key: &str,
        val: T,
    );

    /// Write with our standards.
    fn write_std(&self, path: impl AsRef<Path>) -> std::io::Result<()>;
}

impl ConfigParserExt for configparser::ini::Ini {
    fn new_std() -> Self {
        let mut def = IniDefault::default();
        def.case_sensitive = true;
        def.multiline = false;
        def.comment_symbols = vec![];

        configparser::ini::Ini::new_from_defaults(def)
    }

    fn get_text<S: AsRef<str>, D: Into<String>>(&self, sec: S, key: &str, default: D) -> String {
        if let Some(s) = self.get(sec.as_ref(), key) {
            let mut buf = String::new();
            let mut esc = false;
            for c in s.chars() {
                if c == '\\' {
                    if esc {
                        buf.push('\\');
                        esc = false;
                    } else {
                        esc = true;
                    }
                } else if esc {
                    match c {
                        'r' => buf.push('\r'),
                        'n' => buf.push('\n'),
                        _ => {
                            buf.push('\\');
                            buf.push(c);
                        }
                    }
                    esc = false;
                } else {
                    buf.push(c);
                }
            }
            buf
        } else {
            default.into()
        }
    }

    fn parse_array_v4<const N: usize, T: Copy + FromStr, S: AsRef<str>>(
        &mut self,
        sec: S,
        key: &str,
        default: T,
    ) -> [T; N] {
        let sec = sec.as_ref();
        let mut r = [MaybeUninit::uninit(); N];
        for (i, v) in r.iter_mut().enumerate() {
            v.write(self.parse_val(sec, format!("{}.{}", key, i).as_str(), default));
        }
        // Everything is initialized. Transmute the array to the
        // initialized type.
        unsafe { mem::transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&r) }
    }

    fn parse_array<const N: usize, T: Copy + FromStr + Debug, S: AsRef<str>>(
        &mut self,
        sec: S,
        key: &str,
        default: T,
    ) -> [T; N] {
        let sec = sec.as_ref();

        let mut r = [MaybeUninit::uninit(); N];

        let val_str = self.get_text(sec, key, "");
        let mut val_str = val_str.split(',');
        for i in 0..N {
            if let Some(v) = val_str.next() {
                let v = v.trim();
                match v.parse::<T>() {
                    Ok(v) => {
                        r[i] = MaybeUninit::new(v);
                    }
                    Err(_) => {
                        r[i] = MaybeUninit::new(default);
                    }
                }
            } else {
                r[i] = MaybeUninit::new(default);
            }
        }
        // Everything is initialized. Transmute the array to the
        // initialized type.
        unsafe { mem::transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&r) }
    }

    fn parse_val<T: FromStr, S: AsRef<str>>(
        &self, //
        sec: S,
        key: &str,
        default: T,
    ) -> T {
        if let Some(v) = self.get(sec.as_ref(), key) {
            v.parse::<T>().unwrap_or(default)
        } else {
            default
        }
    }

    fn set_val<T: ToString, S: AsRef<str>>(&mut self, sec: S, key: &str, val: T) {
        self.set(sec.as_ref(), key, Some(val.to_string()));
    }

    fn set_array<const N: usize, T: Copy + ToString, S: AsRef<str>>(
        &mut self,
        sec: S,
        key: &str,
        val: [T; N],
    ) {
        let sec = sec.as_ref();
        let mut val_str = String::new();
        for i in 0..N {
            if i > 0 {
                val_str.push_str(", ");
            }
            val_str.push_str(&val[i].to_string());
        }
        self.set_text(sec, key, val_str);
    }

    fn set_text<T: ToString, S: AsRef<str>>(&mut self, sec: S, key: &str, val: T) {
        let mut buf = String::new();
        for c in val.to_string().chars() {
            if c == '\r' {
                // skip
            } else if c == '\n' {
                buf.push_str("\\n");
            } else {
                buf.push(c)
            }
        }
        self.set(sec.as_ref(), key, Some(buf));
    }

    fn write_std(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        self.pretty_write(path, &WriteOptions::new_with_params(false, 4, 1))
    }
}
