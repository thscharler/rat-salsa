# Details, details

## Focus

### Navigation

* first(): Focus the first widget.
* next()/prev(): Change the focus.
* focus(): Focus a specific widget.
* focus_at(): Focus the widget at a position.
* expel_focus(): Make the focus go away from a widget or a container.
  (I use this when a popup will be hidden. It's nice if the focus doesn't
  just dissapear).

### Debugging

* You can construct the FocusFlag with a name.
* Call Focus::enable_log()
* You might find something useful in your log-file.

### Dynamic changes

You might come to a situation where

* Your state changed
    * which changes the widget structure/focus order/...
        * everything should still work

then you can use one of

* remove_container
* update_container
* replace_container

to change Focus without completely rebuilding it.

They reset the focus state for all widgets that are no longer
part of Focus, so there is no confusion who currently owns the
focus. You can call some focus function to set the new focus
afterwards.

## Navigation flags

This flag controls the interaction of a widget with Focus.

* None - Widget is not reachable at all. You can manually focus() though.
* Mouse - Widget is not keyboard reachable.
* Regular - Normal keyboard and mouse interactions.

* Leave - Widget can lose focus with keyboard navigation, but
  but not gain it.

  For widgets like a MenuBar. I want a hotkey for going to
  the menubar, but using tab to leave it is fine.

* Reach - Widget can gain focus, but not loose it.

  There is one bastard of a widget: TextAreas. They want
  their tabs for themselves.

* Lock - Focus is locked to stay with this widget.

  e.g. To implement a sub-focus-cycle.
  When editing a table-row I want the editor widgets form
  a separate focus-cycle during editing. And leaving the
  table before either commiting or canceling the current edit
  is disturbing. So when the table enters edit-mode it switches
  to Lock and creates a new Focus with only the edit-widgets.
  When editing is done the table switches back into Regular mode.

* ReachLeaveFront - Widget can be reached with normal keyboard
  navigation, but only left with Shift-Tab.
* ReachLeaveBack - Inverse of ReachLeaveFront.

  These flags can achieve similar effects as Lock, but leaving an
  exit open. I use this for MaskedTextField to navigate the different
  sections of an input mask. e.g. 'month' Tab 'day' Tab 'year' Tab next widget.
