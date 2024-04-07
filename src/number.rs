//! Number formatting.
//!
//! This one uses a pattern string instead of the `format!` style.
//!
//! ```
//! use std::rc::Rc;
//! use rat_salsa::number::{fmt_f64};
//!
//! // formats accordingly, uses the default symbols.
//! let s = fmt_f64(4561.2234, "###,##0.00", None)?;
//!
//! assert_eq!(s, "  4,561.22");
//! ```
//!
//! The following patterns are recognized:
//! * `0` - digit or 0
//! * `9` - digit or space
//! * `#` - digit or sign or space
//! * `8` - digit or sign or _empty string_
//! * `+` - sign; show + for positive
//! * `-` - sign; show space for positive
//! * `.` - decimal separator
//! * `:` - decimal separator, always shown
//! * `,` - grouping separator
//! * `;` - grouping separator, always shown
//! * `E` - upper case exponent
//! * `F` - upper case exponent, always shown
//! * `e` - lower case exponent
//! * `f` - lower case exponent, always shown
//! * ` ` - space can be used as separator
//! * '$' - currency, variable width dependent on the defined symbol.
//! * `\` - all ascii characters (ascii 32-128!) are reserved and must be escaped.
//! * `_` - other unicode characters can be used without escaping.
//!

use rust_decimal::Decimal;
use std::fmt;
use std::fmt::{Debug, Display, Error as FmtError, Formatter, LowerExp, Write as FmtWrite};
use std::rc::Rc;
use std::str::{from_utf8_unchecked, FromStr};

/// Symbols for number formatting.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NumberSymbols {
    /// Decimal separator
    pub decimal_sep: char,
    /// Decimal grouping
    pub decimal_grp: char,
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
            decimal_grp: ',',
            negative_sym: '-',
            positive_sym: '+',
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
            currency_sym: CurrencySym::new("$"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CurrencySym {
    len: usize,
    sym: [u8; 16],
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

        CurrencySym { len, sym }
    }

    pub fn first(&self) -> char {
        self.sym().chars().next().expect("currency")
    }

    pub fn sym(&self) -> &str {
        // Safety:
        // Copied from &str and never modified.
        unsafe { from_utf8_unchecked(&self.sym[..self.len]) }
    }

    pub fn char_len(&self) -> usize {
        return self.sym().chars().count();
    }

    pub const fn len(&self) -> usize {
        return self.len;
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
#[allow(variant_size_differences)]
#[derive(Debug, Clone)]
pub enum Token {
    /// Mask char "0". Digit or 0
    Digit0(Mode),
    /// Mask char "9". Digit or space
    Digit(Mode),
    /// Mask char "#". Digit or sign or space
    Numeric(Mode),
    /// Mask char "8". Digit or sign or *empty string*.
    NumericOpt(Mode),
    /// Mask char "+". Show "+" sign for positive number, "-" for negative.
    Plus(Mode),
    /// Mask char "-". Show " " for positive number, "-" for negative.
    Minus(Mode),
    /// Mask char ".". Decimal separator.
    DecimalSep,
    /// Mask char ":". Decimal separator, always displayed.
    DecimalSepAlways,
    /// Mask char ",". Grouping separator.
    GroupingSep,
    /// Mask char ";". Grouping separator.
    GroupingSepAlways,
    /// Mask char "E". Exponent separator.
    ExponentUpper,
    /// Mask char "F". Exponent separator.
    ExponentUpperAlways,
    /// Mask char "e". Exponent separator.
    ExponentLower,
    /// Mask char "f". Exponent separator.
    ExponentLowerAlways,
    /// Mask char "$". Currency.
    Currency,
    /// Other separator char to output literally. May be escaped with '\\'.
    Separator(char),
}

/// Holds the pattern for the numberformat and some additional data.
#[derive(Default, Clone)]
pub struct NumberFormat {
    /// Decides what std-format is used. If true it's `{:e}` otherwise plain `{}`
    pub has_exp: bool,
    /// Has an exponent with a '0' pattern.
    pub has_exp_0: bool,
    /// Has a fraction with a '0' pattern.
    pub has_frac_0: bool,
    /// The required precision for this format. Is used for the underlying std-format.
    pub precision: u8,
    /// Tokens.
    pub tok: Vec<Token>,
    /// Symbols.
    pub sym: Rc<NumberSymbols>,
}

impl Display for NumberFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for t in &self.tok {
            match t {
                Token::Digit0(_) => write!(f, "0")?,
                Token::Digit(_) => write!(f, "9")?,
                Token::Numeric(_) => write!(f, "#")?,
                Token::NumericOpt(_) => write!(f, "8")?,
                Token::Plus(_) => write!(f, "+")?,
                Token::Minus(_) => write!(f, "-")?,
                Token::DecimalSep => write!(f, ".")?,
                Token::DecimalSepAlways => write!(f, ":")?,
                Token::GroupingSep => write!(f, ",")?,
                Token::GroupingSepAlways => write!(f, ";")?,
                Token::ExponentUpper => write!(f, "E")?,
                Token::ExponentUpperAlways => write!(f, "F")?,
                Token::ExponentLower => write!(f, "e")?,
                Token::ExponentLowerAlways => write!(f, "f")?,
                Token::Currency => write!(f, "$")?,
                Token::Separator(c) => {
                    match c {
                        '0' | '9' | '#' | '8' | '+' | '-' | ',' | ';' | '.' | ':' | 'E' | 'F'
                        | 'e' | 'f' => {
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
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self, FmtError> {
        core::parse_number_format(pattern.as_ref())
    }

    pub fn news<S: AsRef<str>>(pattern: S, sym: &Rc<NumberSymbols>) -> Result<Self, FmtError> {
        core::parse_number_format_sym(pattern.as_ref(), sym)
    }

    pub fn with_sym<S: AsRef<str>>(pattern: S, sym: &Rc<NumberSymbols>) -> Result<Self, FmtError> {
        core::parse_number_format_sym(pattern.as_ref(), sym)
    }

    /// Set the decimal symbols.
    #[inline]
    pub fn sym(mut self, sym: &Rc<NumberSymbols>) -> Self {
        self.sym = Rc::clone(sym);
        self
    }

    /// Formats or returns the error converted to a string.
    #[inline]
    pub fn fmt<Number: LowerExp + Display>(&self, number: Number) -> String {
        let mut out = String::new();
        _ = core::format_to(number, self, self.sym.as_ref(), &mut out);
        out
    }

    /// Formats or returns the error converted to a string.
    #[inline]
    pub fn fmt_to<Number: LowerExp + Display, W: FmtWrite>(&self, number: Number, out: &mut W) {
        _ = core::format_to(number, self, self.sym.as_ref(), out);
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

impl Debug for NumberFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = format!("{}", self);
        f.debug_struct("NumberFormat")
            .field("has_exp", &self.has_exp)
            .field("precision", &self.precision)
            .field("sym", &self.sym)
            .field("tok", &s)
            .finish()
    }
}

pub mod core {
    use crate::number::{Mode, NumberFormat, NumberSymbols, Token};
    use std::cell::Cell;
    use std::fmt::{Display, Error as FmtError, LowerExp, Write as FmtWrite};
    use std::iter;
    use std::rc::Rc;
    use std::str::FromStr;

    /// Parses the format string and sets a symbol table.
    #[inline]
    pub fn parse_number_format_sym(
        pattern: &str,
        sym: &Rc<NumberSymbols>,
    ) -> Result<NumberFormat, FmtError> {
        parse_number_format(pattern).map(|mut v| {
            v.sym = Rc::clone(sym);
            v
        })
    }

    /// Parses the format string for reuse.
    pub fn parse_number_format(pattern: &str) -> Result<NumberFormat, FmtError> {
        let mut format = NumberFormat::default();

        let mut esc = false;
        let mut mode = Mode::Integer;

        for m in pattern.chars() {
            let mask = if esc {
                esc = false;
                Token::Separator(m)
            } else {
                match m {
                    '0' => {
                        if mode == Mode::Fraction {
                            format.has_frac_0 = true;
                            format.precision += 1;
                        } else if mode == Mode::Exponent {
                            format.has_exp_0 = true;
                        }
                        Token::Digit0(mode)
                    }
                    '8' => {
                        if mode == Mode::Fraction {
                            format.precision += 1;
                        }
                        Token::NumericOpt(mode)
                    }
                    '9' => {
                        if mode == Mode::Fraction {
                            format.precision += 1;
                        }
                        Token::Digit(mode)
                    }
                    '#' => {
                        if mode == Mode::Fraction {
                            format.precision += 1;
                        }
                        Token::Numeric(mode)
                    }
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
                    ',' => Token::GroupingSep,
                    ';' => Token::GroupingSepAlways,
                    '+' => Token::Plus(mode),
                    '-' => Token::Minus(mode),
                    'e' => {
                        if mode == Mode::Exponent {
                            return Err(FmtError);
                        }
                        format.has_exp = true;
                        mode = Mode::Exponent;
                        Token::ExponentLower
                    }
                    'E' => {
                        if mode == Mode::Exponent {
                            return Err(FmtError);
                        }
                        format.has_exp = true;
                        mode = Mode::Exponent;
                        Token::ExponentUpper
                    }
                    'f' => {
                        if mode == Mode::Exponent {
                            return Err(FmtError);
                        }
                        format.has_exp = true;
                        mode = Mode::Exponent;
                        Token::ExponentLower
                    }
                    'F' => {
                        if mode == Mode::Exponent {
                            return Err(FmtError);
                        }
                        format.has_exp = true;
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
            format.tok.push(mask);
        }

        Ok(format)
    }

    // Splits into sign, integer-part, fraction-part, exponent-sign, exponent
    fn split_num(value: &str) -> (&str, &str, &str, &str, &str) {
        let mut byte_sign = None;
        let mut byte_digits = None;
        let mut byte_sep = None;
        let mut byte_exp = None;
        let mut byte_sign_exp = None;

        for (idx, c) in value.char_indices() {
            if c == '-' || c == '+' {
                if byte_exp.is_none() {
                    byte_sign = Some(idx);
                } else {
                    byte_sign_exp = Some(idx);
                }
            }
            if byte_sep.is_none()
                && byte_exp.is_none()
                && byte_digits.is_none()
                && c.is_ascii_digit()
                && c != '0'
            {
                // first non-zero integer digit
                byte_digits = Some(idx);
            }
            if c == '.' {
                byte_sep = Some(idx);
            }
            if c == 'e' || c == 'E' {
                byte_exp = Some(idx);
            }
        }

        let r_sign = if let Some(byte_sign) = byte_sign {
            byte_sign..byte_sign + 1
        } else {
            0..0
        };

        let r_digits = if let Some(byte_digits) = byte_digits {
            if let Some(byte_sep) = byte_sep {
                byte_digits..byte_sep
            } else if let Some(byte_exp) = byte_exp {
                byte_digits..byte_exp
            } else {
                byte_digits..value.len()
            }
        } else if let Some(byte_sign) = byte_sign {
            if let Some(byte_sep) = byte_sep {
                byte_sign + 1..byte_sep
            } else if let Some(byte_exp) = byte_exp {
                byte_sign + 1..byte_exp
            } else {
                byte_sign + 1..value.len()
            }
        } else {
            if let Some(byte_sep) = byte_sep {
                0..byte_sep
            } else if let Some(byte_exp) = byte_exp {
                0..byte_exp
            } else {
                0..value.len()
            }
        };

        let r_fraction = if let Some(byte_sep) = byte_sep {
            if let Some(byte_exp) = byte_exp {
                byte_sep + 1..byte_exp
            } else {
                byte_sep + 1..value.len()
            }
        } else {
            r_digits.end..r_digits.end
        };

        let r_sign_exp = if let Some(byte_sign_exp) = byte_sign_exp {
            byte_sign_exp..byte_sign_exp + 1
        } else if let Some(byte_exp) = byte_exp {
            byte_exp + 1..byte_exp + 1
        } else {
            value.len()..value.len()
        };

        let r_exp = if let Some(byte_sign_exp) = byte_sign_exp {
            byte_sign_exp + 1..value.len()
        } else if let Some(byte_exp) = byte_exp {
            byte_exp + 1..value.len()
        } else {
            value.len()..value.len()
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
                    write!(out, "{}", c)?;
                }
            } else if c == sym.negative_sym {
                write!(out, "-")?;
            } else if c == sym.decimal_sep {
                write!(out, ".")?;
            } else if c == sym.exponent_lower_sym {
                write!(out, "e")?;
            } else if c == sym.exponent_upper_sym {
                write!(out, "E")?;
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
                Token::Digit0(_) => {
                    if c.is_ascii_digit() {
                        write!(out, "{}", c)?;
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::Digit(_) => {
                    if c.is_ascii_digit() {
                        write!(out, "{}", c)?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::Numeric(_) => {
                    if c.is_ascii_digit() {
                        write!(out, "{}", c)?;
                    } else if c == sym.negative_sym {
                        write!(out, "-")?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::NumericOpt(_) => {
                    unimplemented!("NumericOpt not supported.");
                }
                Token::Plus(_) => {
                    if c == sym.negative_sym {
                        write!(out, "-")?;
                    } else if c == sym.positive_sym {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::Minus(_) => {
                    if c == sym.negative_sym {
                        write!(out, "-")?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::DecimalSep => {
                    if c == sym.decimal_sep {
                        write!(out, ".")?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::DecimalSepAlways => {
                    if c == sym.decimal_sep {
                        write!(out, ".")?;
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::GroupingSep => {
                    if c == sym.decimal_grp {
                        // ok
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::GroupingSepAlways => {
                    if c == sym.decimal_grp {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::ExponentUpper => {
                    if c == sym.exponent_upper_sym {
                        write!(out, "E")?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::ExponentUpperAlways => {
                    if c == sym.exponent_upper_sym {
                        write!(out, "E")?;
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::ExponentLower => {
                    if c == sym.exponent_lower_sym {
                        write!(out, "e")?;
                    } else if c == ' ' {
                        // ok
                    } else {
                        return Err(FmtError);
                    }
                }
                Token::ExponentLowerAlways => {
                    if c == sym.exponent_lower_sym {
                        write!(out, "e")?;
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
    /// The raw number should be in a format produced by the format! macro. decimal point is '.'
    /// and exponent is 'e' or 'E'.
    #[inline]
    pub fn map_num<W: FmtWrite>(
        raw: &str,
        format: &NumberFormat,
        sym: &NumberSymbols,
        out: &mut W,
    ) -> Result<(), FmtError> {
        thread_local! {
            static BUF: Cell<Vec<char>> = const { Cell::new(Vec::new()) };
        }

        let mut buf = BUF.take();

        buf.clear();
        buf.extend(iter::repeat(' ').take(format.tok.len()));

        _map_num(raw, format, sym, &mut buf)?;

        for i in 0..format.tok.len() {
            if buf[i] == '\u{00}' {
                // noop
            } else if buf[i] == '\u{11}' {
                match write!(out, "{}", sym.currency_sym) {
                    Err(e) => {
                        BUF.set(buf);
                        return Err(e);
                    }
                    _ => {}
                }
            } else {
                match write!(out, "{}", buf[i]) {
                    Err(e) => {
                        BUF.set(buf);
                        return Err(e);
                    }
                    _ => {}
                };
            }
        }
        BUF.set(buf);
        Ok(())
    }

    // impl without type parameters
    fn _map_num(
        raw: &str,
        format: &NumberFormat,
        sym: &NumberSymbols,
        buffer: &mut [char],
    ) -> Result<(), FmtError> {
        let (mut sign, integer, fraction, mut exp_sign, exp) = split_num(raw);
        let mut it_integer = integer.chars();
        let mut it_fraction = fraction.chars();
        let mut it_exp = exp.chars();

        for (i, m) in format.tok.iter().enumerate() {
            match m {
                Token::Plus(Mode::Integer) => {
                    if sign.is_empty() {
                        buffer[i] = sym.positive_sym;
                    } else {
                        buffer[i] = sym.negative_sym;
                    }
                    sign = "";
                }
                Token::Minus(Mode::Integer) => {
                    if sign.is_empty() {
                        buffer[i] = ' ';
                    } else {
                        buffer[i] = sym.negative_sym;
                    }
                    sign = "";
                }
                Token::GroupingSep => {}
                Token::GroupingSepAlways => {}
                Token::Digit0(Mode::Integer) => {}
                Token::Digit(Mode::Integer) => {}
                Token::Numeric(Mode::Integer) => {}
                Token::NumericOpt(Mode::Integer) => {}

                Token::DecimalSep => {
                    if !fraction.is_empty() || format.has_frac_0 {
                        buffer[i] = sym.decimal_sep;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::DecimalSepAlways => {
                    buffer[i] = sym.decimal_sep;
                }
                Token::Plus(Mode::Fraction) => {}
                Token::Minus(Mode::Fraction) => {}
                Token::Digit0(Mode::Fraction) => {
                    if let Some(d) = it_fraction.next() {
                        buffer[i] = d;
                    } else {
                        buffer[i] = '0';
                    }
                }
                Token::Digit(Mode::Fraction) => {
                    if let Some(d) = it_fraction.next() {
                        buffer[i] = d;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::Numeric(Mode::Fraction) => {
                    if let Some(d) = it_fraction.next() {
                        buffer[i] = d;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::NumericOpt(Mode::Fraction) => {
                    if let Some(d) = it_fraction.next() {
                        buffer[i] = d;
                    } else {
                        buffer[i] = '\u{00}';
                    }
                }

                Token::ExponentUpper => {
                    if !exp.is_empty() || format.has_exp_0 {
                        buffer[i] = sym.exponent_upper_sym;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::ExponentLower => {
                    if !exp.is_empty() || format.has_exp_0 {
                        buffer[i] = sym.exponent_lower_sym;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::ExponentUpperAlways => {
                    buffer[i] = sym.exponent_upper_sym;
                }
                Token::ExponentLowerAlways => {
                    buffer[i] = sym.exponent_lower_sym;
                }

                Token::Plus(Mode::Exponent) => {
                    if exp_sign.is_empty() {
                        buffer[i] = sym.positive_sym;
                    } else {
                        buffer[i] = sym.negative_sym;
                    }
                    exp_sign = "";
                }
                Token::Minus(Mode::Exponent) => {
                    if exp_sign.is_empty() {
                        buffer[i] = ' ';
                    } else {
                        buffer[i] = sym.negative_sym;
                    }
                    exp_sign = "";
                }
                Token::Digit0(Mode::Exponent) => {}
                Token::Digit(Mode::Exponent) => {}
                Token::Numeric(Mode::Exponent) => {}
                Token::NumericOpt(Mode::Exponent) => {}
                Token::Currency => {
                    buffer[i] = '\u{11}';
                }
                Token::Separator(v) => {
                    buffer[i] = *v;
                }
            }
        }

        let mut d = None;
        for (i, m) in format.tok.iter().enumerate().rev() {
            if d.is_none() {
                d = it_integer.next_back();
            }

            match m {
                Token::Digit0(Mode::Exponent) => {
                    if let Some(d) = it_exp.next_back() {
                        buffer[i] = d;
                    } else {
                        buffer[i] = '0';
                    }
                }
                Token::Digit(Mode::Exponent) => {
                    if let Some(d) = it_exp.next_back() {
                        buffer[i] = d;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::Numeric(Mode::Exponent) => {
                    if let Some(d) = it_exp.next_back() {
                        buffer[i] = d;
                    } else if exp_sign == "-" {
                        buffer[i] = sym.negative_sym;
                        exp_sign = "";
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::NumericOpt(Mode::Exponent) => {
                    if let Some(d) = it_exp.next_back() {
                        buffer[i] = d;
                    } else if exp_sign == "-" {
                        buffer[i] = sym.negative_sym;
                        exp_sign = "";
                    } else {
                        buffer[i] = '\u{00}';
                    }
                }
                Token::Digit0(Mode::Integer) => {
                    if let Some(dd) = d {
                        d = None;
                        buffer[i] = dd;
                    } else {
                        buffer[i] = '0';
                    }
                }
                Token::Digit(Mode::Integer) => {
                    if let Some(dd) = d {
                        d = None;
                        buffer[i] = dd;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::Numeric(Mode::Integer) => {
                    if let Some(dd) = d {
                        d = None;
                        buffer[i] = dd;
                    } else if sign == "-" {
                        buffer[i] = sym.negative_sym;
                        sign = "";
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::NumericOpt(Mode::Integer) => {
                    if let Some(dd) = d {
                        d = None;
                        buffer[i] = dd;
                    } else if sign == "-" {
                        buffer[i] = sym.negative_sym;
                        sign = "";
                    } else {
                        buffer[i] = '\u{00}';
                    }
                }
                Token::GroupingSep => {
                    if d.is_some() {
                        buffer[i] = sym.decimal_grp;
                    } else {
                        buffer[i] = ' ';
                    }
                }
                Token::GroupingSepAlways => {
                    buffer[i] = sym.decimal_grp;
                }
                _ => {}
            }
        }

        if !sign.is_empty() {
            return Err(FmtError);
        }
        if d.is_some() {
            return Err(FmtError);
        }
        // missing fractions are ok.
        // shouldn't occur, we give the precision to display.
        debug_assert!(it_fraction.next().is_none());
        if !exp_sign.is_empty() {
            return Err(FmtError);
        }
        if it_exp.next().is_some() {
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
            write!(raw, "{:.*e}", format.precision as usize, number).map_err(|_| FmtError)?;
        } else {
            write!(raw, "{:.*}", format.precision as usize, number).map_err(|_| FmtError)?;
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
