use crate::core::FocusCore;
use crate::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular, crossterm, ct_event};
use ratatui_crossterm::crossterm::event::Event;
use std::cell::RefCell;
use std::rc::Rc;

/// Focus deals with all focus-related issues.
///
/// Use [FocusBuilder] to construct the current Focus.
///
/// This is usually quick enough to do it for each event.
/// It has to be rebuilt if any area has changed, so
/// rebuilding it after a render() is fine.
#[derive(Default, Debug)]
pub struct Focus {
    pub(crate) last: Rc<RefCell<FocusCore>>,
    pub(crate) core: Rc<RefCell<FocusCore>>,
}

macro_rules! focus_debug {
    ($core:expr, $($arg:tt)+) => {
        if $core.log.get() {
            log::log!(log::Level::Debug, $($arg)+);
        }
    }
}

impl Focus {
    /// Writes a log for each operation.
    pub fn enable_log(&self) {
        self.core.borrow().log.set(true);
    }

    /// Writes a log for each operation.
    pub fn disable_log(&self) {
        self.core.borrow().log.set(false);
    }

    /// Enable insta-panic if any function is called
    /// with a widget that is not part of the Focus.
    #[deprecated(since = "2.3.0", note = "no longer in use")]
    pub fn enable_panic(&self) {}

    /// Disable insta-panic.
    #[deprecated(since = "2.3.0", note = "no longer in use")]
    pub fn disable_panic(&self) {}

    /// Sets the focus to the given widget and remembers
    /// the previous focused widget. If the focus is
    /// currently set to the given widget it sets the
    /// focus back to the previous widget.
    pub fn flip_focus(
        &self,
        widget_state: &'_ dyn HasFocus,
        flip_focus: &'_ mut Option<FocusFlag>,
    ) {
        let flag = widget_state.focus();
        let core = self.core.borrow();
        focus_debug!(core, "flip-focus {:?} {:?}", flag.name(), flip_focus);
        if flag.is_focused() {
            focus_debug!(core, "    -> widget is focused");
            if let Some(flip_focus) = flip_focus {
                core.focus(flip_focus, true);
            } else {
                focus_debug!(core, "    -> no previous widget");
            }
            *flip_focus = None;
        } else {
            focus_debug!(core, "    -> other is focused");
            *flip_focus = core.focused();
            core.focus(&flag, true);
        }
    }

    /// Sets the focus to the given widget.
    ///
    /// This changes the focus and the gained/lost flags.
    /// If this ends up with the same widget as before
    /// gained and lost flags are not set.
    ///
    /// This will ignore the [Navigation] flag of the
    /// currently focused widget.
    ///
    /// You can also use a container-widget for this.
    /// It will set the focus to the first navigable widget
    /// of the container.
    #[inline]
    pub fn focus(&self, widget_state: &'_ dyn HasFocus) {
        let core = self.core.borrow();
        let flag = widget_state.focus();
        focus_debug!(core, "focus {:?}", flag.name());
        core.focus(&flag, true);
    }

    /// Sets the focus to the widget by its widget-id.
    ///
    /// This can be useful if you want to store references
    /// to widgets in some extra subsystem and can't use
    /// a clone of the FocusFlag for that.
    ///
    /// This changes the focus and the gained/lost flags.
    /// If this ends up with the same widget as before
    /// gained and lost flags are not set.
    ///
    /// This will ignore the [Navigation] flag of the
    /// currently focused widget.
    ///
    /// You can also use a container-widget for this.
    /// It will set the focus to the first widget of the
    /// container.
    #[inline]
    pub fn by_widget_id(&self, widget_id: usize) {
        let core = self.core.borrow();
        let flag = core.find_widget_id(widget_id);
        focus_debug!(core, "focus {:?} -> {:?}", widget_id, flag);
        let Some(flag) = flag else {
            return;
        };
        core.focus(&flag, true);
    }

    /// Set the focus to the first navigable widget.
    ///
    /// This changes the focus and the gained/lost flags.
    /// If this ends up with the same widget as before
    /// gained and lost flags are not set.
    ///
    /// This will ignore the [Navigation] flag of the
    /// currently focused widget.
    #[inline(always)]
    pub fn first(&self) {
        let core = self.core.borrow();
        focus_debug!(core, "focus first");
        core.first();
    }

    #[deprecated(since = "1.1.2", note = "use focus() instead")]
    pub fn first_in(&self, container: &'_ dyn HasFocus) {
        self.focus(container);
    }

    /// Clear the focus for all widgets.
    ///
    /// This will reset the focus, gained and lost flags for
    /// all widgets.
    #[inline(always)]
    pub fn none(&self) {
        let core = self.core.borrow();
        focus_debug!(core, "focus none");
        core.none();
    }

    /// This widget will have the focus, but it is not
    /// yet part of the focus cycle.
    ///
    /// * If the widget is part of this focus-cycle this will work
    ///   just as [focus](Focus::focus).
    /// * Otherwise it will build a temp Focus and call [first](Focus::first).
    ///   This will work for a container widget too.
    #[inline(always)]
    pub fn future(&self, widget_state: &'_ dyn HasFocus) {
        let core = self.core.borrow();
        let flag = widget_state.focus();
        focus_debug!(core, "future {:?}", flag.name());
        if core.is_widget(&flag) || core.is_container(&flag) {
            focus_debug!(core, "    | known widget. use focus() instead");
            core.focus(&flag, true);
        } else {
            focus_debug!(core, "    | set focus to none");
            // clear our focus.
            core.none();
            // build focus for new widget. can't know whether it's
            // a single widget or a container, so like this.
            focus_debug!(core, "    | build focus for widget and focus first");
            let mut builder = FocusBuilder::new(None);
            builder.widget(widget_state);
            let focus = builder.build();
            if core.log.get() {
                focus.enable_log();
            }
            focus.core.borrow().first();
        }
    }

    /// Change to focus to the widget at the given position.
    ///
    /// This changes the focus and the gained/lost flags.
    /// If this ends up with the same widget as before
    /// gained and lost flags are not set.
    ///
    /// If the current widget has a [Navigation::Lock], this will
    /// do nothing.
    #[inline(always)]
    pub fn focus_at(&self, col: u16, row: u16) -> bool {
        let core = self.core.borrow();
        focus_debug!(core, "focus at {},{}", col, row);
        match core.navigation() {
            Some(Navigation::Lock) => {
                focus_debug!(core, "    -> locked");
                false
            }
            _ => {
                let r = core.focus_at(col, row);
                r
            }
        }
    }

    /// Sets the obscured state for the given position.
    ///
    /// Hit-tests all widgets for the given position and
    /// sets obscured to false if the widget-area is hit
    /// and there are no containers/widgets with a *higher*
    /// z-index.
    #[inline(always)]
    pub fn find_obscured(&self, col: u16, row: u16) {
        let core = self.core.borrow();
        focus_debug!(core, "find_obscured {} {}", col, row);
        core.find_obscured(col, row);
    }

    /// Focus the next widget in the cycle.
    ///
    /// This function will use the [Navigation] of the current widget
    /// and only focus the next widget if it is `Leave`, `ReachLeaveBack` or
    /// `Regular`.
    ///
    /// If no field has the focus the first navigable one gets it.
    /// Sets the focus, gained and lost flags. If this ends up with
    /// the same widget as before focus, gained and lost flag are not set.
    #[inline]
    pub fn next(&self) -> bool {
        let core = self.core.borrow();
        match core.navigation() {
            None => {
                focus_debug!(core, "focus first");
                core.first();
                true
            }
            Some(Navigation::Leave | Navigation::ReachLeaveBack | Navigation::Regular) => {
                focus_debug!(
                    core,
                    "next after {:?}",
                    core.focused().map(|v| v.name()).unwrap_or("None".into())
                );
                core.next()
            }
            v => {
                focus_debug!(
                    core,
                    "next after {:?}",
                    core.focused().map(|v| v.name()).unwrap_or("None".into())
                );
                focus_debug!(core, "    -> but navigation says {:?}. skip", v);
                false
            }
        }
    }

    /// Focus the previous widget in the cycle.
    ///
    /// This function will use the [Navigation] of the current widget
    /// and only focus the next widget if it is `Leave`, `ReachLeaveFront` or
    /// `Regular`.
    ///
    /// If no field has the focus the first navigable one gets it.
    /// Sets the focus, gained and lost flags. If this ends up with
    /// the same widget as before focus, gained and lost flag are not set.
    #[inline]
    pub fn prev(&self) -> bool {
        let core = self.core.borrow();
        match core.navigation() {
            None => {
                focus_debug!(core, "focus first");
                core.first();
                true
            }
            Some(Navigation::Leave | Navigation::ReachLeaveFront | Navigation::Regular) => {
                focus_debug!(
                    core,
                    "prev before {:?}",
                    core.focused()
                        .map(|v| v.name().to_string())
                        .unwrap_or("None".into())
                );
                core.prev()
            }
            v => {
                focus_debug!(
                    core,
                    "prev before {:?}",
                    core.focused()
                        .map(|v| v.name().to_string())
                        .unwrap_or("None".into())
                );
                focus_debug!(core, "    -> navigation says {:?}. skip", v);
                false
            }
        }
    }

    /// Focus the next widget in the cycle.
    ///
    /// Applies some extra force to this action and allows
    /// leaving widgets that have [Navigation] `Reach` and `ReachLeaveFront`
    /// in addition to the regular `Leave`, `ReachLeaveBack` or
    /// `Regular`.
    ///
    /// If no field has the focus the first navigable one gets it.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with
    /// the same widget as before focus, gained and lost flag are not set.
    #[inline]
    #[deprecated(since = "2.3.0", note = "overengineered")]
    pub fn next_force(&self) -> bool {
        let core = self.core.borrow();
        match core.navigation() {
            None => {
                focus_debug!(core, "focus first");
                core.first();
                true
            }
            Some(
                Navigation::Leave
                | Navigation::Reach
                | Navigation::ReachLeaveFront
                | Navigation::ReachLeaveBack
                | Navigation::Regular,
            ) => {
                focus_debug!(
                    core,
                    "force next after {:?}",
                    core.focused().map(|v| v.name().to_string())
                );
                core.next()
            }
            v => {
                focus_debug!(
                    core,
                    "force next after {:?}",
                    core.focused().map(|v| v.name().to_string())
                );
                focus_debug!(core, "    -> navigation says {:?}. skip", v);
                false
            }
        }
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Applies some extra force to this action and allows
    /// leaving widgets that have [Navigation] `Reach` and `ReachLeaveBack`
    /// in addition to the regular `Leave`, `ReachLeaveFront` or
    /// `Regular`.
    ///
    /// If no field has the focus the first navigable one gets it.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with
    /// the same widget as before focus, gained and lost flag are not set.
    #[inline]
    #[deprecated(since = "2.3.0", note = "overengineered")]
    pub fn prev_force(&self) -> bool {
        let core = self.core.borrow();
        match core.navigation() {
            None => {
                focus_debug!(core, "focus first");
                core.first();
                true
            }
            Some(
                Navigation::Leave
                | Navigation::Reach
                | Navigation::ReachLeaveFront
                | Navigation::ReachLeaveBack
                | Navigation::Regular,
            ) => {
                focus_debug!(
                    core,
                    "force prev before {:?}",
                    core.focused().map(|v| v.name().to_string())
                );
                core.prev()
            }
            v => {
                focus_debug!(
                    core,
                    "force prev before {:?}",
                    core.focused().map(|v| v.name().to_string())
                );
                focus_debug!(core, "    -> navigation says {:?}. skip", v);
                false
            }
        }
    }

    /// Is this widget part of this focus-cycle?
    #[inline(always)]
    pub fn is_valid_widget(&self, widget_state: &'_ dyn HasFocus) -> bool {
        let core = self.core.borrow();
        core.is_widget(&widget_state.focus())
    }

    /// Is this a container that is part of this focus-cycle?
    #[inline(always)]
    pub fn is_valid_container(&self, widget_state: &'_ dyn HasFocus) -> bool {
        let core = self.core.borrow();
        core.is_container(&widget_state.focus())
    }

    /// Returns the focused widget as FocusFlag.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn focused(&self) -> Option<FocusFlag> {
        let core = self.core.borrow();
        core.focused()
    }

    /// Returns the focused widget as widget-id.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn focused_widget_id(&self) -> Option<usize> {
        let core = self.core.borrow();
        core.focused().map(|v| v.id())
    }

    /// Returns the debug name of the focused widget.
    #[inline(always)]
    pub fn focused_name(&self) -> Option<String> {
        let core = self.core.borrow();
        core.focused().map(|v| v.name().to_string())
    }

    /// Returns the [Navigation] flag for the focused widget.
    #[inline(always)]
    pub fn navigation(&self) -> Option<Navigation> {
        let core = self.core.borrow();
        core.navigation()
    }

    /// Returns the widget that lost the focus as FocusFlag.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn lost_focus(&self) -> Option<FocusFlag> {
        let core = self.core.borrow();
        core.lost_focus()
    }

    /// Returns the widget that gained the focus as FocusFlag.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn gained_focus(&self) -> Option<FocusFlag> {
        let core = self.core.borrow();
        core.gained_focus()
    }

    /// Sets the focus to the given widget, but doesn't set
    /// lost and gained. This can be used to prevent any side
    /// effects that use the gained/lost state.
    ///
    /// This changes the focus and the gained/lost flags.
    /// If this ends up with the same widget as before
    /// gained and lost flags are not set.
    ///
    /// This will ignore the [Navigation] flag of the
    /// currently focused widget.
    ///
    /// You can also use a container-widget for this.
    /// It will set the focus to the first widget of the
    /// container.
    #[inline]
    #[deprecated(since = "2.3.0", note = "overengineered")]
    pub fn focus_no_lost(&self, widget_state: &'_ dyn HasFocus) {
        let core = self.core.borrow();
        let flag = widget_state.focus();
        focus_debug!(core, "focus no_lost {:?}", flag.name());
        core.focus(&flag, false);
    }

    /// This expels the focus from the given widget/container.
    ///
    /// This is sometimes useful to set the focus to **somewhere else**.
    /// This is especially useful when used for a popup-widget that will
    /// be hidden. Ensures there is still some widget with focus afterward.
    ///
    /// It will try to set the focus to the next widget or the
    /// next widget following the container. If this ends up within
    /// the given container it will set the focus to none.
    ///
    /// __Note__
    ///
    /// If you want to set the focus temporarily to a widget and later
    /// restore the current focus try [flip_focus](Focus::flip_focus).
    ///
    /// __Note__
    ///
    /// This function doesn't use the Navigation of the current widget.
    #[inline]
    pub fn expel_focus(&self, widget_state: &'_ dyn HasFocus) {
        let core = self.core.borrow();
        let flag = widget_state.focus();
        focus_debug!(core, "expel from widget {:?}", widget_state.focus().name());
        core.expel(&flag);
    }

    /// Dynamic change of the widget structure for a container widget.
    ///
    /// This is only necessary if your widget structure changes
    /// during event-handling, and you need a programmatic
    /// focus-change for the new structure.
    ///
    /// This resets the focus-flags of the removed container.
    pub fn remove_container(&mut self, container: &'_ dyn HasFocus) {
        let mut core = self.core.borrow_mut();
        let flag = container.focus();
        focus_debug!(core, "focus remove container {:?} ", flag.name());
        if core.is_container(&flag) {
            if let Some((cidx, _)) = core.container_index_of(&flag) {
                let removed = core.remove_container(cidx);

                let mut b = FocusBuilder::new_from_core(removed);
                b.cleanup();

                focus_debug!(core, "    -> removed");
            } else {
                focus_debug!(core, "ERR: invalid container?");
            }
        } else {
            focus_debug!(core, "    -> not a container flag");
        }
    }

    /// Dynamic change of the widget structure for a container.
    ///
    /// This is only necessary if your widget structure changes
    /// during event-handling, and you need a programmatic
    /// focus-change for the new structure.
    ///
    /// If the widget that currently has the focus is still
    /// part of the widget structure it keeps the focus.
    /// The focus-flags for all widgets that are no longer part
    /// of the widget structure are reset.
    pub fn update_container(&mut self, container: &'_ dyn HasFocus) {
        let mut core = self.core.borrow_mut();
        let flag = container.focus();
        focus_debug!(core, "focus update container {:?} ", flag.name());
        if core.is_container(&flag) {
            if let Some((cidx, range)) = core.container_index_of(&flag) {
                let removed = core.remove_container(cidx);

                let mut b = FocusBuilder::new_from_core(removed);
                b.widget(container);
                b.cleanup();
                core.insert_container(range.start, cidx, b.build_core());

                focus_debug!(core, "    -> updated");
            } else {
                focus_debug!(core, "ERR: invalid container?");
            }
        } else {
            focus_debug!(core, "    -> not a container flag");
        }
    }

    /// Dynamic change of the widget structure of a container.
    ///
    /// This is only necessary if your widget structure changes
    /// during event-handling, and you need a programmatic
    /// focus-change.
    ///
    /// This removes the widgets of one container and inserts
    /// the widgets of the other one in place.
    ///
    /// If the widget that currently has the focus is still
    /// part of the widget structure it keeps the focus.
    /// The focus-flags for all widgets that are no longer part
    /// of the widget structure are reset.
    pub fn replace_container(&mut self, container: &'_ dyn HasFocus, new: &'_ dyn HasFocus) {
        let mut core = self.core.borrow_mut();
        let flag = container.focus();
        focus_debug!(
            core,
            "focus replace container {:?} with {:?} ",
            flag.name(),
            new.focus().name()
        );
        if core.is_container(&flag) {
            if let Some((cidx, range)) = core.container_index_of(&flag) {
                let removed = core.remove_container(cidx);

                let mut b = FocusBuilder::new_from_core(removed);
                b.widget(new);
                b.cleanup();
                core.insert_container(range.start, cidx, b.build_core());

                focus_debug!(core, "    -> replaced");
            } else {
                focus_debug!(core, "ERR: invalid container");
            }
        } else {
            focus_debug!(core, "    -> not a container flag");
        }
    }

    /// Reset lost + gained flags.
    ///
    /// This is done automatically during event-handling.
    /// Lost+Gained flags will only be set while handling
    /// the original event that made the focus-change.
    /// The next event, whatever it is, will reset these flags.
    #[inline(always)]
    pub fn reset_lost_gained(&self) {
        let core = self.core.borrow();
        core.reset_lost_gained();
    }
}

impl HandleEvent<Event, Regular, Outcome> for Focus {
    #[inline(always)]
    fn handle(&mut self, event: &Event, _keymap: Regular) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                if self.next() {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                if self.prev() {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                }
            }
            _ => self.handle(event, MouseOnly),
        }
    }
}

impl HandleEvent<Event, MouseOnly, Outcome> for Focus {
    #[inline(always)]
    fn handle(&mut self, event: &Event, _keymap: MouseOnly) -> Outcome {
        match event {
            Event::Mouse(crossterm::event::MouseEvent {
                             column: c, row: r, ..
            }) => {
                self.find_obscured(*c, *r);
            }
            _ => {}
        };

        match event {
            ct_event!(mouse down Left for column, row) => {
                if self.focus_at(*column, *row) {
                    Outcome::Changed
                } else {
                    self.reset_lost_gained();
                    Outcome::Continue
                }
            }
            _ => {
                self.reset_lost_gained();
                Outcome::Continue
            }
        }
    }
}

/// Handle all events.
#[inline(always)]
pub fn handle_focus(focus: &mut Focus, event: &Event) -> Outcome {
    HandleEvent::handle(focus, event, Regular)
}
