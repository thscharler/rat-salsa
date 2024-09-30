use crate::_private::NonExhaustive;
use ratatui::prelude::Style;
use std::ops::Range;

mod item;
pub mod menubar;
pub mod menuline;
pub mod popup_menu;
mod util;

pub mod event {
    //!
    //! Event-handler traits and Keybindings.
    //!
    pub use rat_event::*;

    /// Outcome for menuline.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MenuOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
        /// The menuitem was selected.
        Selected(usize),
        /// The menuitem was selected and activated.
        Activated(usize),
        /// Selected popup-menu.
        MenuSelected(usize, usize),
        /// Activated popup-menu.
        MenuActivated(usize, usize),
    }

    impl ConsumedEvent for MenuOutcome {
        fn is_consumed(&self) -> bool {
            *self != MenuOutcome::Continue
        }
    }

    impl From<MenuOutcome> for Outcome {
        fn from(value: MenuOutcome) -> Self {
            match value {
                MenuOutcome::Continue => Outcome::Continue,
                MenuOutcome::Unchanged => Outcome::Unchanged,
                MenuOutcome::Changed => Outcome::Changed,
                MenuOutcome::Selected(_) => Outcome::Changed,
                MenuOutcome::Activated(_) => Outcome::Changed,
                MenuOutcome::MenuSelected(_, _) => Outcome::Changed,
                MenuOutcome::MenuActivated(_, _) => Outcome::Changed,
            }
        }
    }
}

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

/// Menu-Item
#[derive(Debug, Clone)]
pub enum MenuItem<'a> {
    /// Menu item
    Item1(&'a str),
    /// Menu item, byte-idx of key, char
    Item2(&'a str, Range<usize>, char),
    /// Menu item, byte-idx of key, char, hot key
    Item3(&'a str, Range<usize>, char, &'a str),
    /// Menu separator
    Sep(Separator),
}

/// Combined styles.
#[derive(Debug, Clone)]
pub struct MenuStyle {
    pub style: Style,
    pub title: Option<Style>,
    pub highlight: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub non_exhaustive: NonExhaustive,
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            title: None,
            highlight: None,
            select: None,
            focus: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Trait for the structural data of the MenuBar.
pub trait MenuStructure<'a> {
    /// Main menu.
    fn menus(&'a self, menu: &mut Vec<MenuItem<'a>>);
    /// Submenus.
    fn submenu(&'a self, n: usize, submenu: &mut Vec<MenuItem<'a>>);
}

/// Static menu structure.
///
///
#[derive(Debug)]
pub struct StaticMenu {
    pub menu: &'static [(&'static str, &'static [&'static str])],
}

impl MenuStructure<'static> for StaticMenu {
    fn menus(&'static self, menu: &mut Vec<MenuItem<'static>>) {
        for (s, _) in self.menu.iter() {
            menu.push(menu_str(*s))
        }
    }

    fn submenu(&'static self, n: usize, submenu: &mut Vec<MenuItem<'static>>) {
        for s in self.menu[n].1 {
            if is_separator_str(*s) {
                submenu.push(separator_str(*s))
            } else {
                submenu.push(menu_str(*s))
            }
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
pub fn separator_str(s: &str) -> MenuItem<'_> {
    if s == "_   " {
        MenuItem::Sep(Separator::Empty)
    } else if s == "____" {
        MenuItem::Sep(Separator::Plain)
    } else if s == "_______" {
        MenuItem::Sep(Separator::Thick)
    } else if s == "_===" {
        MenuItem::Sep(Separator::Double)
    } else if s == "_---" {
        MenuItem::Sep(Separator::Dashed)
    } else if s == "_..." {
        MenuItem::Sep(Separator::Dotted)
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
            MenuItem::Item2(
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
        MenuItem::Item1(txt)
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
