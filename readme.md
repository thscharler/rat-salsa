# Scrolled

Scrolled adds support for widgets that want to scroll their content.

Scrolled works analogous to Block, as you set it on the widget struct.
The widget can decide wich scrolling it supports, horizontal, vertical
or both.

## ScrolledState

This struct holds the information necessary for scrolling, and is
embedded in the widgets state.

* *max_offset* - Maximum allowed offset for scrolling. This offset is
  calculated as `item_count - last_page_items`. Both are abstract
  values and can denote items or columns/rows as the widget sees fit.

  With the maximum calculated like this, Scrollbar renders nicely
  and the widget still has a full page on display.

* *overscroll_by* - By how much can the max_offset be exceeded.
  This allows displaying some empty space at the end of the content,
  which can be more intuitiv for some widgets.

* *page_len* - Length of the current displayed page. This value
  can be used for page-up/page-down handling and for calculating
  the value used with the scroll-wheel.
* *scroll_by* - How many items are scrolled per scroll event.
  When not set it defaults to 1/10 of the page_len, which gives a
  decent median between scroll speed and disorientation.
* *offset* - The current offset used for display.

Each widget decides on its own, how it wants to expose these values
on the surface.

Proposed names are

* vertical_offset()/set_vertical_offset()
* vertical_page_len()
* horizontal_offset()/set_horizontal_offset()
* horizontal_page_len()

The other values are probably not very useful at the surface level.
If the widget has only vertical/horizontal scrolling the prefixes can
be left out.

These two are useful too

* scroll() - relative scrolling using one or two isize parameters.
  There is ScrolledState::change_offset() to support this.
* scroll_to()/horizontal_scroll_to()/vertical_scroll_to() - Scroll
  to an absolute offset.

  Calls set_offset() directly, but is a good position to add
  `scroll the selection instead of the offset` functionality.

        Remark: Before I had a Scrolled<T> container widget, which
        did all this. But using it this way proved very burdensome. 
        * Widgets with non-trivial scrolling need their own support
          anyway, Scrolled couldn't add much to that except showing
          the scrollbars.
        * The deep layering necessitated long dotted paths all the time.
        * Forwarding to an event-handler for the contained widget
          was some untertaking with wrapped event-handler qualifiers, 
          and multiple layers of wrapped outcome-types. 
        * And all that for some measly two scrollbars ...

## Implementation

* layout_scroll() can calculate the areas in the presence of one/two Scroll
  and a Block, and makes all of them align smoothly (there are edge cases).

* Scroll uses ScrollBarOrientation for the positioning of the scrollbars.
  But I don't support horizontally layout scrollbars used for vertical
  scrolling and vice versa. Where you accept a Scrolled, you should
  immediately call override_vertical()/override_horizontal() to fix any
  misalignment. Vertical scrollbars on the left side and horizontal scrollbars
  on top are still fine.

* If your widget has horizontal and vertical scrolling, a single scroll() function
  that sets both of them to the same scroll is nice.

* Event-handling
    * There is a MouseOnly event-handler for ScrollState. It reacts to mouse
      clicks and drag directly on the scrollbar-area, and returns a
      ScrollOutcome::Offset. You have to change the actual offset to this
      value, the event-handler doesn't do this for you. The reason is
      `scroll my selection` mode. But you can use this value to set the
      selection too.
    * There is a helper struct ScrollArea, which implements a MouseOnly event-handler
      for the mouse-wheel functionality. This one returns ScrollOutcome::Delta,
      which, again, has to be used by your widget. At least this can avoid another
      50 lines of often copied code. It uses ScrollUp/ScrollDown for vertical,
      and ALT-ScrollUp/ScrollDown for horizontal scrolling.

# View and Viewport

These two add scrolling support for Widgets/StatefulWidgets that have no
builtin scrolling. They render the widgets to a temporary buffer and do the
offsetting afterwards.

Both are exactly the same, but the duplication is necessary because of clashing
traits.  



