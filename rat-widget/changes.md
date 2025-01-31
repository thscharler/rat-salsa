# 1.0.1

* fix: ensure that the select_style is always patched onto the
  base-style. This makes a select-style `Style::new().underlined()`
  work fine.

# 1.0.0

... jump ...

# 0.37.2

* fix: Choice: manual popup-len should still consider number
  of items.
* feature: Choice: add width() and height() to ChoiceWidget; add
  layout() to ChoicePopup
* feature: FileDialog: use F1-F5 to jump to different parts.

# 0.37.1

* feature: add Hover widget
* fix: MsgDialog should focus the text.
* fix: use dirs instead of directories_next.

# 0.37.0

* break: CalendarSelection: remove clear()
* break: CalendarSelection: rename len() to count()
* break: Button: rebuild hot-key. make a very, very minimal thing
  that works with button.
* break: ListSelection add count()
* break: Pager&Co: add try_label_str_for() and try_label_str() for
  optional results. default returns "" as default for missing labels.
* feature: add new widget Form that further reduces the surface
  at the price of loosing functionality.

# 0.36.0

* break: full rebuild of the calendar module
    * add a Selection model
        * NoSelection
            * navigation but no selection
        * SingleSelection
            * single date selection + all the navigation.
        * RangeSelection
            * select date ranges
            * week/month selection
    * add Ctrl+Home for go to today.
    * add Home/End for start/end of month.
    * add click+drag selection for days and weeks
    *
    * add CalendarState for a true multi-month calendar. there is no default widget using
      this, as there are many possible layouts.
    * add Calender3 as a simple scrolling 3-month calendar. Using CalendarState.
    * rename MonthStyle to CalendarStyle

* break: Choice: remove Regular event-handling and do everything
  in the Popup event handler. Simplifies usage.

* fix: Choice eventhandling.
    * move_to, move_up and move_down now return a ChoiceOutcome to cover all of the behaviour.
    * event-handling for the popup is tricky. rewritten to make it more stable.
    * add PageUp/Down, Home, End key handling.
* fix: Splitter now does the base style correctly.

* feature: Splitter: use alt+ArrowKey for split navigation.
* feature: Splitter: add horizontal() and vertical() constructors.
* feature: Paired: can work with Widget now.

# 0.35.0

* break: ButtonOutcome moved to the event module as all the other
  Outcome types.
* break: add CheckOutcome::Value, ChoiceOutcome::Value, RadioOutcome::Value
  and SliderOutcome::Value to signal changed values separate from other
  changes.
* break: Outcome types are no longer non_exhaustive. Any change to an
  outcome type is breaking regardless of this flag.
* break: create ChoiceCore to define common behaviour between Choice
  and Radio.
    - Both can work with empty lists of choices now.
      They preserve any value set with set_value() regardless if it is
      present in the list. Such an undefined value will be displayed
      with all unchecked radios or an empty choice. This gives decent
      behaviour if the list of choices diverges from the actual data.
      At least it prevents a silent reset.
    - Adds back default_value semantics. The default-value falls back
      to T::default() if not set.
* break: Clipper, SinglePager, DualPager now hold a Rc<RefCell<GenericLayout>>.
  This enables clear() to reset any current layout. This comes in handy
  if some event handling invalidates the layout. This adds valid_layout()
  to all widgets. An empty layout is always invalid. And it detects size changes.

* feature: Allow setting the navigation markers for PagerNav.
  Hide page-numbers if there is only one page. If no explicit
  border is set for PagerNav no space will be reserved for
  the navigation markers. They will render over any content.
* feature: Choice: add clear()
* feature: Choice: add behaviour flags
    - ChoiceSelect::MouseScroll - select in the popup-list with scroll.
    - ChoiceSelect::MouseMove - select in the popup-list with mouse moves.
    - ChoiceSelect::MouseClick - select in the popup-list with click and allows drag.
    - ChoiceClose::SingleClick - close popup with single click
    - ChoiceClose::DoubleClick - close popup with double click.
* feature: calendar::Month uses ISO week-numbers now.
* feature: calender::Month can now operate without setting start-date
  on the widget. It uses any start-date coming from the state.
* feature: calender: better event-handing for &[[MonthState]].
    - add a MultiMonth qualifier for event-handling. This allows
      setting a month-delta when moving beyond the displayed dates.
      It requires Focus now for correctly focusing the current Month widget.
    - Scrolling through dates outside the displayed months now changes
      the start_date in the state. So start-date must not be set
      when rendering the Month widgets.
    - Result is now plain CalOutcome::Day/CalOutcome::Week dependent on
      the actual selection.
    - PageUp/PageDown supported
* feature: Button: add hover functionality.
* feature: Clipper: add scroll_to(widget)

* fix: Choice must not send a change when it reacts to focus changes.
  The relevant repaint is already triggered by Focus itself.
  This leads to a double repaint and possible other disruptions.
* fix: Choice doesn't consume Enter and Esc if the popup is already hidden.
* fix: use unicode_display_width for Slider
* fix: LayoutForm: off by 1
* fix: Clipper renders the correct background style now.

# 0.34.0

* break: Choice: remove default_key, replace with simple select(0).
* break: Choice: change selection from optional to plain usize.
* break: Slider: refactor: change from Option<T> to T. out of bounds is
  handled differently. ease of use.
* break: Checkbox: remove default-settable. too complicated.
* fix: scrolling in Pager shouldn't consume event if nothing happened

# 0.33.1

* moved all rat-crates to one repo

# 0.33.0

* break: LayoutForm

  FormLabel::Str is split in FormLabel::Str and FormLabel::String with
  a simple &str or String as data. Makes using those much easier.
  Both behave the same otherwise.

  Remove FormLabel::Measure and FormWidget::Measure and replace them
  with min_label() and min_widget().

  FormWidget::Wide, StretchX, WideStretchX, StretchXY, WideStretchXY
  all gain a preferred width.

* break: Choice and Radio return an owned value for value().
  There are new value_ref() and value_opt_ref() that return a reference.
  Most of the time the owned value is easier to handle.

* feature: Pager et al get a few new render() methods for special cases.
* feature: add PairedWidget. Can render 2 widgets side by side in one
  layout area. There are a few simple constraints how to divide the
  space.
* feature: Pager et al allow access to the buffer during rendering.
* feature: Choice gets popup_offset()

* fix: SinglePager, DualPager ensure that the current page
  doesn't exceed page-count.
* fix: LayoutForm. StretchY doesn't work on the last page.
* fix: underflows in LayoutForm
* fix: Clipper::layout_size() must return a Size with height u16::MAX.
* fix: Pager styles labels only if expressedly set.
* fix/break: DualPager must use FnOnce for its render functions too.
* fix: Button cannot rely on Key Released. Add a flag to rat-focus
  to recognize this. The button doesn't show a 'clicked' behaviour
  if no Key Released is accessible.
* fix: Button triggered for a lone Mouse-Up event without
  a prior Mouse-Down.

# 0.32.0

* break: Replace StructuredLayout with GenericLayout
    * removes the array-based access of StructuredLayout with Label+Widget areas.
    * adds support to render arbitrary Blocks as part of the layout.
      this allows to add support for groups of widgets during
      layout calculation.

* feature: add new LayoutForm.
    * page-breaking layouts.
    * Flex support
    * widgets can fill the available width.
    * widgets can use remainder space after a page-break.
    * align labels with the widgets.
    * keep and auto-render labels.
    * allow groups of widgets with a surrounding block.
    * blocks can stack.
    * manual page-breaks.
    * line-spacing.
    * margins via Padding().
    * margins can switch between odd/even pages.

    * creates a GenericLayout

* break: add Pager and PageNavigation widgets and reimplement
  SinglePager and DualPager using those.
* break: SinglePager and DualPager use GenericLayout now, breaking all of their api.
* break: Clipper uses GenericLayout now, breaking all of their api.
* break: remove separate ClipperLayout and PagerLayout. Layout now only
  needs a single stage and creates a GenericLayout directly.
* break: layout_edit() reimplemented. Creates a GenericLayout now.
* break: layout_dialog() reimplemented. Creates a GenericLayout now.
* break: layout_grid() reimplemented. Creates a GenericLayout now.

* break: add a value type T to Choice and Radio. This makes
  them so much easier to use, as the underlying values never
  align with the displayed text. Requires only PartialEq from T.
* fix: show focus for Choice

# 0.31.0

* remove uses of ZRect. Choice now adds two areas for its parts.

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
