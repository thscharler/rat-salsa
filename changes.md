# 0.18.2

* fix docs
* don't publish gifs

# 0.18.1

* Internal updates to rat-scroll changes. No external API change.

# 0.18.0

* update ratatui to 0.28

# 0.17.0

* break: horizontal scrolling scroll cell-wise instead of column-wise.
    * adds Table::auto_layout_width
    * adds TableState:::scroll_to_col
* break: rename FTable to plain Table

* feature: TableState::items_added() and TableState::items_removed() to
  update part of the state to reflect changes in the data. Sometimes
  useful.
* fix: TableState::select() must not constrain the selection.
* fix: panics with offsets near usize::MAX

# 0.16.0

* refactor: replace Scrolled widget with internal handling via Scroll<'a>.
* sync naming methods
* sync key-handling for the different selection-models

* rename FEditTable to EditFTable

# 0.15.5

* breaking: move focus and invalid from the widget to the state.
  this didn't work for StatefulWidgetRef. Use rat-focus:FocusFlag, but just
  as a bool for rendering.
* feature: support StatefulWidgetRef
    * add cloned() to TableDataIter for support.
* breaking: change data() and iter() to take an impl Trait instead of
  &dyn Trait. Aligns better with builders.

* fix horizontal scrollbar
* fix nth() implementation

# 0.15.4

* adds FEditTable for editing support
* add FTableContext for extra information when rendering cells.
* change rendering to render each row to a temp buffer.
  prepare for char-wise horizontal scrolling, and helps with
  clipping.

* add FTable::no_row_count() for Iterators with no known length.
* add FTable::rows(), FTable::columns()
* rename FTable::new() to FTable::new_ratatui().
* remove Styled impl

* FTableState::base_column_areas are now created for *all* columns,
  not only the visible ones. But they may be clipped to nothing.

# 0.15.3

* Add flags for which selection the focus color should apply when focused.
* Add header(), footer() and widths() to TableData and TableDataIter traits.
* Add header_style() and footer_style() to FTable and FTableStyle.
* Add clear(), clear_offset() and clear_selection()
* Add has_selection()

FIX

* Length calculated while iterating diverged from given length. Fixed.
* Fix panic when rendering short tables with known number of rows.
* Fix result of event-handling. Don't ever use Outcome::Unchanged for mouse events.
  Results in nice quirks.
* Vertical scrollbars are now always enabled, when rendering an Iterator.

# 0.15.2

Forgot to remove Debug trait bounds.

# 0.15.1

Missed some warnings.

# 0.15.0

* Add trait TableDataIter as a possible data-source.
  This allows rendering the FTable when all you got is an iterator.

# 0.14.0

* Use the method names from the ScrollingState trait.
* Use new MouseFlags.

# 0.13.1

* Fix versions.

# 0.13.0

* Use the same names as the ScrollingState trait for the same functions.

# 0.12.1

* Wrongly used area dimensions in some places.
* Add inherent methods for different selection models.

# 0.12.0

* Last should have been 0.12.0 instead of 0.11.4

# 0.11.4

* add need_scroll() for Scrollbars.
* remove StatefulWidgetRef. Not useful here.

# 0.11.3

* Use rat-event::Outcome

# 0.11.2

* ?

# 0.11.1

* Add spacer width to the resulting column areas.
  Gives a smoother behaviour and should be conceptually ok.

# 0.11.0

* Implement trait UsedEvent for Outcome.

# 0.10.0

* Initial release, copied from test area.
