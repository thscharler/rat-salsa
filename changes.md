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