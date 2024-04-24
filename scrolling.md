I would suggest using a container widget that manages the scrollbars and
can implement common behaviour.

It would use the following two traits, one for the widget and one for the state.

```rust 
/// Trait for a widget that can scroll.
pub trait ScrolledWidget: StatefulWidget {
    /// Get the scrolling behaviour of the widget.
    ///
    /// The area is the area for the scroll widget minus any block set on the [Scrolled] widget.
    /// It doesn't account for the scroll-bars.
    fn need_scroll(&self, area: Rect, state: &mut Self::State) -> ScrollParam;
}
```

This trait is called before rendering the widget itself.

This allows the scrolling-container to calculate the exact layout.
At least tui_tree_widget needs information from the state to calculate a height,
so that's a parameter too.

The next trait is for the state:

```rust 
pub trait HasScrolling {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn max_v_offset(&self) -> usize;

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn max_h_offset(&self) -> usize;

    /// Vertical page-size at the current offset.
    fn v_page_len(&self) -> usize;

    /// Horizontal page-size at the current offset.
    fn h_page_len(&self) -> usize;

    /// Current vertical offset.
    fn v_offset(&self) -> usize;

    /// Current horizontal offset.
    fn h_offset(&self) -> usize;

    /// Change the vertical offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    fn set_v_offset(&mut self, offset: usize);

    /// Change the horizontal offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    fn set_h_offset(&mut self, offset: usize);
}
```

The scrolling widget imposes no unit on the usize values used in this trait.
Those could be items, lines or something else. The interpretation lies solely
with the widget.

But this means that the scrolling widget can't make any connection between
screen space and the values used in this interface.

### Page_len

That's why it gets the current page_len via the trait.
With the page_len known any fractional scrolling is easy.

### Scrollbar

The values used for the scrollbar are offset + max_offset.
Where max_offset is the maximum allowed offset that ensures that a full page
can be displayed. This should solve the complaints about overscrolling.

Both page_size and max_offset are values that can easily be calculated while
rendering, or are a rather small extra burden.

Letting the widget do all these calculations gives the widgets enough freedom
to do whatever they need to do.

### Setting the offset

With the current offset, page_len and max_offset known all the wanted
behaviour can be done.

### Drawback

The one drawback I saw, was that currently there is a strong link between
the selected item and scrolling. The table blocks scrolling, if the
selected item would go out of view.

This is a bit annoying and would require one more switch to turn this off.
Some function like scroll_selected_to_visible() would be nice. It
could be invoked when navigating with the keyboard.

## Example

I tried this, and here are the docs:

[ScrolledWidget](https://thscharler.github.io/rat-salsa/doc/rat_salsa/trait.ScrolledWidget.html)
[HasScrolling](https://thscharler.github.io/rat-salsa/doc/rat_salsa/trait.HasScrolling.html)
the widget: [Scrolled](https://thscharler.github.io/rat-salsa/doc/rat_salsa/widget/scrolled/struct.Scrolled.html)

I wrote adapters for List, Table, Paragraph and just to see if it works
for tui_textarea and tui_tree_widget too.

[widgets](https://thscharler.github.io/rat-salsa/doc/rat_salsa/widget/index.html)

If you want to run it there is examples/sample1.rs with the crate.

The crate contains more stuff, but it's late alpha/early beta now.

This list was put up as requirements:

## Features of solution

* Will be used by each ratatui widgets that currently scroll: Paragraph, List, Table

Yes

* Can be used by those that don't currently: Calendar, Barchart, Chart, Sparkline?

Not as they are.

But a viewport widget could implement these traits and render any
of these to a temporary buffer.

I haven't tried it, but buffer seems good enough for this.

* Can be used by external widgets that need scroll behavior

Yes

I tried

- tui_textarea
  The information necessary exists, but there is no public api for it.
  Would need only a small patch.
  The example uses the cursor api to do something close.

- tui-rs-tree-widget
  It would work nicely if implemented directly for the widget.
  And it has enough public apis that a wrapper can work too.

* Scrolling based on a line / row / item basis

Abstract notion of an offset. Can denote any of the above.
Could be made configurable for a widget, if it wants to switch its scrolling logic.

* Supports non-scrolled areas in a widget (e.g. table or list header / footer)

Yes. Rendering is the job of the widget. Scrolling only handles some offset.

* Supports scrolling a specified item into view (e.g. selected list item, top / bottom item)

No. This must be implemented by the widget.

* Supports single line / row / item scrolling (possibly both line and item in the same widget)

Yes, but not at the same time. A widget could have a switch that changes its behaviour though.

* Supports multiple scrolling (mouse scroll is often nice to scroll by some amount > 1)

Definitively.

* Supports scrolling by partial / full page

Yes

* Supports / guides truncation / partial rendering of items

Only if the widget supports it.

* Only supports the StatefulWidget approach (not Widget)

Needs some place to store state. But an adapter should always be possible. (See ParagraphExt).

* Should avoid having to implement the same logic on each widget (e.g. struct over trait when reasonable)

There could be some struct to hold the data, but I doubt all widgets that could
support scrolling will make use of it. That's rather an implementation detail of
a specific widget.

* Scrollbar

Yes, both. On each side if wanted.

* Hide scrollbar (for helping clipboard copy work)

```rust 
pub enum ScrollbarPolicy {
    Always,
    #[default]
    AsNeeded,
    Never,
}
```

With AsNeeded the widget can control it too.

* Height calculation

That's up to the widget.

* Visible items calculation

That's up to the widget too.

* Querying scroll direction (useful in a list, where scrolling up / down should show the full new item)

Don't understand what's meant here?

* Smooth scroll (scrolling to an item / page in steps based on tick rate)

That's out of scope. There's no way enough infrastructure to accomplish that.

Could add something like:

```rust
fn ticked_scroll_to(tick_state: &mut TickState);
```

with

```
struct TickState {
  start_offset: usize,
  end_offset: usize,
  tick: usize,
  steps: usize
}
```

which is driven by some external timer.

* Should not alter existing traits to implement

Adds two traits. One for the widget and one for the widget-state.

* Should be able to implement existing widget scroll config on top of this (and mark calls as deprecated to guide users
  to new implementation)

The current behaviour can stay. Maybe needs a flag to switch off the 'scroll-selected-to-visible'
behaviour that exists.

* Should be keyboard and mouse friendly

Mouse interactions can be centralized on the Scrolled widget.
Keyboard navigation rather is the domain of a specific widget.

List/Table may share some behaviour, but something like tree-widget will hardly conform.

* Horizontal and vertical scroll bars

Yes

* Show scrollbar options should be { Always, Auto, Never }

Yes

Questions / things to look into / choices

* Are there requirements / constraints that I've missed?

Wrapping text should probably not scroll at all, or maybe it needs
some extra width parameter which controls the wrap, and then it
could scroll horizontally.

* Should the scroll methods return a Result to indicate when they cannot scroll
  in the requested direction? Or should they just let the render method just fix
  the invalid scroll?

Letting the widget correct invalid offsets should be fine.
It has to validate the state anyway when it's rendering its content.

* a. if the scroll returns a result, we can beep / flash / take some action
  to load more results when a user tries to scroll past the end / beginning of a list of items.

TODO: Not covered yet.

* ScrollState or ViewPort approach?

ViewPort seems only useful for a line/column based approach.
There could be a Viewport widget that supports scrolling.
That one would fit into this proposal just fine.

* Should the behavior for scrolling by items be seperated from the behavior for scrolling by lines?

If a widget wants this differentation it can.
This will probably need to different rendering paths though.

* Struct vs Trait for scroll behavior

Trait

* How much of the common behavior can we pull out of widgets and into the types
  that support scrolling implementation (e.g. scrolling specific items into view and truncation)?

I don't think you can for the whole set of widgets that could scroll.
But it should be possible for some subsets. List and Table are very similar
as they are implemented now. Maybe Paragraph and tui-image could share some?

* Does this conflict with anything currently being worked on?

Maybe? I don't have enough overview.

* a. Maybe the flexbox PR? wip(layout): flex layout - draft / poc / rfc #23

That's a new layout-engine, as I understand it?
It will probably need a width to work against.
So basically the same as with wrapping Paragraph,
either don't scroll horizontally or use some extra
width for its purposes.

* Are there any other examples of scrolling currently being implemented elsewhere worth looking at?
  a. libraries in other languages?

I know Java/Swing. It uses a viewport. It has always been difficult to use when you have
something different from an image.

* Should we be able to scroll tables horizontally by cells? This behavior might be useful to
  implement a carousel object

That's out of scope, I think.

* How would infinite scroll be handled?

What's meant with that one?

* I wonder if it's worth splitting out methods for helping with layout / item bounds into
  a scrollable trait widgets can implement to make it easy to calculate bounds / visibility of items.


