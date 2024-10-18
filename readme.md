[![crates.io](https://img.shields.io/crates/v/rat-focus.svg)](https://crates.io/crates/rat-focus)
[![Documentation](https://docs.rs/rat-focus/badge.svg)](https://docs.rs/rat-focus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-focus)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-focus GitHub][refGithubFocus].

* [Changes](https://github.com/thscharler/rat-focus/blob/master/changes.md)

# Focus handling for ratatui

This crate works by adding a [FocusFlag](FocusFlag) to each widget'
s state.

[FocusBuilder](FocusBuilder) then is used to collect an ordered list of
all widgets that should be considered for focus handling.
It builds up the [Focus](Focus) which has [next](Focus::next),
[prev](Focus::prev) and [focus_at](Focus::focus_at) which can do
the navigation.

> from <focus_input1.rs>

```rust ignore
    fn focus_input(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.input1)
        .widget(&state.input2)
        .widget(&state.input3)
        .widget(&state.input4);
    fb.build()
}

fn handle_input(
    event: &crossterm::event::Event,
    _data: &mut Data,
    _istate: &mut MiniSalsaState,
    state: &mut State,
) -> Result<Outcome, anyhow::Error> {

    // Handle events for focus.
    let f = focus_input(state).handle(event, Regular);

    // ...

    Ok(f)
}
```

- Keeps the focus-state close to the widgets code.
- Rebuilt for each event.
    - No need to update the widget list when the application state
      changes.
    - FocusBuilder can be passed on all over the application to
      build the current widget list.

## Event handling

### React to focus events

Event handling is implemented for crossterm. It uses Tab+BackTab
for navigation and handles mouse clicks on the widget's area.

Focus implements [HandleEvent][refHandleEvent], and there is the
fn [handle_focus](handle_focus) to invoke it.

```rust ignore
    // Handle events for focus.
let f = focus_input(state).handle(event, Regular);
```

It returns `Outcome::Changed` whenever something interesting
has happened.

### Widget events

If the widgets has it's own FocusFlag, it will decide the
appropriate event handling using this state. No external control
needed.

If it doesn't you can use a [FocusAdapter] to keep track of the
focus. Use that state to call the appropriate functions defined
by the widget.

## Traits and Widgets

# HasFocus

[HasFocus] is the interface for single widgets.

It is implemented for the widget state struct and provides access
to

- focus()     - FocusFlag
- area()      - Rendered area.
- z_areas()   - Extended area info.
- navigable() - Control flag.

The widget can then use the FocusFlag for rendering and
event-handling as it sees fit.

# FocusContainer

[FocusContainer] is the interface for container widgets.

This is used to recursively add widgets for focus handling.

- build()     - Uses the FocusBuilder to construct the current
  widget structure.
- container() - Optional. Similar to FocusFlag, identifies the
  container and collects the states of the contained widgets.
- area()      - Optional. Area for the whole container. Used for mouse
  events too.

[refHandleEvent]: https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refGithubFocus]: https://github.com/thscharler/rat-focus/tree/master/examples 