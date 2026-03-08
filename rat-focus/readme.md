![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-focus.svg)](https://crates.io/crates/rat-focus)
[![Documentation](https://docs.rs/rat-focus/badge.svg)](https://docs.rs/rat-focus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-focus GitHub][refGithubFocus].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-focus/changes.md)

# Focus handling for ratatui

This crate works by adding a [FocusFlag](FocusFlag) to each widget's
state.

Then [FocusBuilder](FocusBuilder) is used to collect an ordered
list of all widgets that should be considered for focus handling.
It builds the [Focus](Focus) which has the functions [next](Focus::next),
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
    - No need to update the widget list when the application state changes.
    - FocusBuilder can be passed on all over the application to build the 
      widget tree.
 
[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refGithubFocus]: https://github.com/thscharler/rat-salsa/blob/master/rat-focus/examples