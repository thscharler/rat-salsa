# 0.14.0

* add FileDialog
* add Split
* add Tabbed
* moved View and Viewport from rat-scrolled here.
* PopupMenu: add separators.

* List: add inline editing
* TextArea: tab support, undo, sync via replaying changes.
* TextArea: styling

... and about a few hundred changes more ...

# 0.13.0

Move all the widgets from rat-input over here. The original reason for the
split no longer applies, only the burden of maintaining the separation.

* use new internal Scroll<'a> instead of Scrolled widget.

# 0.12.4

* fix: reexport TextAreaStyle
* fix: Event-handling for TextArea
* feature: Add RMenuBar, RPopupMenu.

# 0.12.3

* impl StatefulWidgetRef for widgets.
    * this moved focus and invalid up to the rat-input widgets (et al.)
* date_input and number_input should provide all functions.
* fix: REditTable should use the focus flag.

# 0.12.2

* Prefix all widgets with 'R' to disambiguate from their rat-input cousins.

* add REditTable
* add RNumberInput
* add HasFocus for container widgets.

* fix: screen_cursor() should only return a value if the widget
  is focused. not correct everywhere.
* hack: Block event processing for TextInputState and MaskedInputState when gained_focus().
  This avoids thrashing the selection with the focus-click.

# 0.12.1

* add various functions to FTable
* add Fill widget

* fix missing event-handling for FTable
* fix missing re-exports
* fix broken event-handling. is_focused() is essential here.

# 0.12.0

* update FTable

# 0.11.0

* migrated after upstream change in rat-event

# 0.10.0

* Added all available widgets now.

# 0.9.0

Initial version from test area.

/// historic -- from rat-input

# 0.17.0

Discontinued. Moved everything to rat-widget as the original reason for
this split is no longer valid.

# ...

* fix: TextArea doesn't use focus-style. Too much color.
* add layout_middle to layout Rect with 4 outer constraints.
* use revert_style for fallback styles.
* use new internal Scroll<'a> instead of Scrolled widget.

# 0.16.6

* Add PopupMenu, MenuBar widgets. Synchronize APIs with MenuLine.

# 0.16.5

* refactor: moved focus and invalid from the widget to the state.
  when using StatefulWidgetRef this was the wrong place.
* impl StatefulWidgetRef
* DateInput, NumberInput: add all functions from the underlying MaskedInput.

* fix MsgDialog: must consume all events.
* fix TextInput replace text.
* fix Button + Enter
* fix Button + orphaned release Enter

# 0.16.4

* add NumberInput

* rename new_localized() to new_loc()
* fix: TextInput shouldn't render selection if not focused.
*

# 0.16.3

* add Fill widget. Clears an area.

* fix menuline panic by `- 1`
* fix strange but when a menu is selected at startup. reacted to Release-Enter
  of starting the program on the command line.

# 0.16.2

* add label_at, widget_at to LayoutEdit.

# 0.16.1

* rat-event got a reorg. mirror this.

# 0.16.0

* Use new MouseFlags.

# 0.15.0

* Add TextArea.
* Add support for 2-wide Emojis. Works ok. Input in Windows-Terminal
  seems somewhat broken? Alacritty does better, so I think its Windows-Terminal.
  Or somebody mixes up the events? Simple emojis work though, but the
  combined ones are jittery and break rendering sometimes ....
    * Added for TextArea, TextInput and MaskedInput
* API cleanup between the three text input widgets.

# 0.14.0

* Remove StatefulWidgetRef

# 0.13.3

* Add optimization when dragging the cursor to select text.
  Only return Changed if the selection changed.

# 0.13.2

* Use rat-event::Outcome

# 0.13.1

* Add missing Clone, Debug, Default.

# 0.13.0

* Use new trait UsedEvent.

# 0.12.0

* Add layout_edit() and layout_dialog()

# 0.11.0

* Add calender widget `Month`
* Add menu widget `MenuLine`
* Add basic `MsgDialog`
* Add widget `StatusLine`

# 0.10.1

Fix some docs.

# 0.10.0

* Move HandleEvent trait to separate crate and reexport.
* Add Button and DateInput

# 0.9.0

Initial release with TextInput and MaskedInput