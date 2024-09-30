use crate::_private::NonExhaustive;
use crate::menuitem::{is_separator_str, menu_str, separator_str, MenuItem, Separator};
use ratatui::prelude::Style;

pub mod menubar;
pub mod menuitem;
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

/// Combined styles.
#[derive(Debug, Clone)]
pub struct MenuStyle {
    pub style: Style,
    pub title: Option<Style>,
    pub highlight: Option<Style>,
    pub disabled: Option<Style>,
    pub right: Option<Style>,
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
            disabled: None,
            right: None,
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
#[derive(Debug)]
pub struct StaticMenu {
    /// Array of menus + array of items.
    ///
    /// __MenuItems__
    ///
    /// The first '_' marks the navigation-char.
    ///
    /// __Separator__
    ///
    /// This uses `_` (underscore) as prefix and
    /// a fixed string to identify the separator:
    ///
    /// * `_   ` - three blanks -> empty separator
    /// * `____` - three underscores -> plain line
    /// * `_______` - six underscore -> thick line
    /// * `_===` - three equals -> double line
    /// * `_---` - three hyphen -> dashed line
    /// * `_...` - three dots -> dotted line
    ///
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
                if let Some(last) = submenu.last_mut() {
                    last.sep = separator_str(*s);
                }
            } else {
                submenu.push(menu_str(*s))
            }
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
