use crate::_private::NonExhaustive;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Separator style
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Separator {
    #[default]
    None,
    Empty,
    Plain,
    Thick,
    Double,
    Dashed,
    Dotted,
}

/// Menu item
#[derive(Debug, Clone)]
pub struct MenuItem<'a> {
    /// Menuitem text
    pub item: &'a str,
    /// Text range to highlight
    pub highlight: Option<Range<usize>>,
    /// Navigation key char
    pub navchar: Option<char>,
    /// Right aligned text
    pub right: &'a str,
    /// Disabled style
    pub disabled: bool,

    /// Separator after the item.
    pub sep: Separator,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> MenuItem<'a> {
    pub fn new_menu_str(s: &'a str) -> Self {
        menu_str(s)
    }

    pub fn new(text: &'a str) -> Self {
        Self {
            item: text,
            highlight: None,
            navchar: None,
            right: "",
            disabled: false,
            sep: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    pub fn new_nav(text: &'a str, highlight: Range<usize>, navchar: char) -> Self {
        Self {
            item: text,
            highlight: Some(highlight),
            navchar: Some(navchar),
            right: "",
            disabled: false,
            sep: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    pub fn width(&self) -> u16 {
        (self.item.graphemes(true).count() + self.right.graphemes(true).count()) as u16
    }

    pub fn height(&self) -> u16 {
        if self.sep == Separator::None {
            1
        } else {
            2
        }
    }
}

pub fn is_separator_str(s: &str) -> bool {
    if s == "_   " {
        true
    } else if s == "____" {
        true
    } else if s == "_______" {
        true
    } else if s == "_===" {
        true
    } else if s == "_---" {
        true
    } else if s == "_..." {
        true
    } else {
        false
    }
}

/// This uses `_` (underscore) as prefix and
/// a fixed string to identify the separator:
///
/// * `_   ` - three blanks -> empty separator
/// * `____` - three underscores -> plain line
/// * `_______` - six underscore -> thick line
/// * `_===` - three equals -> double line
/// * `_---` - three hyphen -> dashed line
/// * `_...` - three dots -> dotted line
pub fn separator_str(s: &str) -> Separator {
    if s == "_   " {
        Separator::Empty
    } else if s == "____" {
        Separator::Plain
    } else if s == "_______" {
        Separator::Thick
    } else if s == "_===" {
        Separator::Double
    } else if s == "_---" {
        Separator::Dashed
    } else if s == "_..." {
        Separator::Dotted
    } else {
        unreachable!()
    }
}

/// Create a Line from the given text.
/// The first '_' marks the navigation-char.
pub fn menu_str(txt: &str) -> MenuItem<'_> {
    let mut idx_underscore = None;
    let mut idx_navchar_start = None;
    let mut idx_navchar_end = None;
    let cit = txt.char_indices();
    for (idx, c) in cit {
        if idx_underscore.is_none() && c == '_' {
            idx_underscore = Some(idx);
        } else if idx_underscore.is_some() && idx_navchar_start.is_none() {
            idx_navchar_start = Some(idx);
        } else if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
            idx_navchar_end = Some(idx);
        }
    }
    if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
        idx_navchar_end = Some(txt.len());
    }

    if let Some(idx_navchar_start) = idx_navchar_start {
        if let Some(idx_navchar_end) = idx_navchar_end {
            MenuItem::new_nav(
                txt,
                idx_navchar_start..idx_navchar_end,
                txt[idx_navchar_start..idx_navchar_end]
                    .chars()
                    .next()
                    .expect("char"),
            )
        } else {
            unreachable!();
        }
    } else {
        MenuItem::new(txt)
    }
}
