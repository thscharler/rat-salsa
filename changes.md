# 0.30.0

** upgrade to ratatui 0.29 **

* break: removed View and Viewport. These container widgets are
  not good enough. and cumbersome to work with.

* add new View and the view-like Clipper, SinglePager and DualPager widgets.
    * Clipper minimizes the required rendering.
    * SinglePager/DualPager don't scroll but page.
      These don't use a temporary Buffer.
* add new StructuredLayout as a collection of Rect.
    * the various layout_xx() return one.
    * ClipperLayout and PagerLayout use it.
* add Checkbox widget
* add Radio widget
* add Slider widget
*
* feature: make Paragraph more useable. Add focus-style.
* feature: add width() and height() to the widgets with a known/required size.
* feature: show weekdays in Month widget.
* feat: implement RelocatableState for most widgets.
* fix: button click is not visible in some terminals. add armed_delay()
  as a primitive fix, which just sleeps for a few milliseconds.
  that's enough time for the terminal to render the armed-state
  before switching to clicked-state.
* fix: keyboard resizing of Split
* fix: allow drag to select tabs.
* check fallbacks when no styling is applied. across all widgets.

* refactor: styles for FileDialog
* fix: revert_style()
* fix: arg order in reset_buf_area()

# 0.29.0

* break: rename DualPagerRender to RenderDualPager
* break: rename SinglePagerRender to RenderSinglePager
* break: rename ClipperRender to RenderClipper
* break: move View and Viewport to separate module view.
* break:

* feature: add ViewStyle
* feature: add ChoiceStyle
* feature: add block and scroll styling to ListStyle
* feature: add block to MsgDialogStyle
* feature: add ParagraphStyle
* feature: add ShadowStyle
* feature: add block to SplitStyle
* feature: add tab_type, placement and block to TabbedStyle
*

# 0.28.2

* feature/fix: some api refinement with SinglePager and DualPager.

# 0.28.1

* feature: Choice widget looks quite complete now.
* update: rat-scrolled
* feature: add widget Clipper, similar to xxPager but does scroll.
* break: sync api for Clipper, SinglePager and DualPager.
  xxPager now uses into_widget() style and has a second stage
  that does all the area mapping.

# 0.28.0

* break: change name in xxxStyle

* feature: Choice widget started, wip.

* fix: changes in rat-popup
* fix: bigger close area for tabs

# 0.27.0

* break: final renames in rat-focus.

* add widgets SinglePager and DualPager.

# 0.26.0

* break: renamed MenuBarState to MenubarState.
* break: PopupMenu uses rat-popup as base. Changes the API.

* feature: add Shadow widget, that draws a drop shadow.

# 0.25.0

Sync version for beta.

* feature: Add movement between different calender::Month.
* feature: Add SplitResize strategy for resizing splits.
* fix: sync+document state structs.
* refactor: sync list edit with table edit. add examples.
* refactor: moved menu widgets to separate crate rat-menu.
* refactor: simplified Tabbed. The internal widgets for the frame
  are no longer exposed. Just an enum left.

# 0.16.2

* dont package gif

# 0.16.1

* feature: add day and week selection to Month.
* feature: add multi-month key-handling to Month.
* refactor: rename inner_area to inner in Button.
* fix: fix quirks with hide_split()
* fix: layout + focus in msgdialog.
* fix: keys with Paragraph

# 0.16.0

Beta preparations started.

* refactor: add lengths to SplitState and disentangle from
  split-areas. Add documentation for the state-values which
  are meant to be changed and which are not.

* feature: add hide_split()/show_split() to Splitter.

* feature: Add better render-parameters for AttachedTab.
  It can no fill in as the 4th side of a 3sided border around
  the widget.

* refactor: remove text-widgets and move them to rat-text.
  Rebuild around a common TextStore trait and build the
  functionality for all widgets upon that. Implement a
  single-line String based version and a second Rope based
  version for multi-line.

  re-export the result here.

* feature: simplify EditList
* feature: FileDialog can now work as directory-chooser.

* remove: Fill widget. Replaced with reset_buf_area() and
  fill_buf_area().
* remove: Remove split_char, join_0_char and join_1_char
  from Splitter. Overkill.

* fix: changes in rat-scroll
* fix: render bug in PopupMenu
* fix: some more bugs due to making StatefulWidgetRef
  a feature.
* fix: Creating a directory in FileDialog didn't work.

# 0.15.0

* update ratatui to 0.28

* use text widgets from rat-text instead of the old ones.

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
