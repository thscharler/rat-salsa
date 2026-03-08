# Documentation details.

## Event handling

Event handling is implemented for crossterm. Focus implements
[HandleEvent][refHandleEvent], and there is also the function
[handle_focus](handle_focus) to invoke it.

It uses Tab/BackTab for navigation and handles mouse clicks on the widget's area.

```rust ignore
    // Handle events for focus.
let f = create_focus(state).handle(event, Regular);
```

It returns `Outcome::Changed` whenever something interesting has happened.
An `Outcome::Changed` should do a repaint, most widgets show their focus-state
somehow.

### Changed focus

When the focused widget changes, the newly focused widget has its
focus_gained flag set, and the previous widget its focus_lost flag.

A widget can also set callbacks for focus_gained and focus_lost
if it needs to react to changes.

## Mouse focus

Besides the (input) focus, there is also a mouse-focus flag.

This flag is set, if the event is a mouse-event, and any of
the widget-areas is hit by the mouse-event. This takes account
for widgets with overlapping areas, it also uses the z-index
of the widget-areas which widget should be concerned with
the mouse-event.

### Widget event handling

The widget can use its FocusFlag to decide the kind of event-handling
it should do. There is usually a big difference between focused and
not focused behavior.

If you are using some third party widget you can add a FocusFlag
somewhere to your state and use that to control the third party
widget. Or you may want to write a wrapper.

# Trait HasFocus

[HasFocus] is the main interface for widgets.

## Widgets

Simple widgets implement at least the first three functions of `HasFocus`.

- build() - For simple widgets with no internal structure this
  adds a leaf to the widget list.
  ```rust ignore
  fn build(&self, builder: &mut FocusBuilder) {
      builder.leaf_widget(self);
  }
  ```

- focus() - Return a clone of FocusFlag. There is a Rc inside, so this is a cheap clone.
- area() - Rendered area. This is used for mouse interaction.
  A widget can register multiple areas (e.g. ComboBox) too.
- area_z() - If a widget has an area that is logically `above` other widgets,
  it can use a z-value to indicate this. When testing for mouse interactions,
  areas with a higher z-value take priority.
  If areas with the same z-value overlap, the last one wins.
- navigable() - A control flag indicating __how__ the widget interacts
  with focus.

## Container widgets

When a widget contains other widgets it also implements HasFocus.

The primary function here is

- build() - This is called with the FocusBuilder and
  can add the separate component widgets of the container.

  You can have a FocusFlag marking the whole container.
  Such a FocusFlag sums the status of each component widget.
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

## Z-index for containers

If a container-widget sets a z-index, all the widgets added to the
container will use this z-index as a base. This ensures that mouse-interactions
with a complex popup work out.

## Mouse focus for containers.

The container mouse-focus will be set if the container itself registers
an area for mouse interactions. This differs from the input-focus, where
the container-flag is set if any of the widgets is focused.

If you stack containers, the mouse-focus will be set for all containers
that pass the hit-test. But; if you use a z-index for a container, none
of the containers with a lower z-index will have the flag set.

[refHandleEvent]: https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html
