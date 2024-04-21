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

#[macro_export]
macro_rules! ct_event {
    (key-char press $mod:ident $code:ident ) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($code),
            modifiers: KeyModifiers::$mod,
            kind: KeyEventKind::Press,
            ..
        })
    };
    (key-char press $mod:ident $code:literal ) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($code),
            modifiers: KeyModifiers::$mod,
            kind: KeyEventKind::Press,
            ..
        })
    };
    (key-code press $mod:ident $code:ident ) => {
        Event::Key(KeyEvent {
            code: KeyCode::$code,
            modifiers: KeyModifiers::$mod,
            kind: KeyEventKind::Press,
            ..
        })
    };

    (mouse-down $mod:ident $button:ident for $col:ident, $row:ident ) => {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: KeyModifiers::$mod,
        })
    };
    (mouse-up $mod:ident $button:ident for $col:ident, $row:ident ) => {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: KeyModifiers::$mod,
        })
    };
    (mouse-drag $mod:ident $button:ident for $col:ident, $row:ident ) => {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::$button),
            column: $col,
            row: $row,
            modifiers: KeyModifiers::$mod,
        })
    };
    (mouse-moved ) => {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            modifiers: KeyModifiers::NONE,
            ..
        })
    };
    (mouse-moved for $col:ident, $row:ident) => {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: $col,
            row: $row,
            modifiers: KeyModifiers::NONE,
        })
    };
}
