
# Focus

The struct [Focus][refFocus] can do all the focus handling for
your application.

As it is essential for almost any application, it got a place
in AppContext.

## Usage

```rust
    if self.w_split.is_focused() {
        ctx.focus().next();
    } else {
        ctx.focus().focus(&self.w_split);
    }
```

Just some example: This queries some widget state whether it
currently has the focus and jumps to the next widget /sets the
focus to the same widget.

## There's always a trait

or two.

* [HasFocus][refHasFocus]:
  
  This trait is for single widgets.
  
  It's main functions are focus() and area().
  
  focus() returns a clone of a [FocusFlag][refFocusFlag] that
  is part of the widgets state. It has a hidden `Rc<>`, so this
  is fine.
  
  > The flag is close to the widget, so it's always there when
  > you need it. As an Rc it can be used elsewhere too, say
  > Focus.
  
  area() returns the widgets current screen area. Which is used
  for mouse focus.
  
* [FocusContainer][refFocusContainer]
  
  The second trait is for container widgets.
  
  It's main function is build().
  
  `build(&mut FocusBuilder)` gets a
  [FocusBuilder][refFocusBuilder] and collects all widgets and
  nested containers in the preferred focus order.
  
## AppState

In your application you construct the current Focus for each
event.

This is necessary as
- the application state might have changed
- the terminal might have been resized

and

- it's hard to track such changes at the point where they occur.
- it's cheap enough not to bother.
- there is room for optimizations later.

```rust
    ctx.focus = Some(FocusBuilder::for_container(&self.app));
```

If you have a AppWidget that `HasFocus` you can simply use
FocusBuilder to construct the current Focus. If you then set it
in the `ctx` it is immediately accessible everywhere.

## Events

Focus implements HandleEvent, so event handling is simple.

```rust
    let f = Control::from(
        ctx.focus_mut().handle(event, Regular)
    );
```

`Regular` event-handling for focus is

- Tab: jump to the next widget.
- Shift-Tab: jump to the previous widget.
- Mouse click: focus that widget.

Focus is independent from rat-salsa, so it returns Outcome
instead of Control, thus the conversion.

> _Complications_
> 
> `handle` returns Outcome::Changed when the focus switches to a
> new widget and everything has to be rendered. On the other hand
> the focused widget might want to use the same mouse click that
> switched the focus to do something else.
> 
> We end up with two results we need to return from the event
> handler.

```rust
    let f = Control::from(ctx.focus_mut().handle(event, Regular));
    let r = self.app.crossterm(event, ctx)?;
```

> Here `Ord` comes to the rescue. The values of Control are
> constructed in order of importance, so

```rust
    Ok(max(f, r))
```

> can save the day. If focus requires Control::Changed we return
> this as the minimum regardless of what the rest of event
> handling says.

Or you can just return a second result to the event-loop using

```rust
    let f = ctx.focus_mut().handle(event, Regular);
    ctx.queue(f);
```    
and be done with it. 


[refFocusContainer]: https://docs.rs/rat-focus/latest/rat_focus/trait.FocusContainer.html

[refHasFocus]: https://docs.rs/rat-focus/latest/rat_focus/trait.HasFocus.html

[refFocusFlag]: https://docs.rs/rat-focus/latest/rat_focus/struct.FocusFlag.html

[refFocusBuilder]: https://docs.rs/rat-focus/latest/rat_focus/struct.FocusBuilder.html

[refFocus]: https://docs.rs/rat-focus/latest/rat_focus/struct.Focus.html




