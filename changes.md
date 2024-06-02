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