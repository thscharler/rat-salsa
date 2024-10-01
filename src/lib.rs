use crate::_private::NonExhaustive;
use crate::menuitem::{MenuItem, Separator};
use ratatui::prelude::Style;
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
    fn menus(&'a self, menu: &mut MenuBuilder<'a>);
    /// Submenus.
    fn submenu(&'a self, n: usize, submenu: &mut MenuBuilder<'a>);
}

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
            menu.item_parsed(*s);
        }
    }

    fn submenu(&'static self, n: usize, submenu: &mut MenuBuilder<'static>) {
        for s in self.menu[n].1 {
            submenu.item_parsed(*s);
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
