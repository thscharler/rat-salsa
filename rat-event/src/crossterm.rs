//! Support for the [ct_event!](macro.ct_event.html) macro.

#![allow(dead_code)]

pub mod event {
    pub use ratatui_crossterm::crossterm::event::Event;
    pub use ratatui_crossterm::crossterm::event::KeyCode;
    pub use ratatui_crossterm::crossterm::event::KeyEvent;
    pub use ratatui_crossterm::crossterm::event::KeyEventKind;
    pub use ratatui_crossterm::crossterm::event::KeyModifiers;
    pub use ratatui_crossterm::crossterm::event::MouseButton;
    pub use ratatui_crossterm::crossterm::event::MouseEvent;
    pub use ratatui_crossterm::crossterm::event::MouseEventKind;
}

/// A copy of the crossterm-KeyModifiers. Plus a few combinations of modifiers.
pub mod modifiers {
    use crate::crossterm::event::KeyModifiers;

    pub const NONE: KeyModifiers = KeyModifiers::NONE;
    pub const CONTROL: KeyModifiers = KeyModifiers::CONTROL;
    pub const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
    pub const ALT: KeyModifiers = KeyModifiers::ALT;
    pub const META: KeyModifiers = KeyModifiers::META;
    pub const SUPER: KeyModifiers = KeyModifiers::SUPER;
    pub const HYPER: KeyModifiers = KeyModifiers::HYPER;
    pub const CONTROL_ALT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0110);
    pub const CONTROL_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0011);
    pub const ALT_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0101);
}

/// This macro produces pattern matches for crossterm events.
///
/// Example:
/// ```rust ignore
/// match event {
///     ct_event!(keycode press Left) => self.move_to_prev(false),
///     ct_event!(keycode press Right) => self .move_to_next(false),
///     ct_event!(keycode press CONTROL-Left) => {
///         let pos = self.prev_word_boundary();
///         self.set_cursor(pos, false);
///     }
///     ct_event!(keycode press CONTROL_SHIFT-Left) => {
///         let pos = self.prev_word_boundary();
///         self.set_cursor(pos, true);
///     }
///     ct_event!(key press CONTROL-'a') => self.set_selection(0, self.len()),
///     ct_event!(key press c) | ct_event!(key press SHIFT-c) => self.insert_char( * c),
///
///     ct_event!(mouse down Left for column,row) => {
///         // ...
///     }
///     ct_event!(mouse drag Left for column, _row) => {
///         // ...
///     }
///     ct_event!(mouse moved) => {
///         // ...
///     }
/// }
/// ```
///
/// __Caveat__
/// press covers both KeyEventKind::Press and KeyEventKind::Repeat
/// which is probably what you expect. This macro doesn't cover
/// the case where those two need to be distinguished.
///
///
/// Syntax:
/// ```bnf
/// "key" ("press"|"release") (modifier "-")? "'" char "'"
/// "keycode" ("press"|"release") (modifier "-")? keycode
/// "mouse" ("down"|"up"|"drag") (modifier "-")? button "for" col_id "," row_id
/// "mouse" "any" (modifier)? ("for" mouseevt)?
/// "mouse" "moved" ("for" col_id "," row_id)?
/// "scroll" ("up"|"down") "for" col_id "," row_id
/// "resized" "for" cols_id "," rows_id
/// "focus_gained"
/// "focus_lost"
/// "paste" identifier
/// ```
///
/// where
///
/// ```bnf
/// modifier := <<one of the KeyModifiers's>> | "CONTROL_SHIFT" | "ALT_SHIFT"
/// char := <<some character>>
/// keycode := <<one of the defined KeyCode's>>
/// button := <<one of the defined MouseButton's>>
/// ```
///
/// __Caveat__
/// `ct_event!(key press SHIFT-'f')` doesn't work. It must be
/// `ct_event!(key press SHIFT-'F')` with capital F. This holds for
/// any combination of SHIFT too.
///
#[macro_export]
macro_rules! ct_event {
    (key press $keychar:pat) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (key press ANY-$keychar:pat) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::Char($keychar),
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (key press $mod:ident-$keychar:pat) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (key release $keychar:pat) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (key release ANY-$keychar:pat) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::Char($keychar),
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (key release $mod:ident-$keychar:pat) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };

    (keycode press F($code:literal)) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::F($code),
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (keycode press $mod:ident-F($code:literal)) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::F($code),
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (keycode press $code:ident) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (keycode press $mod:ident-$code:ident) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: $crate::crossterm::event::KeyEventKind::Press|$crate::crossterm::event::KeyEventKind::Repeat,
            ..
        })
    };
    (keycode release F($code:literal)) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::F($code),
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (keycode release $mod:ident-F($code:literal)) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::F($code),
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (keycode release $code:ident) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (keycode release $mod:ident-$code:ident) => {
        $crate::crossterm::event::Event::Key($crate::crossterm::event::KeyEvent {
            code: $crate::crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: $crate::crossterm::event::KeyEventKind::Release,
            ..
        })
    };

    (mouse down $button:ident for $col:ident, $row:ident ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Down($crate::crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (mouse down $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Down($crate::crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (mouse up $button:ident for $col:ident, $row:ident ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Up($crate::crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (mouse up $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Up($crate::crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (mouse drag $button:ident for $col:ident, $row:ident ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Drag($crate::crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (mouse drag $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Drag($crate::crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };


    (mouse any for $mouse:ident) => {
        $crate::crossterm::event::Event::Mouse($mouse)
    };
    (mouse any $mod:ident for $mouse:ident ) => {
        $crate::crossterm::event::Event::Mouse($mouse @ $crate::crossterm::event::MouseEvent {
            modifiers: $crate::crossterm::modifiers::$mod,
            ..
        })
    };

    (mouse moved ) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Moved,
            modifiers: $crate::crossterm::modifiers::NONE,
            ..
        })
    };
    (mouse moved for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::Moved,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };

    (scroll $mod:ident down for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollDown,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (scroll down for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollDown,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (scroll down) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollDown,
            modifiers: $crate::crossterm::modifiers::NONE,
            ..
        })
    };
    (scroll $mod:ident up for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollUp,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (scroll up for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollUp,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (scroll up) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollUp,
            modifiers: $crate::crossterm::modifiers::NONE,
            ..
        })
    };

    //??
    (scroll left for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollLeft,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (scroll left) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollLeft,
            modifiers: $crate::crossterm::modifiers::NONE,
            ..
        })
    };
    //??
    (scroll right for $col:ident, $row:ident) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollRight,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (scroll right) => {
        $crate::crossterm::event::Event::Mouse($crate::crossterm::event::MouseEvent {
            kind: $crate::crossterm::event::MouseEventKind::ScrollRight,
            modifiers: $crate::crossterm::modifiers::NONE,
            ..
        })
    };

    (resized) => {
        $crate::crossterm::event::Event::Resize(_, _)
    };
    (resized for $cols:ident, $rows:ident ) => {
        $crate::crossterm::event::Event::Resize($cols, $rows)
    };
    (focus_gained) => {
        $crate::crossterm::event::Event::FocusGained
    };
    (focus_lost) => {
        $crate::crossterm::event::Event::FocusLost
    };
    (paste $value:ident) => {
        $crate::crossterm::event::Event::Paste($value)
    };
}
