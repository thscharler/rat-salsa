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
//! * `\` - all ascii characters (ascii 32-128!) are reserved and must be escaped.
//! * `_` - other unicode characters can be used without escaping.
//!

use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Write as FmtWrite};
use std::io::{Cursor, Write as IoWrite};
use std::mem::size_of;
use std::rc::Rc;
use std::str::from_utf8_unchecked;
use std::{fmt, mem};

/// Symbols for number formatting.
#[derive(Debug, PartialEq, Eq, Clone)]
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
}

impl Default for NumberSymbols {
    fn default() -> Self {
        Self {
            decimal_sep: '.',
            decimal_grp: ',',
            negative_sym: '-',
            positive_sym: '+',
            exponent_upper_sym: 'E',
            exponent_lower_sym: 'e',
        }
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
    /// Other separator char to output literally. May be escaped with '\\'.
    Separator(char),
}

/// Holds the pattern for the numberformat and some additional data.
#[derive(Default, Clone)]
pub struct NumberFormat {
    /// Decides what std-format is used. If true it's `{:e}` otherwise plain `{}`
    pub has_exp: bool,
    /// The required precision for this format. Is used for the underlying std-format.
    pub precision: u8,
    /// Tokens.
    pub tok: Vec<Token>,
    /// Symbols.
    pub sym: Option<Rc<NumberSymbols>>,
}

impl Display for NumberFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for t in &self.tok {
            match t {
                Token::Digit0(_) => f.write_char('0')?,
                Token::Digit(_) => f.write_char('9')?,
                Token::Numeric(_) => f.write_char('#')?,
                Token::NumericOpt(_) => f.write_char('8')?,
                Token::Plus(_) => f.write_char('+')?,
                Token::Minus(_) => f.write_char('-')?,
                Token::DecimalSep => f.write_char('.')?,
                Token::DecimalSepAlways => f.write_char(':')?,
                Token::GroupingSep => f.write_char(',')?,
                Token::GroupingSepAlways => f.write_char(';')?,
                Token::ExponentUpper => f.write_char('E')?,
                Token::ExponentUpperAlways => f.write_char('F')?,
                Token::ExponentLower => f.write_char('e')?,
                Token::ExponentLowerAlways => f.write_char('f')?,
                Token::Separator(c) => {
                    match c {
                        '0' | '9' | '#' | '8' | '+' | '-' | ',' | ';' | '.' | ':' | 'E' | 'F'
                        | 'e' | 'f' => {
                            f.write_char('\\')?;
                        }
                        _ => {}
                    }
                    f.write_char(*c)?;
                }
            }
        }
        Ok(())
    }
}

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

/// Parses the format string for reuse.
pub fn parse_format(f: &str) -> Result<NumberFormat, fmt::Error> {
    let mut format = NumberFormat::default();

    let mut esc = false;
    let mut mode = Mode::Integer;
    let mut exp_0 = false;
    let mut frac_0 = false;

    for m in f.chars() {
        let mask = if esc {
            esc = false;
            Token::Separator(m)
        } else {
            match m {
                '0' => {
                    if mode == Mode::Fraction {
                        frac_0 = true;
                        format.precision += 1;
                    } else if mode == Mode::Exponent {
                        exp_0 = true;
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
                        return Err(fmt::Error);
                    }
                    mode = Mode::Fraction;
                    Token::DecimalSep
                }
                ':' => {
                    if matches!(mode, Mode::Fraction | Mode::Exponent) {
                        return Err(fmt::Error);
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
                        return Err(fmt::Error);
                    }
                    format.has_exp = true;
                    mode = Mode::Exponent;
                    Token::ExponentLower
                }
                'E' => {
                    if mode == Mode::Exponent {
                        return Err(fmt::Error);
                    }
                    format.has_exp = true;
                    mode = Mode::Exponent;
                    Token::ExponentUpper
                }
                'f' => {
                    if mode == Mode::Exponent {
                        return Err(fmt::Error);
                    }
                    format.has_exp = true;
                    mode = Mode::Exponent;
                    Token::ExponentLower
                }
                'F' => {
                    if mode == Mode::Exponent {
                        return Err(fmt::Error);
                    }
                    format.has_exp = true;
                    mode = Mode::Exponent;
                    Token::ExponentUpper
                }
                '\\' => {
                    esc = true;
                    continue;
                }
                ' ' => Token::Separator(' '),
                c if c.is_ascii() => return Err(fmt::Error),
                c => Token::Separator(c),
            }
        };
        format.tok.push(mask);
    }

    for t in format.tok.iter_mut().rev() {
        match t {
            Token::DecimalSep => {
                if frac_0 {
                    *t = Token::DecimalSepAlways;
                }
            }
            Token::ExponentLower => {
                if exp_0 {
                    *t = Token::ExponentLowerAlways;
                }
            }
            Token::ExponentUpper => {
                if exp_0 {
                    *t = Token::ExponentUpperAlways;
                }
            }
            _ => {}
        }
    }

    Ok(format)
}

/// Parses the format string and sets a symbol table.
pub fn parse_format_sym(
    format: &str,
    sym: Option<&Rc<NumberSymbols>>,
) -> Result<NumberFormat, std::fmt::Error> {
    parse_format(format).map(|mut v| {
        v.sym = sym.cloned();
        v
    })
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

/// Unmap the formatted string back to a format that `f64::parse()` can understand.
///
/// Token::NumericOpt is not supported for now.
pub fn unmap_num<W: fmt::Write>(
    formatted: &str,
    format: &NumberFormat,
    out: &mut W,
) -> Result<(), fmt::Error> {
    for (t, c) in format.tok.iter().zip(formatted.chars()) {
        match t {
            Token::Digit0(_) => {
                out.write_char(c)?;
            }
            Token::Digit(_) => {
                if c != ' ' {
                    out.write_char(c)?;
                }
            }
            Token::Numeric(_) => {
                if c != ' ' {
                    out.write_char(c)?;
                }
            }
            Token::NumericOpt(_) => {
                unimplemented!("NumericOpt not supported.");
            }
            Token::Plus(_) => {
                if c == '-' {
                    out.write_char('-')?;
                }
            }
            Token::Minus(_) => {
                if c == '-' {
                    out.write_char('-')?;
                }
            }
            Token::DecimalSep => {
                out.write_char('.')?;
            }
            Token::DecimalSepAlways => {
                out.write_char('.')?;
            }
            Token::GroupingSep => {
                // noop
            }
            Token::GroupingSepAlways => {
                // noop
            }
            Token::ExponentUpper => {
                out.write_char('E')?;
            }
            Token::ExponentUpperAlways => {
                out.write_char('E')?;
            }
            Token::ExponentLower => {
                out.write_char('e')?;
            }
            Token::ExponentLowerAlways => {
                out.write_char('e')?;
            }
            Token::Separator(_) => {
                // noop
            }
        }
    }

    Ok(())
}

/// Takes a raw number string and applies the format.
///
/// The raw number should be in a format produced by the format! macro. decimal point is '.'
/// and exponent is 'e' or 'E'.
///
/// There is a need for a buffer, its length must be at least format.tok.len().
#[allow(clippy::needless_range_loop)]
pub fn map_num<W: fmt::Write>(
    raw: &str,
    format: &NumberFormat,
    out: &mut W,
) -> Result<(), fmt::Error> {
    thread_local! {
        static BUF: RefCell<[*const str;32]> = const { RefCell::new(["";32]) };
    }

    BUF.with_borrow_mut(|buf| {
        // Safety:
        // This transmutes pointers to true static str to the required shorter lifetime
        // &'d str of map_num. It's initialized with pointers to true static strings,
        // so any residual values are cleaned up.
        let buffer: &mut [char] = unsafe {
            for b in buf.iter_mut() {
                *b = "";
            }
            assert_eq!(size_of::<*const str>(), size_of::<&str>());
            mem::transmute(&mut buf[..])
        };

        if buffer.len() < format.tok.len() {
            return Err(fmt::Error);
        }

        _map_num(raw, format, buffer)?;

        for i in 0..format.tok.len() {
            if buffer[i] != '\u{7f}' {
                if out.write_char(buffer[i]).is_err() {
                    return Err(fmt::Error);
                }
            }
        }

        Ok(())
    })
}

// impl without type parameters
#[allow(clippy::needless_range_loop)]
fn _map_num(raw: &str, format: &NumberFormat, buffer: &mut [char]) -> Result<(), fmt::Error> {
    for i in 0..format.tok.len() {
        buffer[i] = ' ';
    }

    let sym = format.sym.as_ref().map(|v| v.as_ref());

    let (mut sign, integer, fraction, mut exp_sign, exp) = split_num(raw);
    let mut it_integer = integer.chars();
    let mut it_fraction = fraction.chars();
    let mut it_exp = exp.chars();

    for (i, m) in format.tok.iter().enumerate() {
        match m {
            Token::Plus(Mode::Integer) => {
                if sign.is_empty() {
                    buffer[i] = sym.map(|v| v.positive_sym).unwrap_or('+');
                } else {
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
                }
                sign = "";
            }
            Token::Minus(Mode::Integer) => {
                if sign.is_empty() {
                    buffer[i] = ' ';
                } else {
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
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
                if !fraction.is_empty() {
                    buffer[i] = sym.map(|v| v.decimal_sep).unwrap_or('.');
                } else {
                    buffer[i] = ' ';
                }
            }
            Token::DecimalSepAlways => {
                buffer[i] = sym.map(|v| v.decimal_sep).unwrap_or('.');
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
                    buffer[i] = '\u{7f}';
                }
            }

            Token::ExponentUpper => {
                if !exp.is_empty() {
                    buffer[i] = sym.map(|v| v.exponent_upper_sym).unwrap_or('E');
                } else {
                    buffer[i] = ' ';
                }
            }
            Token::ExponentLower => {
                if !exp.is_empty() {
                    buffer[i] = sym.map(|v| v.exponent_lower_sym).unwrap_or('E');
                } else {
                    buffer[i] = ' ';
                }
            }
            Token::ExponentUpperAlways => {
                buffer[i] = sym.map(|v| v.exponent_upper_sym).unwrap_or('E');
            }
            Token::ExponentLowerAlways => {
                buffer[i] = sym.map(|v| v.exponent_lower_sym).unwrap_or('e');
            }

            Token::Plus(Mode::Exponent) => {
                if exp_sign.is_empty() {
                    buffer[i] = sym.map(|v| v.positive_sym).unwrap_or('+');
                } else {
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
                }
                exp_sign = "";
            }
            Token::Minus(Mode::Exponent) => {
                if exp_sign.is_empty() {
                    buffer[i] = ' ';
                } else {
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
                }
                exp_sign = "";
            }
            Token::Digit0(Mode::Exponent) => {}
            Token::Digit(Mode::Exponent) => {}
            Token::Numeric(Mode::Exponent) => {}
            Token::NumericOpt(Mode::Exponent) => {}
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
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
                    exp_sign = "";
                } else {
                    buffer[i] = ' ';
                }
            }
            Token::NumericOpt(Mode::Exponent) => {
                if let Some(d) = it_exp.next_back() {
                    buffer[i] = d;
                } else if exp_sign == "-" {
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
                    exp_sign = "";
                } else {
                    buffer[i] = '\u{7f}';
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
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
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
                    buffer[i] = sym.map(|v| v.negative_sym).unwrap_or('-');
                    sign = "";
                } else {
                    buffer[i] = '\u{7F}';
                }
            }
            Token::GroupingSep => {
                if d.is_some() {
                    buffer[i] = sym.map(|v| v.decimal_grp).unwrap_or(',');
                } else {
                    buffer[i] = ' ';
                }
            }
            Token::GroupingSepAlways => {
                buffer[i] = sym.map(|v| v.decimal_grp).unwrap_or(',');
            }
            _ => {}
        }
    }

    if !sign.is_empty() {
        return Err(fmt::Error);
    }
    if d.is_some() {
        return Err(fmt::Error);
    }
    // missing fractions are ok.
    if !exp_sign.is_empty() {
        return Err(fmt::Error);
    }
    if it_exp.next().is_some() {
        return Err(fmt::Error);
    }

    Ok(())
}

/// Format a f64 according to the format string.
pub fn fmt_f64<Number: Into<f64>>(
    number: Number,
    format: &str,
    sym: Option<&Rc<NumberSymbols>>,
) -> Result<String, fmt::Error> {
    let format = parse_format_sym(format, sym)?;
    let mut out = String::new();
    format_f64_to(number, &format, &mut out)?;
    Ok(out)
}

/// Format a f64 according to the format string.
pub fn fmt_f64_to<W: fmt::Write, Number: Into<f64>>(
    number: Number,
    format: &str,
    sym: Option<&Rc<NumberSymbols>>,
    out: &mut W,
) -> Result<(), fmt::Error> {
    let format = parse_format_sym(format, sym)?;
    format_f64_to(number, &format, out)
}

/// Format a f64 according to the format.
pub fn format_f64<Number: Into<f64>>(
    number: Number,
    format: &NumberFormat,
) -> Result<String, fmt::Error> {
    let mut out = String::new();
    format_f64_to(number, format, &mut out)?;
    Ok(out)
}

/// Format a f64 according to the format.
pub fn format_f64_to<W: fmt::Write, Number: Into<f64>>(
    number: Number,
    format: &NumberFormat,
    out: &mut W,
) -> Result<(), fmt::Error> {
    thread_local! {
        static RAW: RefCell<Cursor<[u8;32]>> = RefCell::new(Cursor::new([0u8;32]));
    }

    RAW.with_borrow_mut(|raw| {
        raw.set_position(0);
        if format.has_exp {
            write!(raw, "{:.*e}", format.precision as usize, number.into())
                .map_err(|_| fmt::Error)?;
        } else {
            write!(raw, "{:.*}", format.precision as usize, number.into())
                .map_err(|_| fmt::Error)?;
        };
        // Safety:
        // Output is ascii.
        let raw_str = unsafe { from_utf8_unchecked(&raw.get_ref()[..raw.position() as usize]) };

        map_num(raw_str, format, out)
    })
}

/// Format an i64 according to the format string.
pub fn fmt_i64<Number: Into<i64>>(
    number: Number,
    format: &str,
    sym: Option<&Rc<NumberSymbols>>,
) -> Result<String, fmt::Error> {
    let format = parse_format_sym(format, sym)?;
    let mut out = String::new();
    format_i64_to(number, &format, &mut out)?;
    Ok(out)
}

/// Format an i64 according to the format string.
pub fn fmt_i64_to<W: fmt::Write, Number: Into<i64>>(
    number: Number,
    format: &str,
    sym: Option<&Rc<NumberSymbols>>,
    out: &mut W,
) -> Result<(), fmt::Error> {
    let format = parse_format_sym(format, sym)?;
    format_i64_to(number, &format, out)
}

/// Format an i64 according to the format.
pub fn format_i64<Number: Into<i64>>(
    number: Number,
    format: &NumberFormat,
) -> Result<String, fmt::Error> {
    let mut out = String::new();
    format_i64_to(number, format, &mut out)?;
    Ok(out)
}

/// Format an i64 according to the format.
pub fn format_i64_to<W: fmt::Write, Number: Into<i64>>(
    number: Number,
    format: &NumberFormat,
    out: &mut W,
) -> Result<(), fmt::Error> {
    thread_local! {
        static RAW: RefCell<Cursor<[u8;32]>> = RefCell::new(Cursor::new([0u8;32]));
    }

    RAW.with_borrow_mut(|raw| {
        raw.set_position(0);
        write!(raw, "{}", number.into()).map_err(|_| fmt::Error)?;
        // Safety:
        // Output is ascii.
        let raw_str = unsafe { from_utf8_unchecked(&raw.get_ref()[..raw.position() as usize]) };

        map_num(raw_str, format, out)
    })
}
