use crate::_private::NonExhaustive;
use std::borrow::Cow;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Separator style
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Separator {
    #[default]
    Plain,
    Empty,
    Thick,
    Double,
    Dashed,
    Dotted,
}

/// Menu item
#[derive(Debug, Clone)]
pub struct MenuItem<'a> {
    /// Menuitem text
    pub item: Cow<'a, str>,
    /// Text range to highlight
    pub highlight: Option<Range<usize>>,
    /// Navigation key char
    pub navchar: Option<char>,
    /// Right aligned text
    pub right: Cow<'a, str>,
    /// Disabled style
    pub disabled: bool,

    /// Separator after the item.
    pub separator: Option<Separator>,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for MenuItem<'a> {
    fn default() -> Self {
        Self {
            item: Default::default(),
            highlight: None,
            navchar: None,
            right: Default::default(),
            disabled: false,
            separator: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> MenuItem<'a> {
    pub fn new() -> Self {
        Self {
            item: Default::default(),
            highlight: None,
            navchar: None,
            right: Default::default(),
            disabled: false,
            separator: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// Uses '_' as special character.
    ///
    /// __Item__
    ///
    /// The first '_' marks the navigation-char.
    /// Pipe '|' separates the item text and the right text.
    ///
    /// __Separator__
    ///
    /// `_` (underscore) is used as prefix and then
    /// a fixed string to identify the separator:
    ///
    /// * `_   ` - three blanks -> empty separator
    /// * `____` - three underscores -> plain line
    /// * `_______` - six underscore -> thick line
    /// * `_===` - three equals -> double line
    /// * `_---` - three hyphen -> dashed line
    /// * `_...` - three dots -> dotted line
    ///
    pub fn new_parsed(s: &'a str) -> Self {
        if is_separator_str(s) {
            Self::new_sep(separator_str(s))
        } else {
            item_str(s)
        }
    }

    /// New borrowed string as item text.
    pub fn new_str(text: &'a str) -> Self {
        Self {
            item: Cow::Borrowed(text),
            highlight: None,
            navchar: None,
            right: Cow::Borrowed(""),
            disabled: false,
            separator: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// New with owned string as item text.
    pub fn new_string(text: String) -> Self {
        Self {
            item: Cow::Owned(text.into()),
            highlight: None,
            navchar: None,
            right: Default::default(),
            disabled: false,
            separator: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// New with navigation char and highlight.
    /// Highlight here is a byte range into the text.
    pub fn new_nav_str(text: &'a str, highlight: Range<usize>, navchar: char) -> Self {
        Self {
            item: Cow::Borrowed(text),
            highlight: Some(highlight),
            navchar: Some(navchar),
            right: Cow::Borrowed(""),
            disabled: false,
            separator: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// New with navigation char and highlight.
    /// Highlight here is a byte range into the text.
    pub fn new_nav_string(text: String, highlight: Range<usize>, navchar: char) -> Self {
        Self {
            item: Cow::Owned(text),
            highlight: Some(highlight),
            navchar: Some(navchar),
            right: Cow::Borrowed(""),
            disabled: false,
            separator: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }

    /// New separator.
    ///
    /// Such a menu item will be merged with the one before, unless
    /// you set some item-text later.
    pub fn new_sep(separator: Separator) -> Self {
        Self {
            item: Default::default(),
            highlight: None,
            navchar: None,
            right: Default::default(),
            disabled: false,
            separator: Some(separator),
            non_exhaustive: NonExhaustive,
        }
    }

    /// Set the right text.
    pub fn right(mut self, right: &'a str) -> Self {
        self.right = Cow::Borrowed(right);
        self
    }

    /// Set disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Adds a separator after the menuitem.
    pub fn separator(mut self, separator: Separator) -> Self {
        self.separator = Some(separator);
        self
    }

    /// Text-width in graphemes for item + right.
    pub fn width(&self) -> u16 {
        (self.item.graphemes(true).count() + self.right.graphemes(true).count()) as u16
            - if self.navchar.is_some() { 1 } else { 0 }
    }

    /// Text-height.
    pub fn height(&self) -> u16 {
        if self.separator.is_none() {
            1
        } else {
            2
        }
    }
}

fn is_separator_str(s: &str) -> bool {
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
fn separator_str(s: &str) -> Separator {
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
/// Pipe '|' separates the item text and the right text.
fn item_str(txt: &str) -> MenuItem<'_> {
    let mut idx_underscore = None;
    let mut idx_navchar_start = None;
    let mut idx_navchar_end = None;
    let mut idx_pipe = None;
    let cit = txt.char_indices();
    for (idx, c) in cit {
        if idx_underscore.is_none() && c == '_' {
            idx_underscore = Some(idx);
        } else if idx_underscore.is_some() && idx_navchar_start.is_none() {
            idx_navchar_start = Some(idx);
        } else if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
            idx_navchar_end = Some(idx);
        } else if c == '|' {
            idx_pipe = Some(idx);
        }
    }
    if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
        idx_navchar_end = Some(txt.len());
    }

    if let Some(pipe) = idx_pipe {
        if let Some(navchar_end) = idx_navchar_end {
            if pipe < navchar_end {
                idx_pipe = None;
            }
        }
    }

    let (text, right) = if let Some(idx_pipe) = idx_pipe {
        (&txt[..idx_pipe], &txt[idx_pipe + 1..])
    } else {
        (txt, "")
    };

    if let Some(idx_navchar_start) = idx_navchar_start {
        if let Some(idx_navchar_end) = idx_navchar_end {
            MenuItem {
                item: Cow::Borrowed(text),
                highlight: Some(idx_navchar_start..idx_navchar_end),
                navchar: Some(
                    text[idx_navchar_start..idx_navchar_end]
                        .chars()
                        .next()
                        .expect("char"),
                ),
                right: Cow::Borrowed(right),
                ..Default::default()
            }
        } else {
            unreachable!();
        }
    } else {
        MenuItem {
            item: Cow::Borrowed(text),
            right: Cow::Borrowed(right),
            ..Default::default()
        }
    }
}
