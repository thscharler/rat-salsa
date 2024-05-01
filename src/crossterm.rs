#![allow(dead_code)]

/// A copy of the crossterm-KeyModifiers. Plus a few combinations of modifiers.
pub(crate) mod modifiers {
    use crossterm::event::KeyModifiers;

    pub(crate) const NONE: KeyModifiers = KeyModifiers::NONE;
    pub(crate) const CONTROL: KeyModifiers = KeyModifiers::CONTROL;
    pub(crate) const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
    pub(crate) const ALT: KeyModifiers = KeyModifiers::ALT;
    pub(crate) const META: KeyModifiers = KeyModifiers::META;
    pub(crate) const SUPER: KeyModifiers = KeyModifiers::SUPER;
    pub(crate) const HYPER: KeyModifiers = KeyModifiers::HYPER;
    pub(crate) const CONTROL_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0011);
    pub(crate) const ALT_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0101);
}

/// This macro produces pattern matches for crossterm events.
///
/// Syntax:
/// ```bnf
/// "key" ("press"|"release") (modifier "-")? "'" char "'"
/// "keycode" ("press"|"release") (modifier "-")? keycode
/// "mouse" ("down"|"up"|"drag") (modifier "-")? button "for" col_id "," row_id
/// "mouse" "moved" ("for" col_id "," row_id)?
/// "scroll" ("up"|"down") "for" col_id "," row_id
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
#[macro_export]
macro_rules! ct_event {
    (key press $keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (key press $mod:ident-$keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (key release $keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (key release $mod:ident-$keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };

    (keycode press $code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (keycode press $mod:ident-$code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (keycode release $code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (keycode release $mod:ident-$code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::crossterm::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };

    (mouse down $button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (mouse down $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (mouse up $button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (mouse up $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (mouse drag $button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (mouse drag $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };

    (mouse moved ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Moved,
            modifiers: $crate::crossterm::modifiers::NONE,
            ..
        })
    };
    (mouse moved for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Moved,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };

    (scroll $mod:ident down for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollDown,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (scroll down for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollDown,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    (scroll $mod:ident up for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollUp,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::$mod,
        })
    };
    (scroll up for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollUp,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };

    //??
    (scroll left for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollLeft,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
    //??
    (scroll right for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollRight,
            column: $col,
            row: $row,
            modifiers: $crate::crossterm::modifiers::NONE,
        })
    };
}
