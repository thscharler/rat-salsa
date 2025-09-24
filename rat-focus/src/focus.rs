use crate::focus::core::FocusCore;
use crate::{FocusFlag, HasFocus, Navigation};
pub use core::FocusBuilder;
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular, ct_event};
use ratatui::layout::Rect;
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
    last: FocusCore,
    core: FocusCore,
}

macro_rules! focus_debug {
    ($log:expr, $($arg:tt)+) => {
        if $log.get() {
            log::log!(log::Level::Debug, $($arg)+);
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
        focus_debug!(self.core.log, "focus {:?}", widget_state.focus().name());
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if let Some(n) = self.core.index_of(&flag) {
                self.core.focus_idx(n, true);
            } else {
                focus_debug!(self.core.log, "    => widget not found");
            }
        } else if self.core.is_container(&flag) {
            self.core.first_container(&flag);
        } else {
            focus_debug!(self.core.log, "    => not a valid widget");
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
        focus_debug!(self.core.log, "focus {:?} -> {:?}", widget_id, widget_state);
        let Some(widget_state) = widget_state else {
            return;
        };

        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if let Some(n) = self.core.index_of(&flag) {
                self.core.focus_idx(n, true);
            } else {
                focus_debug!(self.core.log, "    => widget not found");
            }
        } else if self.core.is_container(&flag) {
            self.core.first_container(&flag);
        } else {
            focus_debug!(self.core.log, "    => not a valid widget");
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
        focus_debug!(self.core.log, "focus first");
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
        focus_debug!(self.core.log, "focus none");
        self.core.none();
        focus_debug!(self.core.log, "    -> done");
    }

    /// This widget will have the focus, but it is not
    /// yet part of the focus cycle. And the focus cycle
    /// can't be properly rebuilt at this point.
    #[inline(always)]
    pub fn future(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(self.core.log, "future focus");
        self.core.none();
        widget_state.focus().set(true);
        widget_state.focus().set_gained(true);
        focus_debug!(self.core.log, "    -> done");
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
        focus_debug!(self.core.log, "focus at {},{}", col, row);
        match self.navigation() {
            Some(Navigation::Lock) => {
                focus_debug!(self.core.log, "    -> locked");
                false
            }
            _ => self.core.focus_at(col, row),
        }
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
                    self.core.log,
                    "next after {:?}",
                    self.core
                        .focused()
                        .map(|v| v.name().to_string())
                        .unwrap_or("None".into())
                );
                self.core.next()
            }
            v => {
                focus_debug!(
                    self.core.log,
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
                    self.core.log,
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
                    self.core.log,
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
                    self.core.log,
                    "force next after {:?}",
                    self.core.focused().map(|v| v.name().to_string())
                );
                self.core.next()
            }
            v => {
                focus_debug!(
                    self.core.log,
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
                    self.core.log,
                    "force prev before {:?}",
                    self.core.focused().map(|v| v.name().to_string())
                );
                self.core.prev()
            }
            v => {
                focus_debug!(
                    self.core.log,
                    "force prev before {:?}, but navigation says {:?}",
                    self.core.focused().map(|v| v.name().to_string()),
                    v
                );
                false
            }
        }
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
        focus_debug!(
            self.core.log,
            "focus no_lost {:?}",
            widget_state.focus().name()
        );
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if let Some(n) = self.core.index_of(&flag) {
                self.core.focus_idx(n, false);
            } else {
                focus_debug!(self.core.log, "    => widget not found");
            }
        } else if self.core.is_container(&flag) {
            self.core.first_container(&flag);
        } else {
            focus_debug!(self.core.log, "    => not a valid widget");
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
            self.core.log,
            "expel from widget {:?}",
            widget_state.focus().name()
        );
        let flag = widget_state.focus();
        if self.core.is_widget(&flag) {
            if self.core.index_of(&flag).is_some() {
                if widget_state.is_focused() {
                    self.core.next();
                    if widget_state.is_focused() {
                        focus_debug!(self.core.log, "    -> no other focus, cleared");
                        flag.clear();
                    } else {
                        focus_debug!(self.core.log, "    -> expelled");
                    }
                } else {
                    focus_debug!(self.core.log, "    => widget not focused");
                }
            } else {
                focus_debug!(self.core.log, "    => widget not found");
            }
        } else if self.core.is_container(&flag) {
            if flag.is_focused() {
                self.core.expel_container(flag);
            } else {
                focus_debug!(self.core.log, "    => container not focused");
            }
        } else {
            focus_debug!(self.core.log, "    => not a valid widget");
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
            self.core.log,
            "focus remove container {:?} ",
            container.focus().name()
        );
        let flag = container.focus();
        if self.core.is_container(&flag) {
            if let Some((cidx, _)) = self.core.container_index_of(&flag) {
                self.core.remove_container(cidx).reset();
                focus_debug!(self.core.log, "    -> removed");
            } else {
                focus_debug!(self.core.log, "    => container not found");
            }
        } else {
            focus_debug!(self.core.log, "    => no container flag");
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
            self.core.log,
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

                focus_debug!(self.core.log, "    -> updated");
            } else {
                focus_debug!(self.core.log, "    => container not found");
            }
        } else {
            focus_debug!(self.core.log, "    => no container flag");
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
            self.core.log,
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

                focus_debug!(self.core.log, "    -> replaced");
            } else {
                focus_debug!(self.core.log, "    => container not found");
            }
        } else {
            focus_debug!(self.core.log, "    => no container flag");
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

mod core {
    use crate::{Focus, FocusFlag, HasFocus, Navigation};
    use fxhash::FxBuildHasher;
    use ratatui::layout::Rect;
    use std::cell::Cell;
    use std::collections::HashSet;
    use std::ops::Range;

    /// Builder for the Focus.
    #[derive(Debug, Default)]
    pub struct FocusBuilder {
        last: FocusCore,

        log: Cell<bool>,

        // base z value.
        // starting a container adds the z-value of the container
        // to the z_base. closing the container subtracts from the
        // z_base. any widgets added in between have a z-value
        // of z_base + widget z-value.
        //
        // this enables clean stacking of containers/widgets.
        z_base: u16,

        // new core
        focus_ids: HashSet<usize, FxBuildHasher>,
        focus_flags: Vec<FocusFlag>,
        duplicate: Vec<bool>,
        areas: Vec<(Rect, u16)>,
        navigable: Vec<Navigation>,
        container_ids: HashSet<usize, FxBuildHasher>,
        containers: Vec<(Container, Range<usize>)>,
    }

    impl FocusBuilder {
        /// Create a new FocusBuilder.
        ///
        /// This can take the previous Focus and ensures that
        /// widgets that are no longer part of the focus list
        /// have their focus-flag cleared.
        ///
        /// It will also recycle the storage of the old Focus.
        pub fn new(last: Option<Focus>) -> FocusBuilder {
            if let Some(mut last) = last {
                // clear any data but retain the allocation.
                last.last.clear();

                Self {
                    last: last.core,
                    log: Default::default(),
                    z_base: 0,
                    focus_ids: last.last.focus_ids,
                    focus_flags: last.last.focus_flags,
                    duplicate: last.last.duplicate,
                    areas: last.last.areas,
                    navigable: last.last.navigable,
                    container_ids: last.last.container_ids,
                    containers: last.last.containers,
                }
            } else {
                Self {
                    last: FocusCore::default(),
                    log: Default::default(),
                    z_base: Default::default(),
                    focus_ids: Default::default(),
                    focus_flags: Default::default(),
                    duplicate: Default::default(),
                    areas: Default::default(),
                    navigable: Default::default(),
                    container_ids: Default::default(),
                    containers: Default::default(),
                }
            }
        }

        /// Shortcut for building the focus for a container
        /// that implements [HasFocus]().
        ///
        /// This creates a fresh Focus.
        ///
        /// __See__
        ///
        /// Use [rebuild](FocusBuilder::rebuild_for) if you want to ensure that widgets
        /// that are no longer in the widget structure have their
        /// focus flag reset properly. If you don't have
        /// some logic to conditionally add widgets to the focus,
        /// this function is probably fine.
        pub fn build_for(container: &dyn HasFocus) -> Focus {
            let mut b = FocusBuilder::new(None);
            b.widget(container);
            b.build()
        }

        /// Shortcut function for building the focus for a container
        /// that implements [HasFocus]()
        ///
        /// This takes the old Focus and reuses most of its allocations.
        /// It also ensures that any widgets no longer in the widget structure
        /// have their focus-flags reset.
        pub fn rebuild_for(container: &dyn HasFocus, old: Option<Focus>) -> Focus {
            let mut b = FocusBuilder::new(old);
            b.widget(container);
            b.build()
        }

        /// Do some logging of the build.
        pub fn enable_log(&self) {
            self.log.set(true);
        }

        /// Do some logging of the build.
        pub fn disable_log(&self) {
            self.log.set(false);
        }

        /// Add a widget by calling its `build` function.
        pub fn widget(&mut self, widget: &dyn HasFocus) -> &mut Self {
            widget.build(self);
            self
        }

        /// Add a widget by calling its build function.
        ///
        /// This tries to override the default navigation
        /// for the given widget. This will fail if the
        /// widget is a container. It may also fail
        /// for other reasons. Depends on the widget.
        ///
        /// Enable log to check.
        #[allow(clippy::collapsible_else_if)]
        pub fn widget_navigate(
            &mut self,
            widget: &dyn HasFocus,
            navigation: Navigation,
        ) -> &mut Self {
            widget.build(self);

            let widget_flag = widget.focus();
            // override navigation for the widget
            if let Some(idx) = self.focus_flags.iter().position(|v| *v == widget_flag) {
                focus_debug!(
                    self.log,
                    "override navigation for {:?} with {:?}",
                    widget_flag,
                    navigation
                );

                self.navigable[idx] = navigation;
            } else {
                if self.container_ids.contains(&widget_flag.widget_id()) {
                    focus_debug!(
                        self.log,
                        "FAIL to override navigation for {:?}. This is a container.",
                        widget_flag,
                    );
                } else {
                    focus_debug!(
                        self.log,
                        "FAIL to override navigation for {:?}. Widget doesn't use this focus-flag",
                        widget_flag,
                    );
                }
            }

            self
        }

        /// Add a bunch of widgets.
        #[inline]
        pub fn widgets<const N: usize>(&mut self, widgets: [&dyn HasFocus; N]) -> &mut Self {
            for widget in widgets {
                widget.build(self);
            }
            self
        }

        /// Start a container widget. Must be matched with
        /// the equivalent [end](Self::end). Uses focus(), area() and
        /// z_area() of the given container. navigable() is
        /// currently not used, just leave it at the default.
        ///
        /// __Attention__
        ///
        /// Use the returned value when calling [end](Self::end).
        ///
        /// __Panic__
        ///
        /// Panics if the same container-flag is added twice.
        #[must_use]
        pub fn start(&mut self, container: &dyn HasFocus) -> FocusFlag {
            self.start_with_flags(container.focus(), container.area(), container.area_z())
        }

        /// End a container widget.
        pub fn end(&mut self, tag: FocusFlag) {
            focus_debug!(self.log, "end container {:?}", tag);
            assert!(self.container_ids.contains(&tag.widget_id()));

            for (c, r) in self.containers.iter_mut().rev() {
                if c.container_flag != tag {
                    if !c.complete {
                        panic!("FocusBuilder: Unclosed container {:?}", c.container_flag);
                    }
                } else {
                    r.end = self.focus_flags.len();
                    c.complete = true;

                    focus_debug!(self.log, "container range {:?}", r);

                    self.z_base -= c.delta_z;

                    break;
                }
            }
        }

        /// Directly add the given widget's flags. Doesn't call
        /// build() instead it uses focus(), etc. and appends a single widget.
        ///
        /// This is intended to be used when __implementing__
        /// HasFocus::build() for a widget.
        ///
        /// In all other situations it's better to use [widget()](FocusBuilder::widget).
        ///
        /// __Panic__
        ///
        /// Panics if the same focus-flag is added twice.
        pub fn leaf_widget(&mut self, widget: &dyn HasFocus) -> &mut Self {
            self.widget_with_flags(
                widget.focus(),
                widget.area(),
                widget.area_z(),
                widget.navigable(),
            );
            self
        }

        /// Manually add a widgets flags.
        ///
        /// This is intended to be used when __implementing__
        /// HasFocus::build() for a widget.
        ///
        /// In all other situations it's better to use [widget()](FocusBuilder::widget).
        ///
        /// __Panic__
        ///
        /// Panics if the same focus-flag is added twice.
        /// Except it is allowable to add the flag a second time with
        /// Navigation::Mouse or Navigation::None
        pub fn widget_with_flags(
            &mut self,
            focus: FocusFlag,
            area: Rect,
            area_z: u16,
            navigable: Navigation,
        ) {
            let duplicate = self.focus_ids.contains(&focus.widget_id());

            // there can be a second entry for the same focus-flag
            // if it is only for mouse interactions.
            if duplicate {
                assert!(matches!(navigable, Navigation::Mouse | Navigation::None))
            }

            focus_debug!(self.log, "widget {:?}", focus);

            self.focus_ids.insert(focus.widget_id());
            self.focus_flags.push(focus);
            self.duplicate.push(duplicate);
            self.areas.push((area, self.z_base + area_z));
            self.navigable.push(navigable);
        }

        /// Start a container widget.
        ///
        /// Returns the FocusFlag of the container. This flag must
        /// be used to close the container with [end](Self::end).
        ///
        /// __Panic__
        ///
        /// Panics if the same container-flag is added twice.
        #[must_use]
        pub fn start_with_flags(
            &mut self,
            container_flag: FocusFlag,
            area: Rect,
            area_z: u16,
        ) -> FocusFlag {
            focus_debug!(self.log, "start container {:?}", container_flag);

            // no duplicates allowed for containers.
            assert!(!self.container_ids.contains(&container_flag.widget_id()));

            self.z_base += area_z;

            let len = self.focus_flags.len();
            self.container_ids.insert(container_flag.widget_id());
            self.containers.push((
                Container {
                    container_flag: container_flag.clone(),
                    area: (area, self.z_base),
                    delta_z: area_z,
                    complete: false,
                },
                len..len,
            ));

            container_flag
        }

        /// Build the Focus.
        ///
        /// If the previous Focus is known, this will also
        /// reset the FocusFlag for any widget no longer part of
        /// the Focus.
        pub fn build(mut self) -> Focus {
            // cleanup outcasts.
            for v in &self.last.focus_flags {
                if !self.focus_ids.contains(&v.widget_id()) {
                    v.clear();
                }
            }
            for (v, _) in &self.last.containers {
                let have_container = self
                    .containers
                    .iter()
                    .any(|(c, _)| v.container_flag == c.container_flag);
                if !have_container {
                    v.container_flag.clear();
                }
            }
            self.last.clear();

            // check new tree.
            for (c, _) in self.containers.iter_mut().rev() {
                if !c.complete {
                    panic!("FocusBuilder: Unclosed container {:?}", c.container_flag);
                }
            }

            let log = self.last.log.get();

            Focus {
                last: self.last,
                core: FocusCore {
                    log: Cell::new(log),
                    focus_ids: self.focus_ids,
                    focus_flags: self.focus_flags,
                    duplicate: self.duplicate,
                    areas: self.areas,
                    navigable: self.navigable,
                    container_ids: self.container_ids,
                    containers: self.containers,
                },
            }
        }
    }

    /// Struct for the data of the focus-container itself.
    #[derive(Debug, Clone)]
    struct Container {
        /// Summarizes all the contained FocusFlags.
        /// If any of them has the focus set, this will be set too.
        /// This can help if you build compound widgets.
        container_flag: FocusFlag,
        /// Area for the whole compound.
        /// Contains the area and a z-value.
        area: (Rect, u16),
        /// Delta Z value compared to the enclosing container.
        delta_z: u16,
        /// Flag for construction.
        complete: bool,
    }

    /// Focus core.
    #[derive(Debug, Default, Clone)]
    pub(super) struct FocusCore {
        /// Focus logging
        pub(super) log: Cell<bool>,

        /// List of focus-ids.
        focus_ids: HashSet<usize, FxBuildHasher>,
        /// List of flags.
        focus_flags: Vec<FocusFlag>,
        /// Is the flag the primary flag, or just a duplicate
        /// to allow for multiple areas.
        duplicate: Vec<bool>,
        /// Areas for each widget.
        /// Contains the area and a z-value for the area.
        areas: Vec<(Rect, u16)>,
        /// Keyboard navigable
        navigable: Vec<Navigation>,
        /// List of focus-ids.
        container_ids: HashSet<usize, FxBuildHasher>,
        /// List of containers and their dependencies.
        /// Range here is a range in the vecs above. The ranges are
        /// all disjoint or completely contained within one other.
        /// No criss-cross intersections.
        containers: Vec<(Container, Range<usize>)>,
    }

    impl FocusCore {
        /// Clear.
        pub(super) fn clear(&mut self) {
            self.focus_ids.clear();
            self.focus_flags.clear();
            self.duplicate.clear();
            self.areas.clear();
            self.navigable.clear();
            self.container_ids.clear();
            self.containers.clear();
        }

        /// Find the FocusFlag by widget_id
        pub(super) fn find_widget_id(&self, widget_id: usize) -> Option<FocusFlag> {
            self.focus_flags
                .iter()
                .find(|v| widget_id == v.widget_id())
                .cloned()
        }

        /// Is a widget?
        pub(super) fn is_widget(&self, focus_flag: &FocusFlag) -> bool {
            self.focus_ids.contains(&focus_flag.widget_id())
        }

        /// Find the first occurrence of the given focus-flag.
        pub(super) fn index_of(&self, focus_flag: &FocusFlag) -> Option<usize> {
            self.focus_flags
                .iter()
                .enumerate()
                .find(|(_, f)| *f == focus_flag)
                .map(|(idx, _)| idx)
        }

        /// Is a container
        pub(super) fn is_container(&self, focus_flag: &FocusFlag) -> bool {
            self.container_ids.contains(&focus_flag.widget_id())
        }

        /// Find the given container-flag in the list of sub-containers.
        pub(super) fn container_index_of(
            &self,
            container_flag: &FocusFlag,
        ) -> Option<(usize, Range<usize>)> {
            self.containers
                .iter()
                .enumerate()
                .find(|(_, (c, _))| &c.container_flag == container_flag)
                .map(|(idx, (_, range))| (idx, range.clone()))
        }

        /// Append a container.
        ///
        /// * pos - position inside the focus-flags
        /// * cpos - position inside the sub-containers
        pub(super) fn insert_container(
            &mut self,
            idx: usize,
            cidx: usize,
            mut container: FocusCore,
        ) {
            for c in &self.focus_flags {
                for d in &container.focus_flags {
                    assert_ne!(c, d);
                }
            }

            // range for the data of the added container.
            let start = idx;
            let end = idx + container.focus_flags.len();

            self.focus_ids.extend(container.focus_ids.iter());
            self.focus_flags
                .splice(idx..idx, container.focus_flags.drain(..));
            self.duplicate
                .splice(idx..idx, container.duplicate.drain(..));
            self.areas.splice(idx..idx, container.areas.drain(..));
            self.navigable
                .splice(idx..idx, container.navigable.drain(..));

            // expand current ranges
            for (_, r) in &mut self.containers {
                *r = Self::expand(start..end, r.clone());
            }
            // shift inserted ranges into place
            self.containers.splice(
                cidx..cidx,
                container
                    .containers
                    .drain(..)
                    .map(|(c, r)| (c, Self::shift(start, r))),
            );
            self.container_ids.extend(container.container_ids.iter());
        }

        /// Remove everything for the given container.
        /// Return the extracted values as FocusCore.
        pub(super) fn remove_container(&mut self, cidx: usize) -> FocusCore {
            let crange = self.containers[cidx].1.clone();

            // remove
            let focus_flags = self.focus_flags.drain(crange.clone()).collect::<Vec<_>>();
            let mut focus_ids = HashSet::<_, FxBuildHasher>::default();
            for f in focus_flags.iter() {
                self.focus_ids.remove(&f.widget_id());
                focus_ids.insert(f.widget_id());
            }
            let duplicate = self.duplicate.drain(crange.clone()).collect::<Vec<_>>();
            let areas = self.areas.drain(crange.clone()).collect::<Vec<_>>();
            let navigable = self.navigable.drain(crange.clone()).collect::<Vec<_>>();
            let sub_containers = self
                .containers
                .iter()
                .filter(|(_, r)| r.start >= crange.start && r.end <= crange.end)
                .cloned()
                .collect::<Vec<_>>();
            // remove the container and all sub-containers in the range.
            self.containers
                .retain(|(_, r)| !(r.start >= crange.start && r.end <= crange.end));
            let mut sub_container_ids: HashSet<usize, FxBuildHasher> = HashSet::default();
            for (sc, _) in sub_containers.iter() {
                self.container_ids.remove(&sc.container_flag.widget_id());
                sub_container_ids.insert(sc.container_flag.widget_id());
            }

            // adjust the remaining sub-containers
            for (_, r) in &mut self.containers {
                *r = Self::shrink(crange.start..crange.end, r.clone());
            }

            FocusCore {
                log: Cell::new(false),
                focus_ids,
                focus_flags,
                duplicate,
                areas,
                navigable,
                container_ids: sub_container_ids,
                containers: sub_containers,
            }
        }

        // shift the ranges left by n
        fn shift(n: usize, range: Range<usize>) -> Range<usize> {
            range.start + n..range.end + n
        }

        // expand the range caused by insert
        fn expand(insert: Range<usize>, mut range: Range<usize>) -> Range<usize> {
            let len = insert.end - insert.start;

            if range.start >= insert.start {
                range.start += len;
            }
            if range.end > insert.start {
                range.end += len;
            }
            range
        }

        // shrink the range caused by remove
        fn shrink(remove: Range<usize>, mut range: Range<usize>) -> Range<usize> {
            let len = remove.end - remove.start;

            if range.start < remove.start {
                // leave
            } else if range.start >= remove.start && range.start <= remove.end {
                range.start = remove.start;
            } else {
                range.start -= len;
            }

            if range.end < remove.start {
                // leave
            } else if range.end >= remove.start && range.end <= remove.end {
                range.end = remove.start;
            } else {
                range.end -= len;
            }

            range
        }

        /// Reset the flags for a new round.
        /// set_lost - copy the current focus to the lost flag.
        fn __start_change(&self, set_lost: bool) {
            for (f, duplicate) in self.focus_flags.iter().zip(self.duplicate.iter()) {
                if *duplicate {
                    // skip duplicates
                    continue;
                }
                if set_lost {
                    f.set_lost(f.get());
                } else {
                    f.set_lost(false);
                }
                f.set_gained(false);
                f.set(false);
            }
        }

        /// Set the focus to this index. Doesn't touch
        /// other flags.
        fn __focus(&self, n: usize, set_lost: bool) -> bool {
            if let Some(f) = self.focus_flags.get(n) {
                focus_debug!(self.log, "    -> focus {}:{:?}", n, f.name());
                f.set(true);
                if set_lost {
                    if f.lost() {
                        // new focus same as old.
                        // reset lost + gained
                        f.set_lost(false);
                        f.set_gained(false);
                        false
                    } else {
                        f.set_gained(true);
                        true
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }

        /// Accumulate all container flags.
        fn __accumulate(&self) {
            for (f, r) in &self.containers {
                let mut any_gained = false;
                let mut any_lost = false;
                let mut any_focused = false;

                for idx in r.clone() {
                    any_gained |= self.focus_flags[idx].gained();
                    any_lost |= self.focus_flags[idx].lost();
                    any_focused |= self.focus_flags[idx].get();
                }

                f.container_flag.set(any_focused);
                f.container_flag.set_lost(any_lost && !any_gained);
                f.container_flag.set_gained(any_gained && !any_lost);
            }
        }

        /// Reset all lost+gained+focus flags.
        pub(super) fn reset(&self) {
            for f in self.focus_flags.iter() {
                f.set(false);
                f.set_lost(false);
                f.set_gained(false);
            }
            for (f, _) in self.containers.iter() {
                f.container_flag.set(false);
                f.container_flag.set_gained(false);
                f.container_flag.set_lost(false);
            }
        }

        /// Reset all lost+gained flags.
        pub(super) fn reset_lost_gained(&self) {
            for f in self.focus_flags.iter() {
                f.set_lost(false);
                f.set_gained(false);
            }
            for (f, _) in self.containers.iter() {
                f.container_flag.set_gained(false);
                f.container_flag.set_lost(false);
            }
        }

        /// Set the initial focus.
        pub(super) fn first(&self) {
            if let Some(n) = self.first_navigable(0) {
                self.__start_change(true);
                self.__focus(n, true);
                self.__accumulate();
            } else {
                focus_debug!(self.log, "    -> no navigable widget");
            }
        }

        /// Clear the focus.
        pub(super) fn none(&self) {
            self.__start_change(true);
            self.__accumulate();
        }

        /// Set the initial focus.
        pub(super) fn first_container(&self, container: &FocusFlag) {
            if let Some((_idx, range)) = self.container_index_of(container) {
                if let Some(n) = self.first_navigable(range.start) {
                    if n < range.end {
                        self.__start_change(true);
                        self.__focus(n, true);
                        self.__accumulate();
                    } else {
                        focus_debug!(self.log, "    -> no navigable widget for container");
                    }
                } else {
                    focus_debug!(self.log, "    -> no navigable widget");
                }
            } else {
                focus_debug!(self.log, "    => container not found");
            }
        }

        /// Set the focus at the given index.
        pub(super) fn focus_idx(&self, n: usize, set_lost: bool) {
            self.__start_change(set_lost);
            self.__focus(n, set_lost);
            self.__accumulate();
        }

        /// Set the focus at the given screen position.
        /// Traverses the list to find the matching widget.
        /// Checks the area and the z-areas.
        pub(super) fn focus_at(&self, col: u16, row: u16) -> bool {
            let pos = (col, row).into();

            enum ZOrder {
                Widget(usize),
                Container(usize),
            }

            // find any matching areas
            let mut z_order: Option<(ZOrder, u16)> = None;
            // search containers first. the widgets inside have the same z and are
            // more specific, so they should override.
            for (idx, (sub, _)) in self.containers.iter().enumerate() {
                if sub.area.0.contains(pos) {
                    focus_debug!(
                        self.log,
                        "    container area-match {:?}",
                        sub.container_flag.name()
                    );

                    z_order = if let Some(zz) = z_order {
                        if zz.1 <= sub.area.1 {
                            Some((ZOrder::Container(idx), sub.area.1))
                        } else {
                            Some(zz)
                        }
                    } else {
                        Some((ZOrder::Container(idx), sub.area.1))
                    };
                }
            }
            // search widgets
            for (idx, area) in self.areas.iter().enumerate() {
                if area.0.contains(pos) {
                    focus_debug!(
                        self.log,
                        "    area-match {:?}",
                        self.focus_flags[idx].name()
                    );

                    z_order = if let Some(zz) = z_order {
                        if zz.1 <= area.1 {
                            Some((ZOrder::Widget(idx), area.1))
                        } else {
                            Some(zz)
                        }
                    } else {
                        Some((ZOrder::Widget(idx), area.1))
                    };
                }
            }

            // process in order, last is on top if more than one.
            if let Some((idx, _)) = z_order {
                match idx {
                    ZOrder::Widget(idx) => {
                        if self.navigable[idx] != Navigation::None {
                            self.__start_change(true);
                            let r = self.__focus(idx, true);
                            self.__accumulate();
                            return r;
                        } else {
                            focus_debug!(
                                self.log,
                                "    -> not mouse reachable {:?}",
                                self.focus_flags[idx].name()
                            );
                            return false;
                        }
                    }
                    ZOrder::Container(idx) => {
                        let range = &self.containers[idx].1;
                        if let Some(n) = self.first_navigable(range.start) {
                            self.__start_change(true);
                            let r = self.__focus(n, true);
                            self.__accumulate();
                            return r;
                        }
                    }
                }
            }

            // last is on top
            focus_debug!(self.log, "    -> no widget at pos");

            false
        }

        /// Expel focus from the given container.
        pub(super) fn expel_container(&self, flag: FocusFlag) -> bool {
            if let Some((_idx, range)) = self.container_index_of(&flag) {
                self.__start_change(true);
                let n = self.next_navigable(range.end);
                self.__focus(n, true);
                self.__accumulate();

                // still focused?
                if flag.is_focused() {
                    focus_debug!(self.log, "    -> focus not usable. cleared");
                    self.none();
                } else {
                    focus_debug!(self.log, "    -> expelled.");
                }
                true
            } else {
                focus_debug!(self.log, "    => container not found");
                false
            }
        }

        /// Focus next.
        pub(super) fn next(&self) -> bool {
            self.__start_change(true);
            for (n, p) in self.focus_flags.iter().enumerate() {
                if p.lost() {
                    let n = self.next_navigable(n);
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                focus_debug!(
                    self.log,
                    "    use first_navigable {}:{:?}",
                    n,
                    self.focus_flags[n].name()
                );
                self.__focus(n, true);
                self.__accumulate();
                return true;
            }
            focus_debug!(self.log, "    -> no next");
            false
        }

        /// Focus prev.
        pub(super) fn prev(&self) -> bool {
            self.__start_change(true);
            for (i, p) in self.focus_flags.iter().enumerate() {
                if p.lost() {
                    let n = self.prev_navigable(i);
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                focus_debug!(
                    self.log,
                    "    use first_navigable {}:{:?}",
                    n,
                    self.focus_flags[n].name()
                );
                self.__focus(n, true);
                self.__accumulate();
                return true;
            }
            focus_debug!(self.log, "    -> no prev");
            false
        }

        /// Returns the navigation flag for the focused widget.
        pub(super) fn navigation(&self) -> Option<Navigation> {
            self.focus_flags
                .iter()
                .enumerate()
                .find(|(_, v)| v.get())
                .map(|(i, _)| self.navigable[i])
        }

        /// Currently focused.
        pub(super) fn focused(&self) -> Option<FocusFlag> {
            self.focus_flags.iter().find(|v| v.get()).cloned()
        }

        /// Last lost focus.
        pub(super) fn lost_focus(&self) -> Option<FocusFlag> {
            self.focus_flags.iter().find(|v| v.lost()).cloned()
        }

        /// Current gained focus.
        pub(super) fn gained_focus(&self) -> Option<FocusFlag> {
            self.focus_flags.iter().find(|v| v.gained()).cloned()
        }

        /// First navigable flag starting at n.
        fn first_navigable(&self, start: usize) -> Option<usize> {
            focus_debug!(
                self.log,
                "first navigable, start at {}:{:?} ",
                start,
                if start < self.focus_flags.len() {
                    self.focus_flags[start].name()
                } else {
                    "beginning"
                }
            );
            for n in start..self.focus_flags.len() {
                if matches!(
                    self.navigable[n],
                    Navigation::Reach
                        | Navigation::ReachLeaveBack
                        | Navigation::ReachLeaveFront
                        | Navigation::Regular
                ) {
                    focus_debug!(self.log, "    -> {}:{:?}", n, self.focus_flags[n].name());
                    return Some(n);
                }
            }
            focus_debug!(self.log, "    -> no first");
            None
        }

        /// Next navigable flag, starting at start.
        fn next_navigable(&self, start: usize) -> usize {
            focus_debug!(
                self.log,
                "next navigable after {}:{:?}",
                start,
                if start < self.focus_flags.len() {
                    self.focus_flags[start].name()
                } else {
                    "last"
                }
            );

            let mut n = start;
            loop {
                n = if n + 1 < self.focus_flags.len() {
                    n + 1
                } else {
                    0
                };
                if matches!(
                    self.navigable[n],
                    Navigation::Reach
                        | Navigation::ReachLeaveBack
                        | Navigation::ReachLeaveFront
                        | Navigation::Regular
                ) {
                    focus_debug!(self.log, "    -> {}:{:?}", n, self.focus_flags[n].name());
                    return n;
                }
                if n == start {
                    focus_debug!(self.log, "    -> {}:end at start", n);
                    return n;
                }
            }
        }

        /// Previous navigable flag, starting at start.
        fn prev_navigable(&self, start: usize) -> usize {
            focus_debug!(
                self.log,
                "prev navigable before {}:{:?}",
                start,
                self.focus_flags[start].name()
            );

            let mut n = start;
            loop {
                n = if n > 0 {
                    n - 1
                } else {
                    self.focus_flags.len() - 1
                };
                if matches!(
                    self.navigable[n],
                    Navigation::Reach
                        | Navigation::ReachLeaveBack
                        | Navigation::ReachLeaveFront
                        | Navigation::Regular
                ) {
                    focus_debug!(self.log, "    -> {}:{:?}", n, self.focus_flags[n].name());
                    return n;
                }
                if n == start {
                    focus_debug!(self.log, "    -> {}:end at start", n);
                    return n;
                }
            }
        }

        /// Debug destructuring.
        #[allow(clippy::type_complexity)]
        pub(super) fn clone_destruct(
            &self,
        ) -> (
            Vec<FocusFlag>,
            Vec<bool>,
            Vec<(Rect, u16)>,
            Vec<Navigation>,
            Vec<(FocusFlag, (Rect, u16), Range<usize>)>,
        ) {
            (
                self.focus_flags.clone(),
                self.duplicate.clone(),
                self.areas.clone(),
                self.navigable.clone(),
                self.containers
                    .iter()
                    .map(|(v, w)| (v.container_flag.clone(), v.area, w.clone()))
                    .collect::<Vec<_>>(),
            )
        }
    }

    #[cfg(test)]
    mod test {
        use crate::focus::core::FocusCore;
        use crate::{FocusBuilder, FocusFlag, HasFocus};
        use ratatui::layout::Rect;

        #[test]
        fn test_change() {
            assert_eq!(FocusCore::shift(0, 1..1), 1..1);
            assert_eq!(FocusCore::shift(1, 1..1), 2..2);

            assert_eq!(FocusCore::expand(3..4, 0..1), 0..1);
            assert_eq!(FocusCore::expand(3..4, 1..2), 1..2);
            assert_eq!(FocusCore::expand(3..4, 2..3), 2..3);
            assert_eq!(FocusCore::expand(3..4, 3..4), 4..5);
            assert_eq!(FocusCore::expand(3..4, 4..5), 5..6);

            assert_eq!(FocusCore::expand(3..3, 0..1), 0..1);
            assert_eq!(FocusCore::expand(3..3, 1..2), 1..2);
            assert_eq!(FocusCore::expand(3..3, 2..3), 2..3);
            assert_eq!(FocusCore::expand(3..3, 3..4), 3..4);
            assert_eq!(FocusCore::expand(3..3, 4..5), 4..5);

            assert_eq!(FocusCore::shrink(3..4, 0..1), 0..1);
            assert_eq!(FocusCore::shrink(3..4, 2..3), 2..3);
            assert_eq!(FocusCore::shrink(3..4, 3..4), 3..3);
            assert_eq!(FocusCore::shrink(3..4, 4..5), 3..4);
            assert_eq!(FocusCore::shrink(3..4, 5..6), 4..5);

            assert_eq!(FocusCore::shrink(3..3, 0..1), 0..1);
            assert_eq!(FocusCore::shrink(3..3, 1..2), 1..2);
            assert_eq!(FocusCore::shrink(3..3, 2..3), 2..3);
            assert_eq!(FocusCore::shrink(3..3, 3..4), 3..4);
            assert_eq!(FocusCore::shrink(3..3, 4..5), 4..5);
        }

        #[test]
        #[should_panic]
        fn test_double_insert() {
            let a = FocusFlag::named("a");
            let b = FocusFlag::named("b");

            let mut fb = FocusBuilder::new(None);
            fb.widget(&a);
            fb.widget(&b);
            fb.widget(&a);
            fb.build();
        }

        #[test]
        fn test_insert_remove() {
            let a = FocusFlag::named("a");
            let b = FocusFlag::named("b");
            let c = FocusFlag::named("c");
            let d = FocusFlag::named("d");
            let e = FocusFlag::named("e");
            let f = FocusFlag::named("f");
            let g = FocusFlag::named("g");
            let h = FocusFlag::named("h");
            let i = FocusFlag::named("i");

            let mut fb = FocusBuilder::new(None);
            fb.widget(&a);
            fb.widget(&b);
            fb.widget(&c);
            let ff = fb.build();
            assert_eq!(ff.core.focus_flags[0], a);
            assert_eq!(ff.core.focus_flags[1], b);
            assert_eq!(ff.core.focus_flags[2], c);

            let cc = FocusFlag::named("cc");
            let mut fb = FocusBuilder::new(None);
            fb.widget(&a);
            let cc_end = fb.start_with_flags(cc.clone(), Rect::default(), 0);
            fb.widget(&d);
            fb.widget(&e);
            fb.widget(&f);
            fb.end(cc_end);
            fb.widget(&b);
            fb.widget(&c);
            let mut ff = fb.build();
            assert_eq!(ff.core.focus_flags[0], a);
            assert_eq!(ff.core.focus_flags[1], d);
            assert_eq!(ff.core.focus_flags[2], e);
            assert_eq!(ff.core.focus_flags[3], f);
            assert_eq!(ff.core.focus_flags[4], b);
            assert_eq!(ff.core.focus_flags[5], c);
            assert_eq!(ff.core.containers[0].1, 1..4);

            struct DD {
                dd: FocusFlag,
                g: FocusFlag,
                h: FocusFlag,
                i: FocusFlag,
            }

            impl HasFocus for DD {
                fn build(&self, fb: &mut FocusBuilder) {
                    let tag = fb.start_with_flags(self.dd.clone(), self.area(), self.area_z());
                    fb.widget(&self.g);
                    fb.widget(&self.h);
                    fb.widget(&self.i);
                    fb.end(tag);
                }

                fn focus(&self) -> FocusFlag {
                    self.dd.clone()
                }

                fn area(&self) -> Rect {
                    Rect::default()
                }
            }

            let dd = DD {
                dd: FocusFlag::named("dd"),
                g: g.clone(),
                h: h.clone(),
                i: i.clone(),
            };
            ff.replace_container(&cc, &dd);
            assert_eq!(ff.core.focus_flags[0], a);
            assert_eq!(ff.core.focus_flags[1], g);
            assert_eq!(ff.core.focus_flags[2], h);
            assert_eq!(ff.core.focus_flags[3], i);
            assert_eq!(ff.core.focus_flags[4], b);
            assert_eq!(ff.core.focus_flags[5], c);
            assert_eq!(ff.core.containers[0].1, 1..4);
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for Focus {
    #[inline(always)]
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                focus_debug!(
                    self.core.log,
                    "Tab {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                let r = if self.next() {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                };
                focus_debug!(
                    self.core.log,
                    "    -> {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                r
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                focus_debug!(
                    self.core.log,
                    "BackTab {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                let r = if self.prev() {
                    Outcome::Changed
                } else {
                    Outcome::Continue
                };
                focus_debug!(
                    self.core.log,
                    "    -> {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                r
            }
            _ => self.handle(event, MouseOnly),
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for Focus {
    #[inline(always)]
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse down Left for column, row) => {
                focus_debug!(self.core.log, "mouse down {},{}", column, row);
                if self.focus_at(*column, *row) {
                    focus_debug!(
                        self.core.log,
                        "    -> {:?}",
                        self.focused().map(|v| v.name().to_string())
                    );
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
pub fn handle_focus(focus: &mut Focus, event: &crossterm::event::Event) -> Outcome {
    HandleEvent::handle(focus, event, Regular)
}
