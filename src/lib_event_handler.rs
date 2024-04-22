/// Marker struct. Used by [HandleCrossterm] to differentiate between key-mappings.
#[derive(Debug)]
pub struct DefaultKeys;

/// Marker struct like [DefaultKeys]. This one selects an event-handler that processes only
/// mouse events. Useful when creating your own key-bindings but not wanting to touch
/// the mouse interactions. If this separation exists for a widget it should be called
/// automatically by the [DefaultKeys] handler.
#[derive(Debug)]
pub struct MouseOnly;

/// Handle events received by crossterm.
///
/// This one should be implemented for the state struct of a widget and can do whatever. And it
/// can return whatever extra outcome is needed. Common usage would return a
/// [ControlUI](crate::ControlUI) flag as a result.
///
/// There is an extra parameter `KeyMap` which can be used to define more than one mapping for
/// a widget. This can be useful when overriding the default behaviour for a widget. Two
/// keymaps for common usage are defined in this library: [DefaultKeys] and [MouseOnly].
///
/// ```rust ignore
///     check_break!(uistate.page1.table1.handle(evt, DefaultKeys));
/// ```
///
/// _Remark_
///
/// There is only HandleCrossterm for now, as that is what I needed. But there is no problem
/// adding a HandleTermion, HandleTermwiz or whatever. One could add a second type parameter
/// for this differentiation, but I think that would complicate usage unnecessarily. And
/// any application will probably decide to use one or the other and not all of them.
/// A widget library can easily support all of them with this scheme without some added layer
/// of indirection and use a feature flag to select between them.
pub trait HandleCrossterm<R, KeyMap = DefaultKeys> {
    fn handle(&mut self, event: &crossterm::event::Event, keymap: KeyMap) -> R;
}

/// A copy of the crossterm-KeyModifiers. Plus a few combinations of modifiers.
pub mod modifiers {
    use crossterm::event::KeyModifiers;

    pub const NONE: KeyModifiers = KeyModifiers::NONE;
    pub const CONTROL: KeyModifiers = KeyModifiers::CONTROL;
    pub const SHIFT: KeyModifiers = KeyModifiers::SHIFT;
    pub const ALT: KeyModifiers = KeyModifiers::ALT;
    pub const META: KeyModifiers = KeyModifiers::META;
    pub const SUPER: KeyModifiers = KeyModifiers::SUPER;
    pub const HYPER: KeyModifiers = KeyModifiers::HYPER;
    pub const CONTROL_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0011);
    pub const ALT_SHIFT: KeyModifiers = KeyModifiers::from_bits_truncate(0b0000_0101);
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
            modifiers: $crate::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (key press $mod:ident-$keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (key release $keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (key release $mod:ident-$keychar:pat) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($keychar),
            modifiers: $crate::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };

    (keycode press $code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (keycode press $mod:ident-$code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    };
    (keycode release $code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::modifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };
    (keycode release $mod:ident-$code:ident) => {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$code,
            modifiers: $crate::modifiers::$mod,
            kind: crossterm::event::KeyEventKind::Release,
            ..
        })
    };

    (mouse down $button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };
    (mouse down $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::$mod,
        })
    };
    (mouse up $button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };
    (mouse up $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::$mod,
        })
    };
    (mouse drag $button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };
    (mouse drag $mod:ident-$button:ident for $col:ident, $row:ident ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::$mod,
        })
    };

    (mouse moved ) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Moved,
            modifiers: $crate::modifiers::NONE,
            ..
        })
    };
    (mouse moved for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Moved,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };

    (scroll $mod:ident down for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollDown,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::$mod,
        })
    };
    (scroll down for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollDown,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };
    (scroll $mod:ident up for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollUp,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::$mod,
        })
    };
    (scroll up for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollUp,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };

    //??
    (scroll left for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollLeft,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };
    //??
    (scroll right for $col:ident, $row:ident) => {
        crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::ScrollRight,
            column: $col,
            row: $row,
            modifiers: $crate::modifiers::NONE,
        })
    };
}
