#![doc = include_str!("../readme.md")]

use crate::_private::NonExhaustive;
use crate::menuitem::{MenuItem, Separator};
use rat_popup::PopupStyle;
use ratatui::style::Style;
use ratatui::widgets::Block;
use std::fmt::Debug;
use std::ops::Range;

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
    use rat_popup::event::PopupOutcome;

    /// Outcome for menuline.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum MenuOutcome {
        /// The given event was not handled at all.
        Continue,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,

        /// Popup should be hidden.
        ///
        /// Used by PopupMenu.
        Hide,

        /// A menuitem was select.
        ///
        /// Used by MenuLine and PopupMenu.
        /// Used by Menubar for results from the main menu.
        Selected(usize),

        /// A menuitem was activated.
        ///
        /// Used by MenuLine and PopupMenu.
        /// Used by Menubar for results from the main menu.
        Activated(usize),

        /// A popup-menuitem was selected.
        ///
        /// Used by Menubar for results from a popup-menu. Is (main-idx, popup-idx).
        MenuSelected(usize, usize),

        /// A popup-menuitem was activated.
        ///
        /// Used by Menubar for results from a popup-menu. Is (main-idx, popup-idx);
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
                MenuOutcome::Hide => Outcome::Changed,
            }
        }
    }

    impl From<PopupOutcome> for MenuOutcome {
        fn from(value: PopupOutcome) -> Self {
            match value {
                PopupOutcome::Continue => MenuOutcome::Continue,
                PopupOutcome::Unchanged => MenuOutcome::Unchanged,
                PopupOutcome::Changed => MenuOutcome::Changed,
                PopupOutcome::Hide => MenuOutcome::Hide,
            }
        }
    }

    impl From<Outcome> for MenuOutcome {
        fn from(value: Outcome) -> Self {
            match value {
                Outcome::Continue => MenuOutcome::Continue,
                Outcome::Unchanged => MenuOutcome::Unchanged,
                Outcome::Changed => MenuOutcome::Changed,
            }
        }
    }
}

/// Combined styles.
#[derive(Debug, Clone)]
pub struct MenuStyle {
    /// Base style.
    pub style: Style,
    /// Menuline title style.
    pub title: Option<Style>,
    /// Style for the _ highlight/nav-char
    pub highlight: Option<Style>,
    /// Style for a disabled item.
    pub disabled: Option<Style>,
    /// Style for the hotkey
    pub right: Option<Style>,
    /// Focus style
    pub focus: Option<Style>,

    /// Styling for the popup menus.
    pub popup_style: Option<Style>,
    /// Block for the popup menus.
    pub block: Option<Block<'static>>,
    /// Popup itself
    pub popup: PopupStyle,
    /// Border style
    pub popup_border: Option<Style>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            title: Default::default(),
            highlight: Default::default(),
            disabled: Default::default(),
            right: Default::default(),
            focus: Default::default(),
            popup_style: Default::default(),
            block: Default::default(),
            popup: Default::default(),
            popup_border: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

/// Trait for the structural data of the MenuBar.
pub trait MenuStructure<'a>: Debug {
    /// Main menu.
    fn menus(&'a self, menu: &mut MenuBuilder<'a>);
    /// Submenus.
    fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>);
}

/// Builder to fill a menu with items.
#[derive(Debug, Default, Clone)]
pub struct MenuBuilder<'a> {
    pub(crate) items: Vec<MenuItem<'a>>,
}

impl<'a> MenuBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a menu-item.
    pub fn item(&mut self, item: MenuItem<'a>) -> &mut Self {
        self.items.push(item);
        self
    }

    /// Parse the text.
    ///
    /// __See__
    ///
    /// [MenuItem::new_parsed]
    pub fn item_parsed(&mut self, text: &'a str) -> &mut Self {
        let item = MenuItem::new_parsed(text);
        if let Some(separator) = item.separator {
            if let Some(last) = self.items.last_mut() {
                last.separator = Some(separator);
            } else {
                self.items.push(item);
            }
        } else {
            self.items.push(item);
        }
        self
    }

    /// New item.
    pub fn item_str(&mut self, text: &'a str) -> &mut Self {
        self.items.push(MenuItem::new_str(text));
        self
    }

    /// New item with owned text.
    pub fn item_string(&mut self, text: String) -> &mut Self {
        self.items.push(MenuItem::new_string(text));
        self
    }

    /// New item with navigation.
    pub fn item_nav_str(
        &mut self,
        text: &'a str,
        highlight: Range<usize>,
        navchar: char,
    ) -> &mut Self {
        self.items
            .push(MenuItem::new_nav_str(text, highlight, navchar));
        self
    }

    /// New item with navigation.
    pub fn item_nav_string(
        &mut self,
        text: String,
        highlight: Range<usize>,
        navchar: char,
    ) -> &mut Self {
        self.items
            .push(MenuItem::new_nav_string(text, highlight, navchar));
        self
    }

    /// Sets the separator for the last item added.
    /// If there is none adds this as an empty menu-item.
    pub fn separator(&mut self, separator: Separator) -> &mut Self {
        if let Some(last) = self.items.last_mut() {
            last.separator = Some(separator);
        } else {
            self.items.push(MenuItem::new().separator(separator));
        }
        self
    }

    /// Sets the last item to disabled.
    /// If there is no last item does nothing.
    pub fn disabled(&mut self, disable: bool) -> &mut Self {
        if let Some(last) = self.items.last_mut() {
            last.disabled = disable;
        }
        self
    }

    /// Build and deconstruct.
    pub fn items(self) -> Vec<MenuItem<'a>> {
        self.items
    }
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
    fn menus(&'static self, menu: &mut MenuBuilder<'static>) {
        for (s, _) in self.menu.iter() {
            menu.item_parsed(s);
        }
    }

    fn submenu(&'static self, n: usize, submenu: &mut MenuBuilder<'static>) {
        for s in self.menu[n].1 {
            submenu.item_parsed(s);
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
