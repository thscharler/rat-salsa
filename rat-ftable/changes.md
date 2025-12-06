# 2.0.0

ratatui 0.30

# 1.5.0

* fix dependencies
* feat: add Table::show_empty()

# 1.4.2

* fix dependencies

# 1.4.1

* fix dependencies

# 1.4.0

* feature: add move_deselect(). deselects and resets the offset.
* feature: add width() and height() for a minimum size of the table.
* fix: show some warnings only when activated by a feature flag.
* refactor: EditableTableVecState returns a clone of the Vec now
  instead of moving the Vec.

# 1.3.0

* feature: add layout_column_widths() to calculate the table-width
  according to the sum of all columns. Similar to the old auto_layout_width(),
  but does a proper calculation for all Constraint variants.
* fix: add missing border_style and title_style.
* fix: event handling in EditableTableVec
* fix: event handling in EditableTableVec should honor focus.

# 1.2.1

* fix: conformance with guidelines.

# 1.2.0

* feature: set a default type for Table<Selection=RowSelection>
* feature: Table: add rows_changed()
* fix: EditableTableVec: auto-insert an empty row and start editing if the table starts with 0 rows.
* fix: EditableTableVec: up key when editing should move to the row above. same with down key
* fix: EditableTableVec: removing the last row should start editing a new row immediately.
* fix: rendering the scrollbars lagged. the relevant values where only set after the scrollbars where rendered. fixed by
  splitting block/scollbar rendering.
* fix: deprecate auto_layout_width. not useful.
* fix: remove debug()

# 1.1.1

doc fixes

# 1.1.0

* Some breaking changes for TableEditor.
  // There is currently no known user for this api, so I allow myself to just do it:
    * TableEditorState::Context is no longer clone. It is passed by reference
      instead of an owned value now.
    * Remove TableDataVec trait. The Table for rendering is no longer held
      in the EditableTableVec widget, instead a constructor for this table
      is held. This allows to get rid of the rc for the actual data.
    * Add TableEditorState::set_focused_col(). This allows to directly
      edit a specific cell of the table.
* fix: docs

# 1.0.1

* update dependencies

* fix: #7: scroll_to_selected uses scroll_to_x() instead of scroll_to_col().
* fix: set_row_offset() and set_x_offset() no longer correct the given offset.
  This needed fixing when scrolling to an absolute position.

# 1.0.0

... jump ...

# 0.32.0

* break: add count() to TableSelection.

# 0.31.0

* break: TableEditorState changed to value semantics completely.
* break: Change event-handling from Outcome to TableOutcome.
    - adds TableOutcome::Selected to differentiate selection changes from
      any other changes.

* fix: Table::auto_layout_width() doesn't need a bool parameter.

* feature: EditableTableVec: Enter adds a new row in an empty table.

# 0.30.0

* break: rename for clarity: Editor->TableEditor, EditorState->TableEditorState,
  EditorData->TableDataVec,
  EditVec->EditableTableVec,
  EditTable->EditabledTable,
* break: EditableTable+EditableTableVec: remove separate focus.
  use selected_checked() everywhere. allow insert if table is empty.
* feature: add selected_checked(). this provides a selection that stays in 0..rows

# 0.29.2

* moved all rat-crates to one repo

# 0.29.1

* feature: add border_style to TableStyle. Allows setting the style
  without providing a definite border. When applying the TableStyle
  style+border_style override the settings of a previously set block.

# 0.29.0

* clippy fixes

# 0.28.0

** upgrade to ratatui 0.29 **

* feature: add support for rat-reloc. this allows widgets to change
  position after rendering.

# 0.27.2

* feature: enable styles for Block and Scroll

# 0.27.1

* fix: use new ScrollArea. no visible api change.

# 0.27.0

break: rename fields in TableStyle

# 0.26.0

break: final renames in rat-focus.

# 0.25.1

update dependencies

# 0.25.0

Sync version for beta.

* Reimagine table editing.
    * Adds EditTable for free form.
    * Adds EditVec which keeps the elements while editing.

* fix: scroll_to_row did some scrolling even if the row is visible.
* fix: row_cells didn't correct for offset.

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
