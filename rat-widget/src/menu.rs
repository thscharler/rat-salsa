//! Menu widgets.
//! See also [rat-menu](https://docs.rs/rat-menu/latest/rat_menu/)

pub use rat_menu::menubar::{Menubar, MenubarLine, MenubarPopup, MenubarState};
pub use rat_menu::menuitem::{MenuItem, Separator};
pub use rat_menu::menuline::{MenuLine, MenuLineState};
pub use rat_menu::popup_menu::{PopupConstraint, PopupMenu, PopupMenuState};
pub use rat_menu::{MenuBuilder, MenuStructure, MenuStyle, StaticMenu};

pub mod menubar {
    pub use rat_menu::menubar::{handle_events, handle_mouse_events, handle_popup_events};
}
pub mod menuline {
    pub use rat_menu::menuline::{handle_events, handle_mouse_events};
}
pub mod popup_menu {
    pub use rat_menu::popup_menu::{handle_events, handle_mouse_events, handle_popup_events};
}
