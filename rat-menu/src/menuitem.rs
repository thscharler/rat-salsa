//!
//! MenuItem for both MenuLine and PopupMenu.
//!

use crate::_private::NonExhaustive;
use std::borrow::Cow;
use std::ops::Range;
use unicode_display_width::width;

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

/// A menu item.
#[derive(Debug, Clone)]
pub struct MenuItem<'a> {
    /// Menuitem text
    pub item: Cow<'a, str>,
    /// Text range to highlight. This is a byte-range into `item`.
    pub highlight: Option<Range<usize>>,
    /// Navigation key char.
    pub navchar: Option<char>,
    /// Right aligned text. To show the hotkey, or whatever.
    /// Hotkey handling is not included in this crate.
    pub right: Cow<'a, str>,
    /// Disabled item.
    pub disabled: bool,

    /// Separator after the item.
    pub separator: Option<Separator>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for MenuItem<'_> {
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
    /// `\\` (underscore) is used as prefix and then
    /// a fixed string to identify the separator:
    ///
    /// * `\\   ` - three blanks -> empty separator
    /// * `\\___` - three underscores -> plain line
    /// * `\\______` - six underscore -> thick line
    /// * `\\===` - three equals -> double line
    /// * `\\---` - three hyphen -> dashed line
    /// * `\\...` - three dots -> dotted line
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
            item: Cow::Owned(text),
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
            navchar: Some(navchar.to_ascii_lowercase()),
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
            navchar: Some(navchar.to_ascii_lowercase()),
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

    /// Text-width in graphemes for item.
    pub fn item_width(&self) -> u16 {
        width(self.item.as_ref()) as u16 - if self.navchar.is_some() { 1 } else { 0 }
    }

    /// Text-width in graphemes for right.
    pub fn right_width(&self) -> u16 {
        width(self.right.as_ref()) as u16
    }

    /// Text-height.
    pub fn height(&self) -> u16 {
        if self.separator.is_none() { 1 } else { 2 }
    }
}

#[allow(clippy::needless_bool)]
#[allow(clippy::if_same_then_else)]
fn is_separator_str(s: &str) -> bool {
    if s == "\\   " {
        true
    } else if s == "\\___" {
        true
    } else if s == "\\______" {
        true
    } else if s == "\\===" {
        true
    } else if s == "\\---" {
        true
    } else if s == "\\..." {
        true
    } else {
        false
    }
}

/// This uses `\\` (underscore) as prefix and
/// a fixed string to identify the separator:
///
/// * `\\   ` - three blanks -> empty separator
/// * `\\___` - three underscores -> plain line
/// * `\\______` - six underscore -> thick line
/// * `\\===` - three equals -> double line
/// * `\\---` - three hyphen -> dashed line
/// * `\\...` - three dots -> dotted line
fn separator_str(s: &str) -> Separator {
    if s == "\\   " {
        Separator::Empty
    } else if s == "\\___" {
        Separator::Plain
    } else if s == "\\______" {
        Separator::Thick
    } else if s == "\\===" {
        Separator::Double
    } else if s == "\\---" {
        Separator::Dashed
    } else if s == "\\..." {
        Separator::Dotted
    } else {
        unreachable!()
    }
}

/// Create a Line from the given text.
/// The first '_' marks the navigation-char.
/// Pipe '|' separates the item text and the right text.
#[allow(clippy::collapsible_if)]
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
        }
        if c == '|' {
            idx_pipe = Some(idx);
        }
    }
    if idx_navchar_start.is_some() && idx_navchar_end.is_none() {
        idx_navchar_end = Some(txt.len());
    }

    if let Some(pipe) = idx_pipe {
        if let Some(navchar_end) = idx_navchar_end {
            if navchar_end > pipe {
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
                        .expect("char")
                        .to_ascii_lowercase(),
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
