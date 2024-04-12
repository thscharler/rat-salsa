//! Number formatting.
//!
//! This one uses a pattern string instead of the `format!` style.
//!
//! ```
//! use std::rc::Rc;
//! use pure_rust_locales::Locale::de_AT_euro;
//! use rat_salsa::number;
//! use rat_salsa::number::{NumberFormat, NumberSymbols};
//!
//! // formats accordingly, uses the default symbols.
//! let s = number::format(4561.2234, "###,##0.00").expect("works");
//! assert_eq!(s, "  4,561.22");
//!
//! // uses symbols
//! let sym = NumberSymbols::monetary(de_AT_euro);
//! let s = number::format(4561.2234, "$ ###,##0.00").expect("works");
//! assert_eq!(s.as_str(), "€   4.561,22");
//!
//! // prepared format
//! let sym = Rc::new(NumberSymbols::monetary(de_AT_euro));
//! let m2 = NumberFormat::news("$ ###,##0.00", &sym).expect("works");
//!
//! let s = m2.fmt(4561.2234);
//!
//! use crate::rat_salsa::number::FormatNumber;
//! println!("combined output: {}", 4561.2234f64.fmt(&m2));
//!
//! ```
//!
//! The following patterns are recognized:
//! * `0` - digit or 0
//! * `#` - digit or sign or space
//! * `-` - sign; show space for positive
//! * `.` - decimal separator
//! * `:` - decimal separator, always shown
//! * `,` - grouping separator
//! * `E` - upper case exponent
//! * `e` - lower case exponent
//! * ` ` - space can be used as separator
//! * '$' - currency. repeat for multi-char currency symbols.
//! * `\` - all ascii characters (ascii 32-128!) are reserved and must be escaped.
//! * `_` - other unicode characters can be used without escaping.
//!

use pure_rust_locales as rust_locid;
use pure_rust_locales::locale_match;
use rust_decimal::Decimal;
use std::fmt;
use std::fmt::{Debug, Display, Error as FmtError, Error, Formatter, LowerExp, Write as FmtWrite};
use std::rc::Rc;
use std::str::{from_utf8_unchecked, FromStr};

/// Symbols for number formatting.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NumberSymbols {
    /// Decimal separator
    pub decimal_sep: char,
    /// Decimal grouping
    pub decimal_grp: Option<char>,
    /// Minus sign
    pub negative_sym: char,
    /// Plus sign
    pub positive_sym: char,
    /// Exponent
    pub exponent_upper_sym: char,
    /// Exponent
    pub exponent_lower_sym: char,
    /// Currency
    pub currency_sym: CurrencySym,
    // todo: zero-digit, infinity, nan
}

impl Default for NumberSymbols {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberSymbols {
    pub const fn new() -> Self {
        Self {
            decimal_sep: '.',
            decimal_grp: Some(','),
            negative_sym: '-',
            positive_sym: ' ',
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
            currency_sym: CurrencySym::new("$"),
        }
    }

    /// Uses the locale information provided by `pure_rust_locales`.
    ///
    /// This function sets
    /// * decimal_sep to LC_NUMERIC::DECIMAL_POINT,
    /// * decimal_grp to LC_NUMERIC::THOUSANDS_SEP
    /// Fills the rest with defaults.
    pub fn numeric(locale: rust_locid::Locale) -> Self {
        Self {
            decimal_sep: first_or(locale_match!(locale => LC_NUMERIC::DECIMAL_POINT), '.'),
            decimal_grp: first_opt(locale_match!(locale => LC_NUMERIC::THOUSANDS_SEP)),
            negative_sym: '-',
            positive_sym: ' ',
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
            currency_sym: CurrencySym::new("$"),
        }
    }

    /// Uses the locale information provided by `pure_rust_locales`.
    ///
    /// This function sets
    /// * decimal_sep to LC_MONETARY::MON_DECIMAL_POINT,
    /// * decimal_grp to LC_MONETARY::MON_THOUSANDS_SEP
    /// * negative_sym to LC_MONETARY::NEGATIVE_SIGN
    /// * positive_sym to LC_MONETARY::POSITIVE_SIGN
    /// * currency_sym to LC_MONETARY::CURRENCY_SYMBOL
    /// Fills the rest with defaults.
    pub fn monetary(locale: rust_locid::Locale) -> Self {
        Self {
            decimal_sep: first_or(locale_match!(locale => LC_MONETARY::MON_DECIMAL_POINT), '.'),
            decimal_grp: first_opt(locale_match!(locale => LC_MONETARY::MON_THOUSANDS_SEP)),
            negative_sym: first_or(locale_match!(locale => LC_MONETARY::NEGATIVE_SIGN), '-'),
            positive_sym: first_or(locale_match!(locale => LC_MONETARY::POSITIVE_SIGN), ' '),
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
            currency_sym: CurrencySym::new(locale_match!(locale => LC_MONETARY::CURRENCY_SYMBOL)),
        }
    }

    /// Uses the locale information provided by `pure_rust_locales`.
    ///
    /// This function sets
    /// * decimal_sep to LC_MONETARY::MON_DECIMAL_POINT,
    /// * decimal_grp to LC_MONETARY::MON_THOUSANDS_SEP
    /// * negative_sym to LC_MONETARY::NEGATIVE_SIGN
    /// * positive_sym to LC_MONETARY::POSITIVE_SIGN
    /// * currency_sym to LC_MONETARY::INT_CURR_SYMBOL
    /// Fills the rest with defaults.
    pub fn int_monetary(locale: rust_locid::Locale) -> Self {
        Self {
            decimal_sep: first_or(locale_match!(locale => LC_MONETARY::MON_DECIMAL_POINT), '.'),
            decimal_grp: first_opt(locale_match!(locale => LC_MONETARY::MON_THOUSANDS_SEP)),
            negative_sym: first_or(locale_match!(locale => LC_MONETARY::NEGATIVE_SIGN), '-'),
            positive_sym: first_or(locale_match!(locale => LC_MONETARY::POSITIVE_SIGN), ' '),
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
            currency_sym: CurrencySym::new(locale_match!(locale => LC_MONETARY::INT_CURR_SYMBOL)),
        }
    }
}

// first char or default
#[inline]
fn first_or(s: &str, default: char) -> char {
    s.chars().next().unwrap_or(default)
}

// first char or default
#[inline]
fn first_opt(s: &str) -> Option<char> {
    s.chars().next()
}

/// Currency symbol.
///
/// This is a bit of a construction to have an inline, const-capable string.
/// All this to have a cheap, copyable default value for NumberSymbols, that can always be
/// constructed on the fly.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CurrencySym {
    len: u8,
    sym: [u8; 16],
}

impl Debug for CurrencySym {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CurrencySym")
            .field("len", &self.len)
            .field("sym", &self.sym())
            .finish()
    }
}

impl CurrencySym {
    pub const fn new(s: &str) -> Self {
        let mut sym = [0u8; 16];

        let bytes = s.as_bytes();
        let len = bytes.len();

        if len > 0 {
            sym[0] = bytes[0];
        }
        if len > 1 {
            sym[1] = bytes[1];
        }
        if len > 2 {
            sym[2] = bytes[2];
        }
        if len > 3 {
            sym[3] = bytes[3];
        }
        if len > 4 {
            sym[4] = bytes[4];
        }
        if len > 5 {
            sym[5] = bytes[5];
        }
        if len > 6 {
            sym[6] = bytes[6];
        }
        if len > 7 {
            sym[7] = bytes[7];
        }
        if len > 8 {
            sym[8] = bytes[8];
        }
        if len > 9 {
            sym[9] = bytes[9];
        }
        if len > 10 {
            sym[10] = bytes[10];
        }
        if len > 11 {
            sym[11] = bytes[11];
        }
        if len > 12 {
            sym[12] = bytes[12];
        }
        if len > 13 {
            sym[13] = bytes[13];
        }
        if len > 14 {
            sym[14] = bytes[14];
        }
        if len > 15 {
            sym[15] = bytes[15];
        }

        CurrencySym {
            len: len as u8,
            sym,
        }
    }

    pub fn first(&self) -> char {
        self.sym().chars().next().expect("currency")
    }

    pub fn sym(&self) -> &str {
        // Safety:
        // Copied from &str and never modified.
        unsafe { from_utf8_unchecked(&self.sym[..self.len as usize]) }
    }

    pub fn char_len(&self) -> usize {
        return self.sym().chars().count();
    }

    pub const fn len(&self) -> usize {
        return self.len as usize;
    }
}

impl Display for CurrencySym {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sym())
    }
}

impl<'a> From<&'a str> for CurrencySym {
    fn from(value: &'a str) -> Self {
        CurrencySym::new(value)
    }
}

/// Number mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Integer,
    Fraction,
    Exponent,
}

/// Tokens for the format.
///
/// Digit0, Digit, Numeric, NumericOpt, GroupingSep hold an digit-index.
/// Depending on mode that's the index into the integer, fraction or exponent part of
/// the number.
///
/// Numeric has an extra flag, to mark if a sign at this position is possible.
/// Next to a grouping separator there can be no sign, it will be at the position
/// of the grouping separator.
#[allow(variant_size_differences)]
#[derive(Debug, Clone)]
pub enum Token {
    /// Mask char "0". Digit or 0
    Digit0(Mode, u32),
    /// Mask char "#". Digit or sign or space
    Digit(Mode, u32, bool),
    /// Mask char "-". Integer sign.
    SignInt,
    /// Mask char ".". Decimal separator.
    DecimalSep,
    /// Mask char ":". Decimal separator, always displayed.
    DecimalSepAlways,
    /// Mask char ",". Grouping separator.
    GroupingSep(u32),
    /// Mask char "E". Exponent separator.
    ExponentUpper,
    /// Mask char "e". Exponent separator.
    ExponentLower,
    /// Mask char "-". Exponent sign.
    SignExp,
    /// Mask char "$". Currency. Each $ takes one character of the symbol.
    Currency, // todo: u32
    /// Other separator char to output literally. May be escaped with '\\'.
    Separator(char),
}

/// Holds the pattern for the numberformat and some additional data.
#[derive(Debug, Default, Clone)]
pub struct NumberFormat {
    /// Has a separate sign token for the integer part.
    has_int_sign: bool,
    /// Minimum position where a sign can be placed. Just left of a `Token::Digit0`
    min_int_sign: u32,
    /// Number of integer digits.
    len_int: u32,

    /// has a separate sign token for the exponent part.
    has_exp_sign: bool,
    /// Minimum position where a sign can be placed. Just left of a `Token::Digit0`
    min_exp_sign: u32,
    /// Decides which std-format is used. If true it's `{:e}` otherwise plain `{}`
    has_exp: bool,
    /// Number of exponent digits
    len_exp: u32,
    /// Has an exponent with a '0' pattern.
    has_exp_0: bool,

    /// Has a fraction with a '0' pattern.
    has_frac_0: bool,
    /// The required precision for this format. Is used for the underlying std-format.
    len_frac: u8,

    /// Tokens.
    tok: Vec<Token>,
    /// Symbols.
    sym: Rc<NumberSymbols>,
}

impl Display for NumberFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for t in &self.tok {
            match t {
                Token::Digit0(_, _) => write!(f, "0")?,
                Token::Digit(_, _, _) => write!(f, "#")?,
                Token::SignInt => write!(f, "-")?,
                Token::DecimalSep => write!(f, ".")?,
                Token::DecimalSepAlways => write!(f, ":")?,
                Token::GroupingSep(_) => write!(f, ",")?,
                Token::ExponentUpper => write!(f, "E")?,
                Token::ExponentLower => write!(f, "e")?,
                Token::SignExp => write!(f, "-")?,
                Token::Currency => write!(f, "$")?,
                Token::Separator(c) => {
                    match c {
                        '0' | '#' | '-' | ',' | '.' | ':' | 'E' | 'F' | 'e' | 'f' => {
                            write!(f, "{}", '\\')?;
                        }
                        _ => {}
                    }
                    write!(f, "{}", *c)?;
                }
            }
        }
        Ok(())
    }
}

impl NumberFormat {
    /// New format from pattern.
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, FmtError> {
        core::parse_number_format(pattern.as_ref())
    }

    /// New format from pattern + symbols
    pub fn news<S: AsRef<str>>(pattern: S, sym: &Rc<NumberSymbols>) -> Result<Self, FmtError> {
        core::parse_number_format(pattern.as_ref()).map(|mut v| {
            v.sym = Rc::clone(sym);
            v
        })
    }

    /// New format from token-array.
    pub fn new_tok(pattern: Vec<Token>) -> Self {
        Self::news_tok(pattern, &Rc::new(NumberSymbols::new()))
    }

    /// New format from token-array.
    pub fn news_tok(mut pattern: Vec<Token>, sym: &Rc<NumberSymbols>) -> Self {
        let mut has_exp = false;
        let mut has_exp_0 = false;
        let mut has_frac_0 = false;
        let mut has_int_sign = false;
        let mut min_int_sign = 0;
        let mut has_exp_sign = false;
        let mut min_exp_sign = 0;
        let mut len_frac = 0;
        let mut len_int = 0;
        let mut len_exp = 0;

        let mut idx_frac = 0;
        for t in pattern.iter_mut() {
            match t {
                Token::Digit0(Mode::Fraction, x) => {
                    has_frac_0 = true;
                    len_frac += 1;
                    *x = idx_frac;
                    idx_frac += 1;
                }
                Token::Digit(Mode::Fraction, x, sign) => {
                    len_frac += 1;
                    *x = idx_frac;
                    *sign = false;
                    idx_frac += 1;
                }

                Token::ExponentLower | Token::ExponentUpper => {
                    has_exp = true;
                }

                Token::SignInt => has_int_sign = true,
                Token::SignExp => has_exp_sign = true,

                _ => {}
            }
        }
        let mut idx_int = 0;
        let mut idx_exp = 0;
        let mut was_grp = false;
        for t in pattern.iter_mut().rev() {
            match t {
                Token::Digit0(Mode::Integer, x) => {
                    len_int += 1;
                    min_int_sign = idx_int + 1;
                    *x = idx_int;
                    idx_int += 1;
                }
                Token::Digit(Mode::Integer, x, sign) => {
                    len_int += 1;
                    *x = idx_int;
                    *sign = !was_grp;
                    idx_int += 1;
                }

                Token::GroupingSep(x) => {
                    *x = idx_int;
                }

                Token::Digit0(Mode::Exponent, x) => {
                    len_exp += 1;
                    has_exp_0 = true;
                    min_exp_sign = idx_exp + 1;
                    *x = idx_exp;
                    idx_exp += 1;
                }
                Token::Digit(Mode::Exponent, x, sign) => {
                    len_exp += 1;
                    *x = idx_exp;
                    *sign = true;
                    idx_exp += 1;
                }

                _ => {}
            }

            was_grp = matches!(t, Token::GroupingSep(_));
        }

        NumberFormat {
            has_int_sign,
            min_int_sign,
            len_int,
            has_exp_sign,
            min_exp_sign,
            has_exp,
            len_exp,
            has_exp_0,
            has_frac_0,
            len_frac,
            tok: pattern,
            sym: Rc::clone(sym),
        }
    }

    ///
    pub fn sym(&self) -> &NumberSymbols {
        self.sym.as_ref()
    }

    /// Tokens
    pub fn tok(&self) -> &[Token] {
        &self.tok
    }

    /// Formats or returns the error converted to a string.
    #[inline]
    pub fn fmt<Number: LowerExp + Display>(&self, number: Number) -> Result<String, fmt::Error> {
        let mut out = String::new();
        core::format_to(number, self, self.sym.as_ref(), &mut out)?;
        Ok(out)
    }

    /// Formats or returns the error converted to a string.
    #[inline]
    pub fn fmt_to<Number: LowerExp + Display, W: FmtWrite>(
        &self,
        number: Number,
        out: &mut W,
    ) -> Result<(), fmt::Error> {
        core::format_to(number, self, self.sym.as_ref(), out)
    }

    #[inline]
    pub fn parse<F: FromStr>(&self, s: &str) -> Result<F, FmtError> {
        core::parse_fmt(s, self, self.sym.as_ref())
    }
}

/// Parses a number from a &str.
pub trait ParseNumber {
    /// Parse the number after applying [core::clean_num()].
    fn parse_sym<F: FromStr>(&self, sym: &NumberSymbols) -> Result<F, FmtError>;
    /// Parse the number after applying [core::unmap_num()]
    fn parse_fmt<F: FromStr>(&self, fmt: &NumberFormat) -> Result<F, FmtError>;
}

impl ParseNumber for &str {
    fn parse_sym<F: FromStr>(&self, sym: &NumberSymbols) -> Result<F, FmtError> {
        core::parse_sym(self, sym)
    }

    fn parse_fmt<F: FromStr>(&self, fmt: &NumberFormat) -> Result<F, FmtError> {
        core::parse_fmt(self, fmt, &fmt.sym)
    }
}

/// Format a number according to a format string.
pub trait FormatNumber
where
    Self: Copy + LowerExp + Display,
{
    /// Format using the format-string. Uses the given symbols.
    fn format<'a>(
        &self,
        pattern: &'a str,
        sym: &'a NumberSymbols,
    ) -> Result<FormattedNumber<'a, Self>, FmtError>;

    /// Format using the [NumberFormat]
    fn fmt<'a>(&self, format: &'a NumberFormat) -> RefFormattedNumber<'a, Self>;
}

#[derive(Debug)]
pub struct FormattedNumber<'a, Number> {
    num: Number,
    format: NumberFormat,
    sym: &'a NumberSymbols,
}

impl<'a, Number: Copy + LowerExp + Display> Display for FormattedNumber<'a, Number> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        core::format_to(self.num, &self.format, &self.sym, f)
    }
}

#[derive(Debug)]
pub struct RefFormattedNumber<'a, Number> {
    num: Number,
    format: &'a NumberFormat,
}

impl<'a, Number: Copy + LowerExp + Display> Display for RefFormattedNumber<'a, Number> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        core::format_to(self.num, self.format, self.format.sym.as_ref(), f)
    }
}

macro_rules! define_fmt {
    ($t:ty) => {
        impl FormatNumber for $t {
            #[inline]
            fn format<'a>(
                &self,
                pattern: &'a str,
                sym: &'a NumberSymbols,
            ) -> Result<FormattedNumber<'a, Self>, FmtError> {
                Ok(FormattedNumber {
                    num: *self,
                    format: core::parse_number_format(pattern)?,
                    sym,
                })
            }

            #[inline]
            fn fmt<'a>(&self, format: &'a NumberFormat) -> RefFormattedNumber<'a, Self> {
                RefFormattedNumber { num: *self, format }
            }
        }
    };
}

define_fmt!(f64);
define_fmt!(f32);
define_fmt!(u128);
define_fmt!(u64);
define_fmt!(u32);
define_fmt!(u16);
define_fmt!(u8);
define_fmt!(i128);
define_fmt!(i64);
define_fmt!(i32);
define_fmt!(i16);
define_fmt!(i8);
define_fmt!(usize);
define_fmt!(isize);
define_fmt!(Decimal);

pub mod core {
    use crate::number::{Mode, NumberFormat, NumberSymbols, Token};
    #[allow(unused_imports)]
    use log::debug;
    use memchr::memchr;
    use std::cell::Cell;
    use std::cmp::max;
    use std::fmt::{Display, Error as FmtError, LowerExp, Write as FmtWrite};
    use std::str::FromStr;

    /// Parses the format string. Uses the default symbol table.
    pub fn parse_number_format(pattern: &str) -> Result<NumberFormat, FmtError> {
        let mut esc = false;
        let mut mode = Mode::Integer;

        let mut tok = Vec::new();

        for m in pattern.chars() {
            let mask = if esc {
                esc = false;
                Token::Separator(m)
            } else {
                match m {
                    '0' => Token::Digit0(mode, 0),
                    '#' => Token::Digit(mode, 0, false),
                    '.' => {
                        if matches!(mode, Mode::Fraction | Mode::Exponent) {
                            return Err(FmtError);
                        }
                        mode = Mode::Fraction;
                        Token::DecimalSep
                    }
                    ':' => {
                        if matches!(mode, Mode::Fraction | Mode::Exponent) {
                            return Err(FmtError);
                        }
                        mode = Mode::Fraction;
                        Token::DecimalSepAlways
                    }
                    ',' => Token::GroupingSep(0),
                    '-' => {
                        if mode == Mode::Integer {
                            Token::SignInt
                        } else if mode == Mode::Exponent {
                            Token::SignExp
                        } else {
                            return Err(FmtError);
                        }
                    }
                    'e' => {
                        if mode == Mode::Exponent {
                            return Err(FmtError);
                        }
                        mode = Mode::Exponent;
                        Token::ExponentLower
                    }
                    'E' => {
                        if mode == Mode::Exponent {
                            return Err(FmtError);
                        }
                        mode = Mode::Exponent;
                        Token::ExponentUpper
                    }
                    '$' => Token::Currency,
                    '\\' => {
                        esc = true;
                        continue;
                    }
                    ' ' => Token::Separator(' '),
                    c if c.is_ascii() => return Err(FmtError),
                    c => Token::Separator(c),
                }
            };
            tok.push(mask);
        }

        Ok(NumberFormat::new_tok(tok))
    }

    fn split_num(value: &str) -> (&str, &str, &str, &str, &str) {
        // everything is ascii
        let bytes = value.as_bytes();
        let len = bytes.len();

        let idx_sep = memchr(b'.', bytes);
        let idx_exp = memchr(b'e', bytes);

        let digits_end = if let Some(idx_sep) = idx_sep {
            idx_sep
        } else if let Some(idx_exp) = idx_exp {
            idx_exp
        } else {
            len
        };

        let fraction_end = if let Some(idx_exp) = idx_exp {
            idx_exp
        } else {
            len
        };

        let (r_sign, r_digits) = if len > 0 && bytes[0] == b'-' {
            (0usize..1usize, 1usize..digits_end)
        } else {
            (0usize..0usize, 0usize..digits_end)
        };
        let r_fraction = if let Some(idx_sep) = idx_sep {
            idx_sep + 1..fraction_end
        } else {
            fraction_end..fraction_end
        };
        let (r_sign_exp, r_exp) = if let Some(idx_exp) = idx_exp {
            if idx_exp + 1 < len && bytes[idx_exp + 1] == b'-' {
                (idx_exp + 1..idx_exp + 2, idx_exp + 2..len)
            } else {
                (idx_exp + 1..idx_exp + 1, idx_exp + 1..len)
            }
        } else {
            (len..len, len..len)
        };

        (
            &value[r_sign],
            &value[r_digits],
            &value[r_fraction],
            &value[r_sign_exp],
            &value[r_exp],
        )
    }

    /// Get the clean number.
    ///
    /// Takes only digits and maps backwards according to the symbol table.
    /// This will only work if you don't use separators that can be mistaken
    /// with one of those symbols.
    ///
    /// Removes any leading zeros too.
    pub fn clean_num<W: FmtWrite>(
        formatted: &str,
        sym: &NumberSymbols,
        out: &mut W,
    ) -> Result<(), FmtError> {
        let mut seen_non_0 = false;
        for c in formatted.chars() {
            if c.is_ascii_digit() {
                seen_non_0 |= c != '0';
                if seen_non_0 {
                    out.write_char(c)?;
                }
            } else if c == sym.negative_sym {
                out.write_char('-')?;
            } else if c == sym.positive_sym || c == '+' {
                // noop
            } else if c == sym.decimal_sep {
                out.write_char('.')?;
            } else if c == sym.exponent_lower_sym {
                out.write_char('e')?;
            } else if c == sym.exponent_upper_sym {
                out.write_char('e')?;
            }
        }
        Ok(())
    }

    /// Unmap the formatted string back to a format that `f64::parse()` can understand.
    ///
    /// Token::NumericOpt is not supported for now.
    pub fn unmap_num<W: FmtWrite>(
        formatted: &str,
        format: &NumberFormat,
        sym: &NumberSymbols,
        out: &mut W,
    ) -> Result<(), FmtError> {
        let mut it = format.tok.iter();
        let mut jt = formatted.chars();
        loop {
            let Some(t) = it.next() else {
                break;
            };
            let Some(c) = jt.next() else {
                break;
            };

            match t {
                Token::Digit0(_, _) => {
                    if c.is_ascii_digit() {
                        out.write_char(c)?;
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::Digit(_, _, _) => {
                    if c.is_ascii_digit() {
                        out.write_char(c)?;
                    } else if c == sym.negative_sym {
                        out.write_char('-')?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::SignInt => {
                    if c == sym.negative_sym {
                        out.write_char('-')?;
                    } else if c == sym.positive_sym {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::SignExp => {
                    if c == sym.negative_sym {
                        out.write_char('-')?;
                    } else if c == sym.positive_sym {
                        // ok
                    } else if c == '+' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::DecimalSep => {
                    if c == sym.decimal_sep {
                        out.write_char('.')?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::DecimalSepAlways => {
                    if c == sym.decimal_sep {
                        out.write_char('.')?;
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::GroupingSep(_) => {
                    if let Some(decimal_grp) = sym.decimal_grp {
                        if c == decimal_grp {
                            // ok
                        } else if c == ' ' {
                            // ok
                        } else {
                            return Err(FmtError);
                        }
                    }
                }
                Token::ExponentUpper => {
                    if c == sym.exponent_upper_sym {
                        out.write_char('e')?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::ExponentLower => {
                    if c == sym.exponent_lower_sym {
                        out.write_char('e')?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::Currency => {
                    if c == sym.currency_sym.first() {
                        for _ in 1..sym.currency_sym.char_len() {
                            jt.next();
                        }
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::Separator(sep) => {
                    if c == *sep {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
            }
        }

        Ok(())
    }

    /// Takes a raw number string and applies the format.
    ///
    /// The raw number should be in a format produced by the format! macro. decimal point is '.',
    /// exponent is 'e' and negative sign is '-'.
    #[inline]
    pub fn map_num<W: FmtWrite>(
        raw: &str,
        format: &NumberFormat,
        sym: &NumberSymbols,
        out: &mut W,
    ) -> Result<(), FmtError> {
        let (sign, int, frac, exp_sign, exp) = split_num(raw);

        let int = int.as_bytes();
        let len_int = int.len() as u32;
        let frac = frac.as_bytes();
        let len_frac = frac.len() as u32;
        let exp = exp.as_bytes();
        let len_exp = exp.len() as u32;

        // let len_sign = sign.len() as u32;
        let len_exp_sign = exp_sign.len() as u32;

        let mut used_sign = sign.is_empty();
        let mut used_exp_sign = exp_sign.is_empty();

        // missing fractions are ok. other missing digits not.
        if len_int > format.len_int {
            return Err(FmtError);
        }
        if len_exp > format.len_exp {
            return Err(FmtError);
        }
        // not enough space for the exponent
        if max(len_exp, format.min_exp_sign) + len_exp_sign > format.len_exp {
            return Err(FmtError);
        }
        // left shift the exponent and fill the rest with ' '.
        let shift_exp_n = format.len_exp - max(len_exp, format.min_exp_sign) - len_exp_sign;
        let shift_exp_pos = max(len_exp, format.min_exp_sign) + len_exp_sign;

        for m in format.tok.iter() {
            match m {
                Token::SignInt => {
                    if !used_sign {
                        out.write_char(sym.positive_sym)?;
                    } else {
                        out.write_char(sym.negative_sym)?;
                    }
                    used_sign = true;
                }
                Token::GroupingSep(i) => {
                    let Some(decimal_grp) = sym.decimal_grp else {
                        continue;
                    };
                    if len_int > *i {
                        out.write_char(decimal_grp)?;
                    } else if max(len_int, format.min_int_sign) == *i
                        && !used_sign
                        && !format.has_int_sign
                    {
                        out.write_str(sign)?;
                        used_sign = true;
                    } else {
                        out.write_char(' ')?;
                    }
                }
                Token::Digit0(Mode::Integer, i) => {
                    if len_int > *i {
                        out.write_char(int[(len_int - i - 1) as usize] as char)?;
                    } else {
                        out.write_char('0')?;
                    }
                }
                Token::Digit(Mode::Integer, i, can_be_sign) => {
                    if len_int > *i {
                        out.write_char(int[(len_int - i - 1) as usize] as char)?;
                    } else if max(len_int, format.min_int_sign) == *i
                        && *can_be_sign
                        && !used_sign
                        && !format.has_int_sign
                    {
                        out.write_str(sign)?;
                        used_sign = true;
                    } else {
                        out.write_char(' ')?;
                    }
                }
                Token::DecimalSep => {
                    if !frac.is_empty() || format.has_frac_0 {
                        out.write_char(sym.decimal_sep)?;
                    } else {
                        out.write_char(' ')?;
                    }
                }
                Token::DecimalSepAlways => {
                    out.write_char(sym.decimal_sep)?;
                }
                Token::Digit0(Mode::Fraction, i) => {
                    if len_frac > *i {
                        out.write_char(frac[*i as usize] as char)?;
                    } else {
                        out.write_char('0')?;
                    }
                }
                Token::Digit(Mode::Fraction, i, _) => {
                    if len_frac > *i {
                        out.write_char(frac[*i as usize] as char)?;
                    } else {
                        out.write_char(' ')?;
                    }
                }
                Token::ExponentUpper => {
                    if !exp.is_empty() || format.has_exp_0 {
                        out.write_char(sym.exponent_upper_sym)?;
                    } else {
                        out.write_char(' ')?;
                    }
                }
                Token::ExponentLower => {
                    if !exp.is_empty() || format.has_exp_0 {
                        out.write_char(sym.exponent_lower_sym)?;
                    } else {
                        out.write_char(' ')?;
                    }
                }
                Token::SignExp => {
                    if exp_sign.is_empty() && sym.positive_sym == ' ' {
                        // explicit sign in the exponent shows '+'.
                        out.write_char('+')?;
                    } else if exp_sign.is_empty() {
                        out.write_char(sym.positive_sym)?;
                    } else {
                        out.write_char(sym.negative_sym)?;
                    }
                    used_exp_sign = true;
                }
                Token::Digit0(Mode::Exponent, i) => {
                    if *i >= shift_exp_pos {
                        // left-shift exponent
                    } else if len_exp > *i {
                        out.write_char(exp[(len_exp - i - 1) as usize] as char)?;
                    } else {
                        out.write_char('0')?;
                    }

                    // append shifted digits as blank
                    if *i == 0 {
                        for _ in 0..shift_exp_n {
                            out.write_char(' ')?;
                        }
                    }
                }
                Token::Digit(Mode::Exponent, i, _) => {
                    if *i >= shift_exp_pos {
                        // left-shift exponent
                    } else if len_exp > *i {
                        out.write_char(exp[(len_exp - i - 1) as usize] as char)?;
                    } else if max(len_exp, format.min_exp_sign) == *i
                        && !used_exp_sign
                        && !format.has_exp_sign
                    {
                        out.write_str(exp_sign)?;
                        used_exp_sign = true;
                    } else {
                        out.write_char(' ')?;
                    }

                    // append shifted digits as blank
                    if *i == 0 {
                        for _ in 0..shift_exp_n {
                            out.write_char(' ')?;
                        }
                    }
                }
                Token::Currency => {
                    out.write_str(sym.currency_sym.sym())?;
                }
                Token::Separator(v) => {
                    out.write_char(*v)?;
                }
            }
        }

        if !used_sign {
            return Err(FmtError);
        }
        if !used_exp_sign {
            return Err(FmtError);
        }

        Ok(())
    }

    /// Formats the number and writes the result to out.
    pub fn format_to<W: FmtWrite, Number: LowerExp + Display>(
        number: Number,
        format: &NumberFormat,
        sym: &NumberSymbols,
        out: &mut W,
    ) -> Result<(), FmtError> {
        thread_local! {
            static RAW: Cell<String> = Cell::new(String::new());
        }

        let mut raw = RAW.take();

        raw.clear();
        if format.has_exp {
            write!(raw, "{:.*e}", format.len_frac as usize, number).map_err(|_| FmtError)?;
        } else {
            write!(raw, "{:.*}", format.len_frac as usize, number).map_err(|_| FmtError)?;
        };

        match map_num(raw.as_str(), format, sym, out) {
            Ok(v) => {
                RAW.set(raw);
                Ok(v)
            }
            Err(e) => {
                RAW.set(raw);
                Err(e)
            }
        }
    }

    pub fn parse_fmt<F: FromStr>(
        s: &str,
        fmt: &NumberFormat,
        sym: &NumberSymbols,
    ) -> Result<F, FmtError> {
        thread_local! {
            static RAW: Cell<String> = Cell::new(String::new());
        }

        let mut raw = RAW.take();

        raw.clear();
        unmap_num(s, fmt, sym, &mut raw)?;

        match raw.parse::<F>() {
            Ok(v) => {
                RAW.set(raw);
                Ok(v)
            }
            Err(_) => {
                RAW.set(raw);
                Err(FmtError)
            }
        }
    }

    pub fn parse_sym<F: FromStr>(s: &str, sym: &NumberSymbols) -> Result<F, FmtError> {
        thread_local! {
            static RAW: Cell<String> = Cell::new(String::new());
        }

        let mut raw = RAW.take();

        raw.clear();
        clean_num(s, sym, &mut raw)?;

        match raw.parse::<F>() {
            Ok(v) => {
                RAW.set(raw);
                Ok(v)
            }
            Err(_) => {
                RAW.set(raw);
                Err(FmtError)
            }
        }
    }
}

/// Format a Number according to the format string.
/// Uses the default symbols.
pub fn format<Number: LowerExp + Display>(
    number: Number,
    pattern: &str,
) -> Result<String, FmtError> {
    let format = core::parse_number_format(pattern)?;
    let mut out = String::new();
    core::format_to(number, &format, &NumberSymbols::new(), &mut out)?;
    Ok(out)
}

/// Format a Number according to the format string.
/// Uses the default symbols.
pub fn format_to<W: FmtWrite, Number: LowerExp + Display>(
    number: Number,
    pattern: &str,
    out: &mut W,
) -> Result<(), FmtError> {
    let format = core::parse_number_format(pattern)?;
    core::format_to(number, &format, &NumberSymbols::new(), out)
}

/// Format a Number according to the format string.
pub fn formats<Number: LowerExp + Display>(
    number: Number,
    pattern: &str,
    sym: &NumberSymbols,
) -> Result<String, FmtError> {
    let format = core::parse_number_format(pattern)?;
    let mut out = String::new();
    core::format_to(number, &format, sym, &mut out)?;
    Ok(out)
}

/// Format a Number according to the format string.
pub fn formats_to<W: FmtWrite, Number: LowerExp + Display>(
    number: Number,
    pattern: &str,
    sym: &NumberSymbols,
    out: &mut W,
) -> Result<(), FmtError> {
    let format = core::parse_number_format(pattern)?;
    core::format_to(number, &format, &sym, out)
}

/// Format a Number according to the format.
pub fn fmt<Number: LowerExp + Display>(number: Number, format: &NumberFormat) -> String {
    let mut out = String::new();
    _ = core::format_to(number, format, format.sym.as_ref(), &mut out);
    out
}

/// Format a Number according to the format.
pub fn fmt_to<W: FmtWrite, Number: LowerExp + Display>(
    number: Number,
    format: &NumberFormat,
    out: &mut W,
) {
    _ = core::format_to(number, format, format.sym.as_ref(), out)
}

/// Parse using the NumberSymbols.
/// Parses the number after applying [core::clean_num]
pub fn parse_sym<F: FromStr>(s: &str, sym: &NumberSymbols) -> Result<F, FmtError> {
    core::parse_sym(s, sym)
}

/// Parse using the NumberFormat.
/// Parses the number after applying [core::unmap_num]
pub fn parse_fmt<F: FromStr>(s: &str, fmt: &NumberFormat) -> Result<F, FmtError> {
    core::parse_fmt(s, fmt, &fmt.sym)
}

/// Parse using the NumberFormat.
/// Parses the number after applying [core::unmap_num]
pub fn parse_format<F: FromStr>(
    s: &str,
    pattern: &str,
    sym: &NumberSymbols,
) -> Result<F, FmtError> {
    let format = core::parse_number_format(pattern)?;
    core::parse_fmt(s, &format, sym)
}
