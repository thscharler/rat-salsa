use crate::focus::core::FocusCore;
use crate::{FocusFlag, HasFocus, FocusContainer, Navigation};
use rat_event::{ct_event, HandleEvent, MouseOnly, Outcome, Regular};

pub use core::FocusBuilder;

/// Focus deals with all focus-related issues.
///
/// It must be constructed at least after each render(), as it holds
/// copies of the widget-areas for mouse-handling.
///
/// In practice, construct it, when you first need it.
#[derive(Debug, Clone)]
pub struct Focus {
    last: FocusCore,
    core: FocusCore,
}

#[macro_export]
macro_rules! focus_debug {
    ($log:expr, $($arg:tt)+) => {
        if $log.get() {
            log::log!(log::Level::Debug, $($arg)+);
        }
    }
}

impl Focus {
    /// Dynamic change of the widget structure of a container widget.
    ///
    /// This is only necessary if your widget structure changes
    /// during event-handling, and you need a programmatic
    /// focus-change for the new structure.
    ///
    /// This resets the focus-flags of the removed container.
    pub fn remove_container(&mut self, container: &'_ dyn FocusContainer) {
        focus_debug!(
            self.core.log,
            "focus remove container {:?} ",
            container.container().map(|v| v.name().to_string())
        );
        if let Some(flag) = container.container() {
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

    /// Dynamic change of the widget structure of a container.
    ///
    /// This is only necessary if your widget structure changes
    /// during event-handling, and you need a programmatic
    /// focus-change for the new structure.
    ///
    /// If the widget that currently has the focus is still
    /// part of the widget structure it keeps the focus.
    /// The focus-flags for all widgets that are no longer part
    /// of the widget structure are reset.
    pub fn update_container(&mut self, container: &'_ dyn FocusContainer) {
        focus_debug!(
            self.core.log,
            "focus update container {:?} ",
            container.container().map(|v| v.name().to_string())
        );
        if let Some(flag) = container.container() {
            if let Some((cidx, range)) = self.core.container_index_of(&flag) {
                let removed = self.core.remove_container(cidx);

                let mut b = FocusBuilder::new(Some(Focus {
                    last: Default::default(),
                    core: removed,
                }));
                b.container(container);
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
    pub fn replace_container(
        &mut self,
        container: &'_ dyn FocusContainer,
        new: &'_ dyn FocusContainer,
    ) {
        focus_debug!(
            self.core.log,
            "focus replace container {:?} with {:?} ",
            container.container().map(|v| v.name().to_string()),
            new.container().map(|v| v.name().to_string())
        );
        if let Some(flag) = container.container() {
            if let Some((cidx, range)) = self.core.container_index_of(&flag) {
                let removed = self.core.remove_container(cidx);

                let mut b = FocusBuilder::new(Some(Focus {
                    last: Default::default(),
                    core: removed,
                }));
                b.container(new);
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

    /// Sets the focus to the widget.
    ///
    /// Sets the focus, but doesn't set lost or gained.
    /// This can be used to prevent validation of the field.
    pub fn focus_no_lost(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(
            self.core.log,
            "focus no_lost {:?}",
            widget_state.focus().name()
        );
        if let Some(n) = self.core.index_of(widget_state.focus()) {
            self.core.focus_idx(n, false);
        } else {
            focus_debug!(self.core.log, "    => widget not found");
        }
    }

    /// Sets the focus to the given widget.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If this ends up with the same widget as
    /// before gained and lost flags are not set.
    pub fn focus(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(self.core.log, "focus {:?}", widget_state.focus().name());
        if let Some(n) = self.core.index_of(widget_state.focus()) {
            self.core.focus_idx(n, true);
        } else {
            focus_debug!(self.core.log, "    => widget not found");
        }
    }

    /// Expels the focus from the given widget regardless of
    /// the current state.
    pub fn expel_focus(&self, widget_state: &'_ dyn HasFocus) {
        focus_debug!(
            self.core.log,
            "expel from widget {:?}",
            widget_state.focus().name()
        );
        if self.core.index_of(widget_state.focus()).is_some() {
            if widget_state.is_focused() {
                self.core.next();
                if widget_state.is_focused() {
                    focus_debug!(self.core.log, "    -> no other focus, cleared");
                    widget_state.focus().clear();
                } else {
                    focus_debug!(self.core.log, "    -> expelled");
                }
            } else {
                focus_debug!(self.core.log, "    => widget not focused");
            }
        } else {
            focus_debug!(self.core.log, "    => widget not found");
        }
    }

    /// Sets the focus to the given container.
    ///
    /// This sets the focus to the first widget that can be found
    /// for this container.
    pub fn focus_container(&self, container: &'_ dyn FocusContainer) {
        focus_debug!(
            self.core.log,
            "focus container {:?} ",
            container.container().map(|v| v.name().to_string())
        );
        if let Some(flag) = container.container() {
            if let Some((_idx, range)) = self.core.container_index_of(&flag) {
                self.core.focus_idx(range.start, true);
                focus_debug!(self.core.log, "    -> focused");
            } else {
                focus_debug!(self.core.log, "    => container not found");
            }
        } else {
            focus_debug!(self.core.log, "    => no container id");
        }
    }

    /// Expels the focus from the given container regardless of
    /// the current state.
    pub fn expel_focus_container(&self, container: &'_ dyn FocusContainer) {
        focus_debug!(
            self.core.log,
            "focus expel from container {:?} ",
            container.container().map(|v| v.name().to_string())
        );
        if let Some(flag) = container.container() {
            if flag.is_container_focused() {
                self.core.expel_container(flag);
            } else {
                focus_debug!(self.core.log, "    => container not focused");
            }
        } else {
            focus_debug!(self.core.log, "    => no container id");
        }
    }

    /// Sets the focus to the widget with the given flag.
    ///
    /// Sets focus and gained but not lost.
    /// This can be used to prevent validation of the field.
    pub fn focus_flag_no_lost(&self, flag: FocusFlag) {
        focus_debug!(self.core.log, "focus no_lost {:?}", flag.name());
        if let Some(n) = self.core.index_of(flag) {
            self.core.focus_idx(n, false);
            focus_debug!(self.core.log, "    -> focused");
        } else {
            focus_debug!(self.core.log, "    => widget not found");
        }
    }

    /// Sets the focus to the widget with the given flag.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If this ends up with the same widget as
    /// before gained and lost flags are not set.
    pub fn focus_flag(&self, flag: FocusFlag) {
        focus_debug!(self.core.log, "focus {:?}", flag.name());
        if let Some(n) = self.core.index_of(flag) {
            self.core.focus_idx(n, true);
            focus_debug!(self.core.log, "    -> focused");
        } else {
            focus_debug!(self.core.log, "    => widget not found");
        }
    }

    /// Returns the focused widget as FocusFlag.
    ///
    /// This is mainly for debugging purposes.
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    pub fn focused(&self) -> Option<FocusFlag> {
        self.core.focused()
    }

    /// Returns the debug name of the focused widget.
    ///
    /// This is mainly for debugging purposes.
    pub fn focused_name(&self) -> Option<String> {
        self.core.focused().map(|v| v.name().to_string())
    }

    /// Returns the navigation flag for the focused widget.
    pub fn navigation(&self) -> Option<Navigation> {
        self.core.navigation()
    }

    /// Returns the widget that lost the focus as FocusFlag.
    ///
    /// This is mainly for debugging purposes.
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    pub fn lost_focus(&self) -> Option<FocusFlag> {
        self.core.lost_focus()
    }

    /// Returns the widget that gained the focus as FocusFlag.
    ///
    /// This is mainly for debugging purposes.
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    pub fn gained_focus(&self) -> Option<FocusFlag> {
        self.core.gained_focus()
    }

    /// Reset lost + gained flags.
    /// This is done automatically in `HandleEvent::handle()` for every event.
    pub fn reset_lost_gained(&self) {
        self.core.reset_lost_gained();
    }

    /// Change to focus to the given position.
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

    /// Set the initial state for all widgets.
    ///
    /// This ensures that there is only one focused widget.
    /// The first widget in the list gets the focus.
    pub fn first(&self) {
        focus_debug!(self.core.log, "focus first");
        self.core.first();
    }

    /// Focus the first widget of a given container.
    ///
    /// The first navigable widget in the container gets the focus.
    pub fn first_container(&self, container: &'_ dyn FocusContainer) {
        focus_debug!(
            self.core.log,
            "focus first in container {:?} ",
            container.container().map(|v| v.name().to_string())
        );
        if let Some(flag) = container.container() {
            self.core.first_container(flag);
        } else {
            focus_debug!(self.core.log, "    => no container id");
        }
    }

    /// Clear the focus for all widgets.
    ///
    pub fn none(&self) {
        focus_debug!(self.core.log, "focus none");
        self.core.none();
        focus_debug!(self.core.log, "    -> done");
    }

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    ///
    /// If no field has the focus the first one gets it.
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
                    self.core.focused().map(|v| v.name().to_string())
                );
                self.core.next()
            }
            _ => false,
        }
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *true* and sets the focus, gained and lost flag.
    ///
    /// If no field has the focus the first one gets it.
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
                    self.core.focused().map(|v| v.name().to_string())
                );
                self.core.prev()
            }
            _ => false,
        }
    }
}

mod core {
    use crate::{ContainerFlag, Focus, FocusFlag, HasFocus, FocusContainer, Navigation, ZRect};
    use ratatui::layout::Rect;
    use std::cell::Cell;
    use std::ops::Range;

    /// Builder for the Focus.
    #[derive(Debug, Default)]
    pub struct FocusBuilder {
        last: FocusCore,

        log: Cell<bool>,

        // new core
        focus_flags: Vec<FocusFlag>,
        areas: Vec<Rect>,
        z_areas: Vec<Vec<ZRect>>,
        navigable: Vec<Navigation>,
        containers: Vec<(Container, Range<usize>)>,
    }

    impl FocusBuilder {
        pub fn new(last: Option<Focus>) -> FocusBuilder {
            if let Some(last) = last {
                Self {
                    last: last.core,
                    log: Default::default(),
                    focus_flags: last.last.focus_flags,
                    areas: last.last.areas,
                    z_areas: last.last.z_areas,
                    navigable: last.last.navigable,
                    containers: last.last.containers,
                }
            } else {
                Self {
                    last: FocusCore::default(),
                    log: Default::default(),
                    focus_flags: vec![],
                    areas: vec![],
                    z_areas: vec![],
                    navigable: vec![],
                    containers: vec![],
                }
            }
        }

        pub fn enable_log(self) -> Self {
            self.log.set(true);
            self
        }

        /// Add a container widget.
        ///
        /// Adds start+end and calls container.build()
        pub fn container(&mut self, container: &dyn FocusContainer) -> &mut Self {
            let flag = self.start(container.container(), container.area());
            container.build(self);
            self.end(flag);
            self
        }

        /// Add a widget.
        ///
        /// The widget is added to all open containers.
        pub fn widget(&mut self, widget: &dyn HasFocus) -> &mut Self {
            widget.build(self);
            self
        }

        /// Add a widget by setting all the attributes.
        /// The widget is added to all open containers.
        pub fn add_widget(
            &mut self,
            focus: FocusFlag,
            area: Rect,
            z_areas: &[ZRect],
            navigable: Navigation,
        ) {
            for c in &self.focus_flags {
                assert_ne!(*c, focus);
            }

            focus_debug!(self.log, "widget {:?}", focus);

            self.focus_flags.push(focus);
            self.areas.push(area);
            self.z_areas.push(z_areas.into());
            self.navigable.push(navigable);

            let len = self.focus_flags.len();
            for (c, r) in self.containers.iter_mut().rev() {
                if !c.complete {
                    r.end = len;
                }
            }
        }

        /// Manually start a container widget. Must be matched with
        /// the equivalent [end](Self::end).
        ///
        /// Use of [container](Self::container) is preferred.
        ///
        /// __Attention__
        /// If container_flag is None a dummy flag will be created and
        /// returned. Use the returned value when calling [end](Self::end).
        pub fn start(
            &mut self,
            container_flag: Option<ContainerFlag>,
            area: Rect,
        ) -> Option<ContainerFlag> {
            if container_flag.is_none() && area.is_empty() {
                return None;
            }
            let container_flag = container_flag.unwrap_or_default();

            focus_debug!(self.log, "start container {:?}", container_flag);

            let len = self.focus_flags.len();
            self.containers.push((
                Container {
                    container_flag: container_flag.clone(),
                    area,
                    complete: false,
                },
                len..len,
            ));

            Some(container_flag)
        }

        /// Manually end a container widget.
        ///
        /// Use of [container](Self::container) is preferred.
        pub fn end(&mut self, container_flag: Option<ContainerFlag>) {
            let Some(container_flag) = container_flag else {
                return;
            };

            focus_debug!(self.log, "end container {:?}", container_flag);

            for (c, r) in self.containers.iter_mut().rev() {
                if c.container_flag != container_flag {
                    if !c.complete {
                        panic!("FocusBuilder: Unclosed container {:?}", c.container_flag);
                    }
                } else {
                    c.complete = true;
                    focus_debug!(self.log, "container range {:?}", r);
                    break;
                }
            }
        }

        /// Build the final focus.
        ///
        /// If old is given, all widgets/containers no longer in
        /// the focus list are cleared().
        pub fn build(mut self) -> Focus {
            // cleanup outcasts.
            for v in &self.last.focus_flags {
                if !self.focus_flags.contains(v) {
                    v.clear();
                }
            }
            'l: for (v, _) in &self.last.containers {
                for (w, _) in &self.containers {
                    if v.container_flag == w.container_flag {
                        break 'l;
                    }
                }
                v.container_flag.clear();
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
                    focus_flags: self.focus_flags,
                    areas: self.areas,
                    z_areas: self.z_areas,
                    navigable: self.navigable,
                    containers: self.containers,
                },
            }
        }

        /// Shortcut for building the focus for a container
        /// that implements [HasFocus]().
        ///
        /// This creates a fresh Focus.
        ///
        /// __See__
        /// Use [rebuild](FocusBuilder::rebuild) if you want to ensure that widgets
        /// that are no longer in the widget structure have their
        /// focus flag reset properly. If you don't have
        /// some logic to conditionally add widgets to the focus,
        /// this function is probably fine.
        pub fn for_container(container: &dyn FocusContainer) -> Focus {
            let mut b = FocusBuilder::new(None);
            b.container(container);
            b.build()
        }

        /// Shortcut function for building the focus for a container
        /// that implements [HasFocus]()
        ///
        /// This takes the old Focus and reuses most of its allocations.
        /// It also ensures that any widgets no longer in the widget structure
        /// have their focus-flags reset.
        pub fn rebuild(container: &dyn FocusContainer, old: Option<Focus>) -> Focus {
            let mut b = FocusBuilder::new(old);
            b.container(container);
            b.build()
        }
    }

    /// Struct for the data of the focus-container itself.
    #[derive(Debug, Clone)]
    struct Container {
        /// Summarizes all the contained FocusFlags.
        /// If any of them has the focus set, this will be set too.
        /// This can help if you build compound widgets.
        container_flag: ContainerFlag,
        /// Area for the whole compound.
        area: Rect,
        /// Flag for construction.
        complete: bool,
    }

    /// Focus core.
    #[derive(Debug, Default, Clone)]
    pub(super) struct FocusCore {
        /// Focus logging
        pub(super) log: Cell<bool>,

        /// List of flags.
        focus_flags: Vec<FocusFlag>,
        /// Areas for each widget.
        areas: Vec<Rect>,
        /// Areas for each widget split in regions.
        z_areas: Vec<Vec<ZRect>>,
        /// Keyboard navigable
        navigable: Vec<Navigation>,
        /// List of containers and their dependencies.
        /// Range here is a range in the vecs above. The ranges are
        /// all disjoint or completely contained within one other.
        /// No criss-cross intersections.
        containers: Vec<(Container, Range<usize>)>,
    }

    impl FocusCore {
        /// Clear.
        pub(super) fn clear(&mut self) {
            self.focus_flags.clear();
            self.areas.clear();
            self.z_areas.clear();
            self.navigable.clear();
            self.containers.clear();
        }

        /// Find the given focus-flag.
        pub(super) fn index_of(&self, focus_flag: FocusFlag) -> Option<usize> {
            self.focus_flags
                .iter()
                .enumerate()
                .find(|(_, f)| **f == focus_flag)
                .map(|(idx, _)| idx)
        }

        /// Find the given container-flag in the list of sub-containers.
        pub(super) fn container_index_of(
            &self,
            container_flag: &ContainerFlag,
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

            self.focus_flags
                .splice(idx..idx, container.focus_flags.drain(..));
            self.areas.splice(idx..idx, container.areas.drain(..));
            self.z_areas.splice(idx..idx, container.z_areas.drain(..));
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
        }

        /// Remove everything for the given container.
        /// Return the extracted values as FocusCore.
        pub(super) fn remove_container(&mut self, cidx: usize) -> FocusCore {
            let crange = self.containers[cidx].1.clone();

            // remove
            let focus_flags = self.focus_flags.drain(crange.clone()).collect::<Vec<_>>();
            let areas = self.areas.drain(crange.clone()).collect::<Vec<_>>();
            let z_areas = self.z_areas.drain(crange.clone()).collect::<Vec<_>>();
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

            // adjust the remaining sub-containers
            for (_, r) in &mut self.containers {
                *r = Self::shrink(crange.start..crange.end, r.clone());
            }

            FocusCore {
                log: Cell::new(false),
                focus_flags,
                areas,
                z_areas,
                navigable,
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
            for p in self.focus_flags.iter() {
                if set_lost {
                    p.set_lost(p.get());
                } else {
                    p.set_lost(false);
                }
                p.set_gained(false);
                p.set(false);
            }
        }

        /// Set the focus to this index. Doesn't touch
        /// other flags.
        fn __focus(&self, n: usize, set_lost: bool) {
            if let Some(f) = self.focus_flags.get(n) {
                focus_debug!(self.log, "    -> manual focus {:?}", f.name());
                f.set(true);
                if set_lost {
                    if f.lost() {
                        // new focus same as old.
                        // reset lost + gained
                        f.set_lost(false);
                        f.set_gained(false);
                    } else {
                        f.set_gained(true);
                    }
                }
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
            self.__start_change(true);
            if let Some(n) = self.first_navigable(0) {
                focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
                self.__focus(n, true);
            } else {
                focus_debug!(self.log, "    -> no navigable widget");
            }
            self.__accumulate();
        }

        /// Clear the focus.
        pub(super) fn none(&self) {
            self.__start_change(true);
            self.__accumulate();
        }

        /// Set the initial focus.
        pub(super) fn first_container(&self, container: ContainerFlag) {
            self.__start_change(true);
            if let Some((_idx, range)) = self.container_index_of(&container) {
                if let Some(n) = self.first_navigable(range.start) {
                    if n < range.end {
                        focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
                        self.__focus(n, true);
                    }
                } else {
                    focus_debug!(self.log, "    -> no navigable widget");
                }
            } else {
                focus_debug!(self.log, "    => container not found");
            }
            self.__accumulate();
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

            let mut z_order = Vec::new();
            for (idx, area) in self.areas.iter().enumerate() {
                if area.contains(pos) {
                    focus_debug!(
                        self.log,
                        "    area-match {:?}",
                        self.focus_flags[idx].name()
                    );

                    // check for split areas
                    if !self.z_areas[idx].is_empty() {
                        for z_area in &self.z_areas[idx] {
                            // use all matching areas. might differ in z.
                            if z_area.contains(pos) {
                                focus_debug!(
                                    self.log,
                                    "    add z-area-match {:?} -> {:?}",
                                    self.focus_flags[idx].name(),
                                    z_area
                                );
                                z_order.push((idx, z_area.z));
                            }
                        }
                    } else {
                        focus_debug!(self.log, "    add area-match");
                        z_order.push((idx, 0));
                    }
                }
            }
            // process in order, last is on top if more than one.
            if let Some((max_last, _)) = z_order.iter().max_by(|v, w| v.1.cmp(&w.1)) {
                if self.navigable[*max_last] != Navigation::None {
                    self.__start_change(true);
                    self.__focus(*max_last, true);
                    self.__accumulate();
                    focus_debug!(
                        self.log,
                        "    -> focus {:?}",
                        self.focus_flags[*max_last].name()
                    );
                    return true;
                } else {
                    focus_debug!(
                        self.log,
                        "    -> not mouse reachable {:?}",
                        self.focus_flags[*max_last].name()
                    );
                    return false;
                }
            }

            // look through the sub-containers
            z_order.clear();
            for (idx, (sub, _)) in self.containers.iter().enumerate() {
                if sub.area.contains(pos) {
                    focus_debug!(
                        self.log,
                        "    container area-match {:?}",
                        sub.container_flag.name()
                    );
                    z_order.push((idx, 0));
                }
            }
            // last is on top
            if let Some((idx, _)) = z_order.last() {
                let range = &self.containers[*idx].1;
                if let Some(n) = self.first_navigable(range.start) {
                    self.__start_change(true);
                    self.__focus(n, true);
                    self.__accumulate();
                    focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
                    return true;
                }
            }

            focus_debug!(self.log, "    -> no widget at pos");

            false
        }

        /// Expel focus from the given container.
        pub(super) fn expel_container(&self, flag: ContainerFlag) -> bool {
            if let Some((_idx, range)) = self.container_index_of(&flag) {
                self.__start_change(true);
                let n = self.next_navigable(range.end);
                focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
                self.__focus(n, true);
                self.__accumulate();

                // still focused?
                if flag.is_container_focused() {
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
                    focus_debug!(self.log, "    current {:?}", p.name());
                    let n = self.next_navigable(n);
                    focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
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
                    focus_debug!(self.log, "    current {:?}", p.name());
                    let n = self.prev_navigable(i);
                    focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                focus_debug!(self.log, "    -> focus {:?}", self.focus_flags[n].name());
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
                "first navigable {:?} after ",
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
                    focus_debug!(self.log, "    -> {:?}", self.focus_flags[n].name());
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
                "next navigable after {:?}",
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
                    focus_debug!(self.log, "    -> {:?}", self.focus_flags[n].name());
                    return n;
                }
                if n == start {
                    focus_debug!(self.log, "    -> end at start");
                    return n;
                }
            }
        }

        /// Previous navigable flag, starting at start.
        fn prev_navigable(&self, start: usize) -> usize {
            focus_debug!(
                self.log,
                "prev navigable before {:?}",
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
                    focus_debug!(self.log, "    -> {:?}", self.focus_flags[n].name());
                    return n;
                }
                if n == start {
                    focus_debug!(self.log, "    -> end at start");
                    return n;
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::focus::core::FocusCore;
        use crate::{ContainerFlag, FocusBuilder, FocusFlag, FocusContainer};
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

            let cc = ContainerFlag::named("cc");
            let mut fb = FocusBuilder::new(None);
            fb.widget(&a);
            let cc_end = fb.start(Some(cc.clone()), Rect::default());
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
                dd: ContainerFlag,
                g: FocusFlag,
                h: FocusFlag,
                i: FocusFlag,
            }

            impl FocusContainer for DD {
                fn build(&self, fb: &mut FocusBuilder) {
                    fb.widget(&self.g);
                    fb.widget(&self.h);
                    fb.widget(&self.i);
                }

                fn container(&self) -> Option<ContainerFlag> {
                    Some(self.dd.clone())
                }

                fn area(&self) -> Rect {
                    Rect::default()
                }
            }

            let dd = DD {
                dd: ContainerFlag::named("dd"),
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
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                focus_debug!(
                    self.core.log,
                    "Tab {:?}",
                    self.focused().map(|v| v.name().to_string())
                );
                let r = self.next().into();
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
                let r = self.prev().into();
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
pub fn handle_focus(focus: &mut Focus, event: &crossterm::event::Event) -> Outcome {
    HandleEvent::handle(focus, event, Regular)
}
