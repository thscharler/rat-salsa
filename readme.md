!! This is out of date !!

# Scroll

Scroll adds support for widgets that want to scroll their content.

Scroll works analogous to Block, as you set it on the widget struct.
The widget can decide wich scrolling it supports, horizontal, vertical
or both.

## ScrollState

This struct holds the information necessary for scrolling, and is
embedded in the widgets state.

* `max_offset` - Maximum allowed offset for scrolling. This offset is
  calculated as `item_count - last_page_items`. Both are abstract
  values and can denote items or columns/rows as the widget sees fit.
  With the maximum calculated like this, Scrollbar renders nicely
  and the widget still has a full page on display.
* `overscroll_by` - By how much can the max_offset be exceeded.
  This allows displaying some empty space at the end of the content,
  which can be more intuitiv for some widgets.
* `page_len` - Length of the current displayed page. This value
  can be used for page-up/page-down handling. This value is used
  for the scrollbar-thumb.
* `scroll_by` - How many items are scrolled per scroll event.
  When not set it defaults to 1/10 of the page_len, which gives a
  decent median between scroll speed and disorientation.
* `offset` - The current offset used for display.

## Widget implementation

* `layout_scroll()` can calculate the areas in the presence of one/two Scroll
  and a Block, and makes all of them align smoothly (there are edge cases).

* Scroll uses ScrollBarOrientation for the positioning of the scrollbars.
  With that there are the possible combinations VerticalLeft/VerticalRight
  set as horizontal Scroll and vice versa.
  Those are undesirable, and layout_scroll() panics when it gets one of
  these combinations. You should call `override_vertical()` or `override_horizontal`
  where you accept the Scroll to match the expectations.

* If your widget has horizontal and vertical scrolling, a single scroll() function
  that sets both of them to the same scroll is nice.

### Event-handling

* There is a MouseOnly event-handler for ScrollState. It reacts to mouse
  clicks and drag directly on the scrollbar-area, and returns a
  ScrollOutcome.

  The event-handler for ScrollState doesn't change the offset by itself,
  that's up to the widget. This indirect approach gives the widget more
  flexibility.

* `ScrollArea` is a small helper that implements a MouseOnly event-handler.
  It consists of the widget-area and one or two Scroll. It covers all
  the scrolling in the given area and the scrolling by the two ScrollBars.

* This too just returns a ScrollOutcome and changes no values.

* Scroll modes. There are widgets which support switching between
  'scroll-offset' and 'scroll-selection'. There is no direct support
  for this in Scroll. But you can get a good result for the selection
  if you rescale the offset as `(item_len * offset) / max_offset`.
  This gives you a value in the range 0..item_len which can be used
  as selection value.
