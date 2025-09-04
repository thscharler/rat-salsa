# Container widgets

Container widgets are just widgets with some inner structure
they want to expose.

They, too, implement HasFocus, but their main function is build()
instead of just defining the focus()-flag and area().

## With identity

The container widget can have a FocusFlag of its own.

```rust ignore
impl HasFocus for FooWidget {
    fn build(&self, builder: &mut FocusBuilder) {
        let tag = builder.start(self);
        builder.widget(&self.component_a);
        builder.widget(&self.component_b);
        builder.end(tag);
    }

    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }
}
```

If it does so,

- focusing the container sets the focus to the first widget in the container.
- mouse-click in the area does the same.
- the container-flag will be a summary of the component flags.
  If any component has the focus, the container will have its focus-flag set too.

- Other functions of Focus will differentiate between Widgets and Containers too.

Containers can be used to update the Focus structure after
creation. There are Focus::update_container(), remove_container()
and replace_container() that take containers and change the
internal structure. As the Focus is rebuilt regularly this is
rarely needed.

There can be state changes that will change Focus the next time
it is rebuilt. But with the same state change you already want
to act upon the new future structure. e.g. when changing the
selected tab. Focus on tabs is created for the visible tab only,
and with the tab change the focus should be transferred to the
first widget on the newly visible tab.

## Anonymous

A container widget can be just a bunch of components.

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

This just adds the widgets to the overall focus. focus(), area() and area_z()
will not be used. navigable() is not used for containers anyway.
 
