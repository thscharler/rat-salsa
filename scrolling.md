Features of solution

* Will be used by each ratatui widgets that currently scroll: Paragraph, List, Table

Can do so.

* Can be used by those that don't currently: Calendar, Barchart, Chart, Sparkline?

Probably

* Can be used by external widgets that need scroll behavior

Yes

* Scrolling based on a line / row / item basis

Abstract notion of an offset. Can denote any of the above.
Could be made configurable for a widget, if it wants to alternate it's scrolling logic.

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

Needs some place to store state. But an adapter should be possible.

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

I don't think you can for the whole set of widget that could scroll.
But it should be possible for some subsets. List and Table are very similar
as they are implemented. Maybe Paragraph and tui-image could share some?
// Maybe there's too much focus on commonalities rather than having widgets that
// are different enough to warrant the effort.

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


