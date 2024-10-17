
# Container widgets

Container widgets are just widgets with some inner structure
they want to expose.

For a container widget implement FocusContainer instead of HasFocus.

```rust
pub trait FocusContainer {
    // Required method
    fn build(&self, builder: &mut FocusBuilder);

    // Provided methods
    fn container(&self) -> Option<ContainerFlag> { ... }
    fn area(&self) -> Rect { ... }
    fn is_container_focused(&self) -> bool { ... }
    fn container_lost_focus(&self) -> bool { ... }
    fn container_gained_focus(&self) -> bool { ... }
}
```

* build()
  
  This is called to construct the focus recursively.
  Use FocusBuilder::widget() to add a single widget, or
  FocusBuilder::container() to add a container widget.
  
  That's it.
  
* container()
  
  The container widget may want to know if any of the contained
  widgets has a focus. If container() returns a ContainerFlag
  (which is the same as FocusFlag just a separate type for
  clarity). Focus updates the container flag for focus changes in
  any of the widgets added with build; recursively.
  
  The container-flag is also used to focus the first widget for a
  container with Focus::focus_container().
  
  And the container-flag is used to remove/update/replace the
  widgets of a container.
  
* area()
  
  If area() returns a value than the first widget in the
  container is focused if you click on that area.
  
* is_container_focused(), container_lost_focus(),
  container_gained_focus()
  
  For application code; uses the container flag.
  
