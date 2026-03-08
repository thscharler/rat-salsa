use crate::core::FocusCore;
use crate::{FocusBuilder, FocusFlag, HasFocus, Navigation};
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular, crossterm, ct_event};
use ratatui_core::layout::Rect;
use ratatui_crossterm::crossterm::event::Event;
use std::ops::Range;

/// Focus deals with all focus-related issues.
///
/// Use [FocusBuilder] to construct the current Focus.
///
/// This is usually quick enough to do it for each event.
/// It has to be rebuilt if any area has changed, so
/// rebuilding it after a render() is fine.
#[derive(Default, Debug, Clone)]
pub struct Focus {
    pub(crate) last: FocusCore,
    pub(crate) core: FocusCore,
}

macro_rules! focus_debug {
    ($core:expr, $($arg:tt)+) => {
        if $core.log.get() {
            log::log!(log::Level::Debug, $($arg)+);
        }
    }
}

macro_rules! focus_fail {
    ($core:expr, $($arg:tt)+) => {
        if $core.log.get() {
            log::log!(log::Level::Debug, $($arg)+);
        }
        if $core.insta_panic.get() {
            panic!($($arg)+)
        }
    }
}

impl Focus {
    /// Writes a log for each operation.
    pub fn enable_log(&self) {
        self.core.log.set(true);
        self.last.log.set(true);
    }

    /// Writes a log for each operation.
    pub fn disable_log(&self) {
        self.core.log.set(false);
        self.last.log.set(false);
    }

    /// Enable insta-panic if any function is called
    /// with a widget that is not part of the Focus.
    pub fn enable_panic(&self) {
        self.core.insta_panic.set(true);
        self.last.insta_panic.set(true);
    }

    /// Disable insta-panic.
    pub fn disable_panic(&self) {
        self.core.insta_panic.set(false);
        self.last.insta_panic.set(false);
    }

    /// Sets the focus to the given widget and remembers
    /// the previous focused widget. If the focus is
    /// currently set to the given widget it sets the
    /// focus back to the previous widget.
    pub fn flip_focus(
        &self,
        widget_state: &'_ dyn HasFocus,
        flip_focus: &'_ mut Option<FocusFlag>,
    ) {
        focus_debug!(
            self.core,
            "flip-focus {:?} {:?}",
            widget_state.focus().name(),
            flip_focus
        );
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if flag.is_focused() {
                if let Some(flip_focus) = flip_focus {
                    self.focus(flip_focus);
                } else {
                    focus_fail!(self.core, "    => no previous widget");
                }
            } else {
                *flip_focus = self.focused();
                self.focus(&flag);
            }
        } else if self.core.is_container(&flag) {
            if flag.is_focused() {
                if let Some(flip_focus) = flip_focus {
                    self.focus(flip_focus);
                } else {
                    focus_fail!(self.core, "    => no previous widget");
                }
            } else {
                self.core.first_container(&flag);
            }
        } else {
            focus_fail!(self.core, "    => not a valid widget");
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
        focus_debug!(self.core, "focus {:?}", widget_state.focus().name());
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if let Some(n) = self.core.index_of(&flag) {
                self.core.focus_idx(n, true);
            } else {
                panic!("    => invalid widget?");
            }
        } else if self.core.is_container(&flag) {
            self.core.first_container(&flag);
        } else {
            focus_fail!(self.core, "    => not a valid widget");
        }
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
        let widget_state = self.core.find_widget_id(widget_id);
        focus_debug!(self.core, "focus {:?} -> {:?}", widget_id, widget_state);
        let Some(widget_state) = widget_state else {
            return;
        };

        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if let Some(n) = self.core.index_of(&flag) {
                self.core.focus_idx(n, true);
            } else {
                panic!("    => invalid widget");
            }
        } else if self.core.is_container(&flag) {
            self.core.first_container(&flag);
        } else {
            focus_fail!(self.core, "    => not a valid widget");
        }
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
        focus_debug!(self.core, "focus first");
        self.core.first();
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
        focus_debug!(self.core, "focus none");
        self.core.none();
        focus_debug!(self.core, "    -> done");
    }

    /// This widget will have the focus, but it is not
    /// yet part of the focus cycle. And the focus cycle
    /// can't be properly rebuilt at this point.
    ///
    /// If the widget *is* part of the focus this will do nothing.
    ///
    /// If the widget is a container, it will just set
    /// the container-flag. If you want to set a future widget
    /// and its container, call future() for the widget first,
    /// then the container.
    #[inline(always)]
    pub fn future(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(self.core, "focus {:?}", widget_state.focus().name());
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            focus_fail!(
                self.core,
                "    => widget is part of focus. use focus() instead"
            );
        } else if self.core.is_container(&flag) {
            focus_debug!(self.core, "future container");
            let had_focus = flag.get();
            flag.set(true);
            if !had_focus {
                flag.set_gained(true);
                flag.call_on_gained();
            }
            focus_debug!(self.core, "    -> done");
        } else {
            focus_debug!(self.core, "future focus");
            self.core.none();
            flag.set(true);
            flag.set_gained(true);
            flag.call_on_gained();
            focus_debug!(self.core, "    -> done");
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
        focus_debug!(self.core, "focus at {},{}", col, row);
        match self.navigation() {
            Some(Navigation::Lock) => {
                focus_debug!(self.core, "    -> locked");
                false
            }
            _ => self.core.focus_at(col, row),
        }
    }

    /// Reset the mouse-focus flag to __true__.
    #[inline(always)]
    pub fn reset_mouse_focus(&self) -> bool {
        self.core.reset_mouse_focus()
    }

    /// Set the mouse-focus to the given position.  
    ///
    /// The top-most widget with a matching area will have
    /// its mouse_focus flag set. Any containers
    /// with an associated area that matches, will get
    /// their mouse_focus flag set too.
    #[inline(always)]
    pub fn mouse_focus(&self, col: u16, row: u16) -> bool {
        focus_debug!(self.core, "mouse-focus {} {}", col, row);
        self.core.mouse_focus(col, row)
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
        match self.navigation() {
            None => {
                self.first();
                true
            }
            Some(Navigation::Leave | Navigation::ReachLeaveBack | Navigation::Regular) => {
                focus_debug!(
                    self.core,
                    "next after {:?}",
                    self.core
                        .focused()
                        .map(|v| v.name())
                        .unwrap_or("None".into())
                );
                self.core.next()
            }
            v => {
                focus_debug!(
                    self.core,
                    "next after {:?}, but navigation says {:?}",
                    self.core
                        .focused()
                        .map(|v| v.name().to_string())
                        .unwrap_or("None".into()),
                    v
                );
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
        match self.navigation() {
            None => {
                self.first();
                true
            }
            Some(Navigation::Leave | Navigation::ReachLeaveFront | Navigation::Regular) => {
                focus_debug!(
                    self.core,
                    "prev before {:?}",
                    self.core
                        .focused()
                        .map(|v| v.name().to_string())
                        .unwrap_or("None".into())
                );
                self.core.prev()
            }
            v => {
                focus_debug!(
                    self.core,
                    "prev before {:?}, but navigation says {:?}",
                    self.core
                        .focused()
                        .map(|v| v.name().to_string())
                        .unwrap_or("None".into()),
                    v
                );
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
    pub fn next_force(&self) -> bool {
        match self.navigation() {
            None => {
                self.first();
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
                    self.core,
                    "force next after {:?}",
                    self.core.focused().map(|v| v.name().to_string())
                );
                self.core.next()
            }
            v => {
                focus_debug!(
                    self.core,
                    "force next after {:?}, but navigation says {:?}",
                    self.core.focused().map(|v| v.name().to_string()),
                    v
                );
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
    pub fn prev_force(&self) -> bool {
        match self.navigation() {
            None => {
                self.first();
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
                    self.core,
                    "force prev before {:?}",
                    self.core.focused().map(|v| v.name().to_string())
                );
                self.core.prev()
            }
            v => {
                focus_debug!(
                    self.core,
                    "force prev before {:?}, but navigation says {:?}",
                    self.core.focused().map(|v| v.name().to_string()),
                    v
                );
                false
            }
        }
    }

    /// Is this widget part of this focus-cycle?
    #[inline(always)]
    pub fn is_valid_widget(&self, widget_state: &'_ dyn HasFocus) -> bool {
        self.core.is_widget(&widget_state.focus())
    }

    /// Is this a container that is part of this focus-cycle?
    #[inline(always)]
    pub fn is_valid_container(&self, widget_state: &'_ dyn HasFocus) -> bool {
        self.core.is_container(&widget_state.focus())
    }

    /// Returns the focused widget as FocusFlag.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn focused(&self) -> Option<FocusFlag> {
        self.core.focused()
    }

    /// Returns the focused widget as widget-id.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn focused_widget_id(&self) -> Option<usize> {
        self.core.focused().map(|v| v.id())
    }

    /// Returns the debug name of the focused widget.
    #[inline(always)]
    pub fn focused_name(&self) -> Option<String> {
        self.core.focused().map(|v| v.name().to_string())
    }

    /// Returns the [Navigation] flag for the focused widget.
    #[inline(always)]
    pub fn navigation(&self) -> Option<Navigation> {
        self.core.navigation()
    }

    /// Returns the widget that lost the focus as FocusFlag.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn lost_focus(&self) -> Option<FocusFlag> {
        self.core.lost_focus()
    }

    /// Returns the widget that gained the focus as FocusFlag.
    ///
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    #[inline(always)]
    pub fn gained_focus(&self) -> Option<FocusFlag> {
        self.core.gained_focus()
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
    pub fn focus_no_lost(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(self.core, "focus no_lost {:?}", widget_state.focus().name());
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if let Some(n) = self.core.index_of(&flag) {
                self.core.focus_idx(n, false);
            } else {
                panic!("    => invalid widget");
            }
        } else if self.core.is_container(&flag) {
            self.core.first_container(&flag);
        } else {
            focus_fail!(self.core, "    => not a valid widget");
        }
    }

    /// This expels the focus from the given widget/container.
    ///
    /// This is sometimes useful to set the focus to **somewhere else**.
    /// This is especially useful when used for a container-widget that will
    /// be hidden. Ensures there is still some widget with focus afterward.
    ///
    /// It will try to set the focus to the next widget or the
    /// next widget following the container. If this ends up within
    /// the given container it will set the focus to none.
    ///
    /// This function doesn't use the Navigation of the current widget.
    #[inline]
    pub fn expel_focus(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(
            self.core,
            "expel from widget {:?}",
            widget_state.focus().name()
        );
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if self.core.index_of(&flag).is_some() {
                if widget_state.is_focused() {
                    self.core.next();
                    if widget_state.is_focused() {
                        focus_debug!(self.core, "    -> no other focus, cleared");
                        flag.clear();
                    } else {
                        focus_debug!(self.core, "    -> expelled");
                    }
                } else {
                    focus_debug!(self.core, "    => widget not focused");
                }
            } else {
                panic!("    => invalid widget");
            }
        } else if self.core.is_container(&flag) {
            if flag.is_focused() {
                self.core.expel_container(flag);
            } else {
                focus_debug!(self.core, "    => container not focused");
            }
        } else {
            focus_fail!(self.core, "    => not a valid widget");
        }
    }

    /// Dynamic change of the widget structure for a container widget.
    ///
    /// This is only necessary if your widget structure changes
    /// during event-handling, and you need a programmatic
    /// focus-change for the new structure.
    ///
    /// This resets the focus-flags of the removed container.
    pub fn remove_container(&mut self, container: &'_ dyn HasFocus) {
        focus_debug!(
            self.core,
            "focus remove container {:?} ",
            container.focus().name()
        );
        let flag = container.focus();
        if self.core.is_container(&flag) {
            if let Some((cidx, _)) = self.core.container_index_of(&flag) {
                self.core.remove_container(cidx).reset();
                focus_debug!(self.core, "    -> removed");
            } else {
                panic!("    => invalid container?");
            }
        } else {
            focus_fail!(self.core, "    => no container flag");
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
        focus_debug!(
            self.core,
            "focus update container {:?} ",
            container.focus().name()
        );
        let flag = container.focus();
        if self.core.is_container(&flag) {
            if let Some((cidx, range)) = self.core.container_index_of(&flag) {
                let removed = self.core.remove_container(cidx);

                let mut b = FocusBuilder::new(Some(Focus {
                    last: Default::default(),
                    core: removed,
                }));
                b.widget(container);
                let insert = b.build();

                self.core.insert_container(range.start, cidx, insert.core);

                focus_debug!(self.core, "    -> updated");
            } else {
                panic!("    => invalid container?");
            }
        } else {
            focus_fail!(self.core, "    => no container flag");
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
        focus_debug!(
            self.core,
            "focus replace container {:?} with {:?} ",
            container.focus().name(),
            new.focus().name()
        );
        let flag = container.focus();
        if self.core.is_container(&flag) {
            if let Some((cidx, range)) = self.core.container_index_of(&flag) {
                let removed = self.core.remove_container(cidx);

                let mut b = FocusBuilder::new(Some(Focus {
                    last: Default::default(),
                    core: removed,
                }));
                b.widget(new);
                let insert = b.build();

                self.core.insert_container(range.start, cidx, insert.core);

                focus_debug!(self.core, "    -> replaced");
            } else {
                panic!("    => invalid container");
            }
        } else {
            focus_fail!(self.core, "    => no container flag");
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
        self.core.reset_lost_gained();
    }

    /// Debug destructuring.
    #[allow(clippy::type_complexity)]
    pub fn clone_destruct(
        &self,
    ) -> (
        Vec<FocusFlag>,
        Vec<bool>,
        Vec<(Rect, u16)>,
        Vec<Navigation>,
        Vec<(FocusFlag, (Rect, u16), Range<usize>)>,
    ) {
        self.core.clone_destruct()
    }
}

impl HandleEvent<Event, Regular, Outcome> for Focus {
    #[inline(always)]
    fn handle(&mut self, event: &Event, _keymap: Regular) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                focus_debug!(
                    self.core,
                    "Tab {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                let r = if self.next() {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                };
                focus_debug!(
                    self.core,
                    "    -> {:?} {:?}",
                    r,
                    self.focused().map(|v| v.name().to_string())
                );
                r
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                focus_debug!(
                    self.core,
                    "BackTab {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                let r = if self.prev() {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                };
                focus_debug!(
                    self.core,
                    "    -> {:?} {:?}",
                    r,
                    self.focused().map(|v| v.name().to_string())
                );
                r
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
                kind: crossterm::event::MouseEventKind::Drag(_),
                ..
            }) => {
                self.reset_mouse_focus();
            }
            Event::Mouse(crossterm::event::MouseEvent {
                kind:
                    crossterm::event::MouseEventKind::Moved
                    | crossterm::event::MouseEventKind::Down(_)
                    | crossterm::event::MouseEventKind::Up(_)
                    | crossterm::event::MouseEventKind::ScrollDown
                    | crossterm::event::MouseEventKind::ScrollUp
                    | crossterm::event::MouseEventKind::ScrollLeft
                    | crossterm::event::MouseEventKind::ScrollRight,
                column: c,
                row: r,
                ..
            }) => {
                self.mouse_focus(*c, *r);
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
