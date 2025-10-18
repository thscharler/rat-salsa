use crate::upos_type;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

/// Edit direction for part of a mask.
/// Numeric values can switch between right-to-left (integer part) and left-to-right (fraction).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditDirection {
    Ltor,
    Rtol,
}

/// One char of the input mask.
#[allow(variant_size_differences)]
#[derive(Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Mask {
    // 0-9, display 0
    Digit0(EditDirection),
    // 0-9, display space
    Digit(EditDirection),
    // 0-9;sign, display space
    Numeric(EditDirection),
    DecimalSep,
    GroupingSep,
    Sign,
    Plus,
    Hex0,
    Hex,
    Oct0,
    Oct,
    Dec0,
    Dec,
    Letter,
    LetterOrDigit,
    LetterDigitSpace,
    AnyChar,
    Separator(Box<str>),
    #[default]
    None,
}

/// One token of the input mask.
///
/// Which field of the mask does this belong to:
/// * Number with integer part, decimal separator, fraction and sign.
/// * Consecutive mask parts of similar type.
///
/// At this cursor position, what is the main mask (right) and what is possible left of
/// the cursor position (peek_left).
///
/// Default-values for editing and display.
#[derive(Clone, PartialEq, Eq)]
pub struct MaskToken {
    pub sec_id: u16,
    // section/number
    pub sec_start: upos_type,
    // section/number
    pub sec_end: upos_type,
    // part of a number/section
    pub sub_start: upos_type,
    // part of a number/section
    pub sub_end: upos_type,

    // token left of the cursor
    pub peek_left: Mask,
    // token right of the cursor
    pub right: Mask,

    // edit-value of the token
    pub edit: Box<str>,
}

impl Debug for EditDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EditDirection::Ltor => ">",
                EditDirection::Rtol => "<",
            }
        )
    }
}

impl Display for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Mask::Digit0(_) => "0",
            Mask::Digit(_) => "9",
            Mask::Numeric(_) => "#",
            Mask::DecimalSep => ".",
            Mask::GroupingSep => ",",
            Mask::Sign => "-",
            Mask::Plus => "+",
            Mask::Hex0 => "H",
            Mask::Hex => "h",
            Mask::Oct0 => "O",
            Mask::Oct => "o",
            Mask::Dec0 => "D",
            Mask::Dec => "d",
            Mask::Letter => "l",
            Mask::LetterOrDigit => "a",
            Mask::LetterDigitSpace => "c",
            Mask::AnyChar => "_",
            Mask::Separator(s) => {
                write!(f, "\\")?;
                s
            }
            Mask::None => "",
        };
        write!(f, "{}", s)
    }
}

impl Debug for Mask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Mask::Digit0(d) => {
                write!(f, "{:?}0", d)
            }
            Mask::Digit(d) => {
                write!(f, "{:?}9", d)
            }
            Mask::Numeric(d) => {
                write!(f, "{:?}#", d)
            }
            Mask::DecimalSep => write!(f, "."),
            Mask::GroupingSep => write!(f, ","),
            Mask::Sign => write!(f, "-"),
            Mask::Plus => write!(f, "+"),
            Mask::Hex0 => write!(f, "H"),
            Mask::Hex => write!(f, "h"),
            Mask::Oct0 => write!(f, "O"),
            Mask::Oct => write!(f, "o"),
            Mask::Dec0 => write!(f, "D"),
            Mask::Dec => write!(f, "d"),
            Mask::Letter => write!(f, "l"),
            Mask::LetterOrDigit => write!(f, "a"),
            Mask::LetterDigitSpace => write!(f, "c"),
            Mask::AnyChar => write!(f, "_"),
            Mask::Separator(s) => {
                write!(f, "\\")?;
                write!(f, "{}", s)
            }
            Mask::None => write!(f, ""),
        }
    }
}

impl EditDirection {
    pub(crate) fn is_ltor(&self) -> bool {
        *self == EditDirection::Ltor
    }

    pub(crate) fn is_rtol(&self) -> bool {
        *self == EditDirection::Rtol
    }
}

impl Mask {
    /// is not editable. the last field of the mask at position txt.len() can not be edited,
    /// but it's a valid cursor position.
    pub(crate) fn is_none(&self) -> bool {
        *self == Mask::None
    }

    /// left to right editing
    #[inline]
    pub(crate) fn is_ltor(&self) -> bool {
        match self {
            Mask::Digit0(d) => d.is_ltor(),
            Mask::Digit(d) => d.is_ltor(),
            Mask::Numeric(d) => d.is_ltor(),
            Mask::GroupingSep => false,
            Mask::Sign => false,
            Mask::Plus => false,
            Mask::DecimalSep => true,
            Mask::Hex0 => true,
            Mask::Hex => true,
            Mask::Oct0 => true,
            Mask::Oct => true,
            Mask::Dec0 => true,
            Mask::Dec => true,
            Mask::Letter => true,
            Mask::LetterOrDigit => true,
            Mask::LetterDigitSpace => true,
            Mask::AnyChar => true,
            Mask::Separator(_) => true,
            Mask::None => false,
        }
    }

    /// right to left editing
    #[inline]
    pub(crate) fn is_rtol(&self) -> bool {
        match self {
            Mask::Digit0(d) => d.is_rtol(),
            Mask::Digit(d) => d.is_rtol(),
            Mask::Numeric(d) => d.is_rtol(),
            Mask::GroupingSep => true,
            Mask::Sign => true,
            Mask::Plus => true,
            Mask::DecimalSep => false,
            Mask::Hex0 => false,
            Mask::Hex => false,
            Mask::Oct0 => false,
            Mask::Oct => false,
            Mask::Dec0 => false,
            Mask::Dec => false,
            Mask::Letter => false,
            Mask::LetterOrDigit => false,
            Mask::LetterDigitSpace => false,
            Mask::AnyChar => false,
            Mask::Separator(_) => false,
            Mask::None => false,
        }
    }

    /// is a number mask
    #[inline]
    pub(crate) fn is_number(&self) -> bool {
        match self {
            Mask::Digit0(_)
            | Mask::Digit(_)
            | Mask::Numeric(_)
            | Mask::DecimalSep
            | Mask::GroupingSep
            | Mask::Sign
            | Mask::Plus => true,
            Mask::None => false,
            _ => false,
        }
    }

    /// is a separator
    #[inline]
    pub(crate) fn is_separator(&self) -> bool {
        match self {
            Mask::Separator(_) => true,
            Mask::None => false,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_fraction(&self) -> bool {
        match self {
            Mask::Digit0(d) | Mask::Digit(d) | Mask::Numeric(d) => d.is_ltor(),
            Mask::None => false,
            _ => false,
        }
    }

    /// which mask-types are put together.
    #[inline]
    pub(crate) fn sub_section(&self) -> u8 {
        match self {
            Mask::Digit0(_) => 0,
            Mask::Digit(_) => 0,
            Mask::Numeric(_) => 0,
            Mask::GroupingSep => 0,

            Mask::Sign => 1,

            Mask::Plus => 2,

            Mask::DecimalSep => 3,

            Mask::Hex0 => 4,
            Mask::Hex => 4,

            Mask::Oct0 => 5,
            Mask::Oct => 5,

            Mask::Dec0 => 6,
            Mask::Dec => 6,

            Mask::Letter => 7,
            Mask::LetterOrDigit => 8,
            Mask::LetterDigitSpace => 9,
            Mask::AnyChar => 10,

            Mask::Separator(_) => 11,

            Mask::None => 12,
        }
    }

    /// which mask-types constitute a number/section
    #[inline]
    pub(crate) fn section(&self) -> u8 {
        match self {
            Mask::Digit0(_) => 0,
            Mask::Digit(_) => 0,
            Mask::Numeric(_) => 0,
            Mask::GroupingSep => 0,
            Mask::Sign => 0,
            Mask::Plus => 0,
            Mask::DecimalSep => 0,

            Mask::Hex0 => 1,
            Mask::Hex => 1,
            Mask::Oct0 => 1,
            Mask::Oct => 1,
            Mask::Dec0 => 1,
            Mask::Dec => 1,
            Mask::Letter => 1,
            Mask::LetterOrDigit => 1,
            Mask::LetterDigitSpace => 1,
            Mask::AnyChar => 1,

            Mask::Separator(_) => 2,
            Mask::None => 3,
        }
    }

    /// mask should overwrite instead of insert
    #[inline]
    pub(crate) fn can_overwrite_fraction(&self, c: &str) -> bool {
        match self {
            Mask::Digit0(_) => c == "0",
            Mask::Digit(_) | Mask::Numeric(_) => c == " ",
            Mask::DecimalSep => false,
            Mask::GroupingSep => false,
            Mask::Sign => false,
            Mask::Plus => false,
            Mask::Hex0 => false,
            Mask::Hex => false,
            Mask::Oct0 => false,
            Mask::Oct => false,
            Mask::Dec0 => false,
            Mask::Dec => false,
            Mask::Letter => false,
            Mask::LetterOrDigit => false,
            Mask::LetterDigitSpace => false,
            Mask::AnyChar => false,
            Mask::Separator(_) => false,
            Mask::None => false,
        }
    }

    /// mask should overwrite instead of insert
    #[inline]
    pub(crate) fn can_overwrite(&self, c: &str) -> bool {
        match self {
            Mask::Digit0(_) | Mask::Digit(_) | Mask::Numeric(_) => false,
            Mask::DecimalSep => "." == c,
            Mask::GroupingSep => false,
            Mask::Sign => "-" == c || " " == c,
            Mask::Plus => "-" == c || "+" == c || " " == c,
            Mask::Hex0 => c == "0",
            Mask::Hex => false,
            Mask::Oct0 => c == "0",
            Mask::Oct => false,
            Mask::Dec0 => c == "0",
            Mask::Dec => false,
            Mask::Letter => false,
            Mask::LetterOrDigit => false,
            Mask::LetterDigitSpace => false,
            Mask::AnyChar => false,
            Mask::Separator(sep) => sep.as_ref() == c,
            Mask::None => false,
        }
    }

    /// char can be dropped from the text and it's ok.
    #[inline]
    pub(crate) fn can_drop(&self, c: &str) -> bool {
        match self {
            Mask::Digit0(_) => c == "0",
            Mask::Digit(_) => c == " ",
            Mask::Numeric(_) => c == " ",
            Mask::DecimalSep => false,
            Mask::Sign => false,
            Mask::Plus => false,
            Mask::GroupingSep => true,
            Mask::Hex0 => c == "0",
            Mask::Hex => c == " ",
            Mask::Oct0 => c == "0",
            Mask::Oct => c == " ",
            Mask::Dec0 => c == "0",
            Mask::Dec => c == " ",
            Mask::Letter => c == " ",
            Mask::LetterOrDigit => c == " ",
            Mask::LetterDigitSpace => c == " ",
            Mask::AnyChar => c == " ",
            Mask::Separator(_sep) => false,
            Mask::None => false,
        }
    }

    /// default char for this mask.
    #[inline]
    pub(crate) fn edit_value(&self) -> &str {
        match self {
            Mask::Digit0(_) => "0",
            Mask::Digit(_) => " ",
            Mask::Numeric(_) => " ",
            Mask::DecimalSep => ".",
            Mask::GroupingSep => " ", // don't show. reformat fills it in if necessary.
            Mask::Sign => " ",
            Mask::Plus => "+",
            Mask::Hex0 => "0",
            Mask::Hex => " ",
            Mask::Oct0 => "0",
            Mask::Oct => " ",
            Mask::Dec0 => "0",
            Mask::Dec => " ",
            Mask::Letter => " ",
            Mask::LetterOrDigit => " ",
            Mask::LetterDigitSpace => " ",
            Mask::AnyChar => " ",
            Mask::Separator(g) => g.as_ref(),
            Mask::None => "",
        }
    }
}

impl Debug for MaskToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Mask #{}:{}-{} {:?} | {:?}",
            self.sec_id, self.sub_start, self.sub_end, self.peek_left, self.right
        )
    }
}

impl MaskToken {
    /// is somewhere in the integer part of a number.
    #[inline]
    pub(crate) fn is_integer_part(&self) -> bool {
        self.peek_left.is_rtol() || self.peek_left.is_none() && self.right.is_rtol()
    }

    /// Create a string with the default edit mask.
    pub(crate) fn empty_section(mask: &[MaskToken]) -> String {
        let mut buf = String::new();
        for m in mask {
            buf.push_str(&m.edit);
        }
        buf
    }
}
