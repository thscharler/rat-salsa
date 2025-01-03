# Widget focus

For a widget to work with Focus it must implement HasFocus.

```rust
pub trait HasFocusFlag {
    // Required methods
    fn focus(&self) -> FocusFlag;
    fn area(&self) -> Rect;

    // Provided methods
    fn area_z(&self) -> u16 { ... }
    fn navigable(&self) -> Navigation { ... }
    fn is_focused(&self) -> bool { ... }
    fn lost_focus(&self) -> bool { ... }
    fn gained_focus(&self) -> bool { ... }

    // 
    fn build(&self, builder: &mut FocusBuilder) { ... }
}
```

* focus()

The widget state should contain a FocusFlag somewhere. It returns a
clone here. The current state of the widget is always accessible
during rendering and event-handling.

* area()

Area for mouse focus.

* area_z()

The z-value for the area. When you add overlapping areas the
z-value is used to find out which area should be focused by
a given mouse event.

* navigable()

  This indicates if/how the widget can be reached/left by Focus.
  It has a lot of Options, see [Navigation][refNavigation].

* is_focused(), lost_focus(), gained_focus()

  These are for application code.

* build()

  Like FocusContainer there is a build method. For most widgets
  the default implementation will suffice.

  But if you have a complex widget with inner structures,
  you can implement this to set up your focus requirements.

[refNavigation]: https://docs.rs/rat-focus/latest/rat_focus/enum.Navigation.html

    
