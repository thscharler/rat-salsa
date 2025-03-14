![stable](https://img.shields.io/badge/stability-β--3-850101)
[![crates.io](https://img.shields.io/crates/v/rat-focus.svg)](https://crates.io/crates/rat-focus)
[![Documentation](https://docs.rs/rat-focus/badge.svg)](https://docs.rs/rat-focus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-focus GitHub][refGithubFocus].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-focus/changes.md)

# Focus handling for ratatui

This crate works by adding a [FocusFlag](FocusFlag) to each widget'
s state.

[FocusBuilder](FocusBuilder) then is used to collect an ordered list of
all widgets that should be considered for focus handling.
It builds up the [Focus](Focus) which has the functions [next](Focus::next),
[prev](Focus::prev) and [focus_at](Focus::focus_at) that can do
the navigation.

> from <focus_input1.rs>

```rust ignore
fn focus_input(state: &mut State) -> Focus {
    let mut fb = FocusBuilder::default();
    fb.widget(&state.input1);
    fb.widget(&state.input2);
    fb.widget(&state.input3);
    fb.widget(&state.input4);
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

The widget can use its FocusFlag to decide the kind of event-handling
it should do. There is usually a big difference between focused and
not focused behaviour.

If you are using some third party widget you can add a FocusFlag
somewhere to your state and use that to control the third party
widget. Or you may want to write a wrapper.

## Traits and Widgets

# HasFocus

[HasFocus] is the main interface for focus.

## Widgets

Simple widgets implement at least the first three of these functions.

- build() - For simple widgets with no internal structure this
  adds a leaf to the widget list.
  ```rust ignore
  fn build(&self, builder: &mut FocusBuilder) {
      builder.leaf_widget(self);
  }
  ```

- focus() - Return a clone of FocusFlag. There is a Rc inside,
  so this is a cheap clone.
- area() - Rendered area. This is used for mouse interaction.
- area_z() - When there are overlapping areas an extra z-value
  can be provided to find the top widget.
- navigable() - A control flag indicating __how__ the widget interacts
  with focus.

## Widgets as Containers

When a widget contains other widgets it also implements HasFocus.

The primary function here is

- build() - This is called with the FocusBuilder and
  can add the separate component widgets of the container.

  You can have a FocusFlag marking the whole container.
  Such a FocusFlag collects the status of each component widget.
  That means the FocusFlag of the container 'is_focused' when any
  of the components 'is_focused'.

  If you manually call Focus::focus() for a container, the first
  component widget will get the focus. Similarly, if you click anywhere
  in the provided area the first component widget will get the focus.

```rust ignore
impl HasFocus for FooWidget {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        builder.widget(&self.component_a);
        builder.widget(&self.component_b);
        builder.end(tag);
    }
}
```

This will use focus(), area() and area_z() to define the container.

If you don't need this, just leave it out

```rust ignore
impl HasFocus for FooWidget {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.component_a);
        builder.widget(&self.component_b);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not in use");
    }

    fn area(&self) -> Rect {
        unimplemented!("not in use");
    }
}
```

This will just add more widgets to the focus. The focus() and area() functions
are still technically necessary, but are not used.


[refHandleEvent]: https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refGithubFocus]: https://github.com/thscharler/rat-salsa/blob/master/rat-focus/examples