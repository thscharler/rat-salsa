use crate::focus::core::FocusCore;
use crate::{ContainerFlag, FocusFlag, HasFocus, HasFocusFlag, Navigation};
use log::debug;
use rat_event::{ct_event, HandleEvent, MouseOnly, Outcome, Regular};
use ratatui::layout::Rect;

/// Focus deals with all focus-related issues.
///
/// It must be constructed at least after each render(), as it holds
/// copies of the widget-areas for mouse-handling.
/// In practice, construct it, when you first need it.
#[derive(Debug, Default, Clone)]
pub struct Focus {
    core: FocusCore,
}

impl Focus {
    /// Construct a new focus list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a new focus list.
    pub fn new_list(list: &[&'_ dyn HasFocusFlag]) -> Self {
        let mut ff = Focus::default();
        ff.add_all(list);
        ff
    }

    /// Construct a new focus list for a container widget.
    ///
    /// The focus-flag for the container accumulates all the flags.
    /// If any widget has focus, the container has the focus too.
    /// Focus-gained and focus-lost are only set if the focus
    /// leaves the container.
    ///
    /// The container widget itself interacts with the mouse too.
    /// If no single widget is hit with the mouse, but the area of
    /// the container is, the first widget gets the focus.
    ///
    /// See `examples/focus_recursive` and `examples/focus_recursive2`
    pub fn new_container(container: ContainerFlag, area: Rect) -> Self {
        let mut ff = Focus::default();
        ff.core.set_container_flag(container, area);
        ff
    }

    /// Construct a new focus list for a container widget.
    ///
    /// The focus-flag for the container accumulates all the flags.
    /// If any widget has focus, the container has the focus too.
    /// Focus-gained and focus-lost are only set if the focus
    /// leaves the container.
    ///
    /// The container widget itself interacts with the mouse too.
    /// If no single widget is hit with the mouse, but the area of
    /// the container is, the first widget gets the focus.
    ///
    /// See `examples/focus_recursive` and `examples/focus_recursive2`
    pub fn new_container_list(
        container: ContainerFlag,
        area: Rect,
        list: &[&'_ dyn HasFocusFlag],
    ) -> Self {
        let mut ff = Focus::default();
        ff.core.set_container_flag(container, area);
        ff.add_all(list);
        ff
    }

    /// Container-flag for this Focus.
    pub fn container_flag(&self) -> Option<ContainerFlag> {
        self.core.container_flag()
    }

    /// Container-area for this Focus.
    pub fn container_area(&self) -> Rect {
        self.core.container_area()
    }

    /// Add a container widget.
    pub fn add_container(&mut self, container: &'_ dyn HasFocus) {
        if self.core.log.get() {
            debug!(
                "focus add container {:?} ",
                container.container().map(|v| v.name().to_string())
            );
        }
        let len = self.core.len();
        let clen = self.core.len_sub_containers();
        self.core
            .insert_container(len, clen, container.focus().core);
    }

    /// Remove a container widget.
    pub fn remove_container(&mut self, container: &'_ dyn HasFocus) {
        if self.core.log.get() {
            debug!(
                "focus remove container {:?} ",
                container.container().map(|v| v.name().to_string())
            );
        }
        if let Some(flag) = container.container() {
            if let Some((cidx, _)) = self.core.container_index_of(flag) {
                self.core.remove_container(cidx);
            } else {
                if self.core.log.get() {
                    debug!("    -> container not found");
                }
            }
        } else {
            if self.core.log.get() {
                debug!("    -> no container flag");
            }
        }
    }

    /// Update the composition of the container.
    pub fn update_container(&mut self, container: &'_ dyn HasFocus) {
        if self.core.log.get() {
            debug!(
                "focus update container {:?} ",
                container.container().map(|v| v.name().to_string())
            );
        }
        if let Some(flag) = container.container() {
            if let Some((cidx, range)) = self.core.container_index_of(flag) {
                debug!("{:#?}", self);
                self.core.remove_container(cidx);
                self.core
                    .insert_container(range.start, cidx, container.focus().core);
                debug!("{:#?}", self);
            } else {
                if self.core.log.get() {
                    debug!("    => container not found");
                }
            }
        } else {
            if self.core.log.get() {
                debug!("    => no container flag");
            }
        }
    }

    /// Replace a container widget.
    ///
    /// # Panic
    /// Panics if the
    pub fn replace_container(&mut self, container: &'_ dyn HasFocus, new: &'_ dyn HasFocus) {
        if self.core.log.get() {
            debug!(
                "focus replace container {:?} with {:?} ",
                container.container().map(|v| v.name().to_string()),
                new.container().map(|v| v.name().to_string())
            );
        }
        if let Some(flag) = container.container() {
            if let Some((cidx, range)) = self.core.container_index_of(flag) {
                self.core.remove_container(cidx);
                self.core
                    .insert_container(range.start, cidx, new.focus().core);
            } else {
                if self.core.log.get() {
                    debug!("    -> container not found");
                }
            }
        } else {
            if self.core.log.get() {
                debug!("    -> no container flag");
            }
        }
    }

    /// Add a single widget.
    pub fn add(&mut self, f: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!("focus add {:?} ", f.focus().name());
        }
        self.core.insert(
            self.core.len(),
            f.focus(),
            f.area(),
            f.z_areas(),
            f.navigable(),
        );
    }

    /// Add a single widget.
    pub fn insert_before(&mut self, f: &'_ dyn HasFocusFlag, before: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!(
                "focus insert {:?} before {:?}",
                f.focus().name(),
                before.focus().name()
            );
        }
        if let Some(idx) = self.core.index_of(before.focus()) {
            self.core
                .insert(idx, f.focus(), f.area(), f.z_areas(), f.navigable());
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Add a single widget.
    pub fn insert_after(&mut self, f: &'_ dyn HasFocusFlag, after: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!(
                "focus insert {:?} after {:?}",
                f.focus().name(),
                after.focus().name()
            );
        }
        if let Some(idx) = self.core.index_of(after.focus()) {
            self.core
                .insert(idx + 1, f.focus(), f.area(), f.z_areas(), f.navigable());
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Remove a single widget
    pub fn remove(&mut self, f: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!("focus remove {:?}", f.focus().name());
        }
        if let Some(idx) = self.core.index_of(f.focus()) {
            self.core.remove(idx)
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Update the flags of a widget.
    pub fn update(&mut self, f: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!("focus update {:?}", f.focus().name());
        }
        if let Some(idx) = self.core.index_of(f.focus()) {
            self.core
                .replace(idx, f.focus(), f.area(), f.z_areas(), f.navigable());
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Replace a single widget.
    pub fn replace(&mut self, f: &'_ dyn HasFocusFlag, new: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!(
                "focus replace {:?} -> {:?}",
                f.focus().name(),
                new.focus().name()
            );
        }
        if let Some(idx) = self.core.index_of(f.focus()) {
            self.core
                .replace(idx, new.focus(), new.area(), new.z_areas(), new.navigable());
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Append a list of widgets.
    pub fn add_all(&mut self, list: &[&'_ dyn HasFocusFlag]) {
        for f in list {
            if self.core.log.get() {
                debug!("focus add {:?}", f.focus().name());
            }
            self.core.insert(
                self.core.len(),
                f.focus(),
                f.area(),
                f.z_areas(),
                f.navigable(),
            );
        }
    }

    /// Add a single widget.
    /// This doesn't add any z_areas and assumes navigable is true.
    pub fn add_flag(&mut self, flag: FocusFlag, area: Rect) {
        if self.core.log.get() {
            debug!("focus add {:?}", flag.name().to_string());
        }
        self.core
            .insert(self.core.len(), flag, area, &[], Navigation::Regular);
    }

    /// Add a sub-focus cycle.
    ///
    /// All its widgets are appended to this list. If the sub-cycle
    /// has an accumulator it's added to the sub-accumulators. All
    /// sub-sub-accumulators are appended too.
    pub fn add_focus(&mut self, container: Focus) {
        if self.core.log.get() {
            debug!(
                "focus add {:?}",
                container.container_flag().map(|v| v.name().to_string())
            );
        }
        let len = self.core.len();
        let clen = self.core.len_sub_containers();
        self.core.insert_container(len, clen, container.core);
    }

    /// Writes a log for each operation.
    pub fn enable_log(&self) {
        self.core.log.set(true);
    }

    /// Writes a log for each operation.
    pub fn disable_log(&self) {
        self.core.log.set(false);
    }

    /// Sets the focus to the widget.
    ///
    /// Sets the focus, but doesn't set lost or gained.
    /// This can be used to prevent validation of the field.
    pub fn focus_no_lost(&self, widget_state: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!("focus_no_lost {:?}", widget_state.focus().name());
        }
        if let Some(n) = self.core.index_of(widget_state.focus()) {
            self.core.focus_idx(n, false);
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Sets the focus to the given widget.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If this ends up with the same widget as
    /// before gained and lost flags are not set.
    pub fn focus(&self, widget_state: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!("focus {:?}", widget_state.focus().name());
        }
        if let Some(n) = self.core.index_of(widget_state.focus()) {
            self.core.focus_idx(n, true);
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Sets the focus to the widget with the given flag.
    ///
    /// Sets focus and gained but not lost.
    /// This can be used to prevent validation of the field.
    pub fn focus_flag_no_lost(&self, flag: FocusFlag) {
        if self.core.log.get() {
            debug!("focus_no_lost {:?}", flag.name());
        }
        if let Some(n) = self.core.index_of(flag) {
            self.core.focus_idx(n, false);
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
        }
    }

    /// Sets the focus to the widget with the given flag.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If this ends up with the same widget as
    /// before gained and lost flags are not set.
    pub fn focus_flag(&self, flag: FocusFlag) {
        if self.core.log.get() {
            debug!("focus {:?}", flag.name());
        }
        if let Some(n) = self.core.index_of(flag) {
            self.core.focus_idx(n, true);
        } else {
            if self.core.log.get() {
                debug!("    -> not found");
            }
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
        if self.core.log.get() {
            debug!("reset_lost_gained");
        }
        self.core.reset_lost_gained();
    }

    /// Change the focus.
    ///
    /// Sets the focus, gained and lost flags.
    /// If this ends up with the same widget as
    /// before gained and lost flags are not set.
    pub fn focus_idx(&self, idx: usize) {
        if self.core.log.get() {
            debug!("focus_idx {}", idx);
        }
        self.core.focus_idx(idx, true);
    }

    /// Change to focus to the given position.
    pub fn focus_at(&self, col: u16, row: u16) -> bool {
        if self.core.log.get() {
            debug!("focus_at {},{}", col, row);
        }
        self.core.focus_at(col, row)
    }

    /// Set the initial state for all widgets.
    ///
    /// This ensures that there is only one focused widget.
    /// The first widget in the list gets the focus.
    pub fn first(&self) {
        if self.core.log.get() {
            debug!("first focus");
        }
        self.core.first();
    }

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    ///
    /// If no field has the focus the first one gets it.
    pub fn next(&self) -> bool {
        if self.core.log.get() {
            debug!(
                "next {:?}",
                self.core.focused().map(|v| v.name().to_string())
            );
        }
        self.core.next()
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *true* and sets the focus, gained and lost flag.
    ///
    /// If no field has the focus the first one gets it.
    pub fn prev(&self) -> bool {
        if self.core.log.get() {
            debug!(
                "prev {:?}",
                self.core.focused().map(|v| v.name().to_string())
            );
        }
        self.core.prev()
    }
}

mod core {
    use crate::{ContainerFlag, FocusFlag, Navigation, ZRect};
    use log::debug;
    use ratatui::layout::Rect;
    use std::cell::Cell;
    use std::ops::Range;

    /// Struct for the data of the focus-container itself.
    #[derive(Debug, Clone)]
    struct Container {
        /// Summarizes all the contained FocusFlags.
        /// If any of them has the focus set, this will be set too.
        /// This can help if you build compound widgets.
        container_flag: ContainerFlag,
        /// Area for the whole compound.
        area: Rect,
    }

    /// Focus core.
    #[derive(Debug, Default, Clone)]
    pub(super) struct FocusCore {
        /// Focus logging
        pub(super) log: Cell<bool>,

        /// Summary of all focus-flags in one container focus flag.
        container: Option<Container>,

        /// List of flags.
        focus_flags: Vec<FocusFlag>,
        /// Areas for each widget.
        areas: Vec<Rect>,
        /// Areas for each widget split in regions.
        z_areas: Vec<Vec<ZRect>>,
        /// Keyboard navigable
        navigable: Vec<Navigation>,

        /// List of sub-containers and their dependencies.
        ///
        /// This is filled if you call [crate::Focus::add_focus]. The
        /// container_focus of the appended Focus and all its focus-flags
        /// are added. And all the sub_container's of it are appended too.
        ///
        /// Range here is a range in the vecs above.
        sub_containers: Vec<(Container, Range<usize>)>,
    }

    impl FocusCore {
        /// Set the container itself.
        pub(super) fn set_container_flag(&mut self, container_flag: ContainerFlag, area: Rect) {
            self.container = Some(Container {
                area,
                container_flag,
            })
        }

        /// Get the FocusFlag for the container.
        pub(super) fn container_flag(&self) -> Option<ContainerFlag> {
            self.container.as_ref().map(|v| v.container_flag.clone())
        }

        /// Container-area for this Focus.
        pub(super) fn container_area(&self) -> Rect {
            self.container.as_ref().map(|v| v.area).unwrap_or_default()
        }

        /// Count of focus-flags
        pub(super) fn len(&self) -> usize {
            self.focus_flags.len()
        }

        /// Count of sub-containers.
        pub(super) fn len_sub_containers(&self) -> usize {
            self.sub_containers.len()
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
            container_flag: ContainerFlag,
        ) -> Option<(usize, Range<usize>)> {
            self.sub_containers
                .iter()
                .enumerate()
                .find(|(_, (c, _))| c.container_flag == container_flag)
                .map(|(idx, (_, range))| (idx, range.clone()))
        }

        /// Add focus data.
        pub(super) fn insert(
            &mut self,
            idx: usize,
            focus_flag: FocusFlag,
            area: Rect,
            z_areas: &'_ [ZRect],
            navigable: Navigation,
        ) {
            for c in &self.focus_flags {
                assert_ne!(*c, focus_flag);
            }

            self.focus_flags.insert(idx, focus_flag);
            self.areas.insert(idx, area);
            self.z_areas.insert(idx, Vec::from(z_areas));
            self.navigable.insert(idx, navigable);

            for (_, r) in &mut self.sub_containers {
                *r = Self::expand(idx..idx + 1, r.clone());
            }
        }

        /// Remove a focus flag.
        pub(super) fn remove(&mut self, idx: usize) {
            self.focus_flags.remove(idx);
            self.areas.remove(idx);
            self.z_areas.remove(idx);
            self.navigable.remove(idx);

            for (_, r) in &mut self.sub_containers {
                *r = Self::shrink(idx..idx + 1, r.clone());
            }
        }

        /// Replace a focus flag.
        pub(super) fn replace(
            &mut self,
            idx: usize,
            new: FocusFlag,
            area: Rect,
            z_areas: &'_ [ZRect],
            navigable: Navigation,
        ) {
            for (i, c) in self.focus_flags.iter().enumerate() {
                if i != idx {
                    assert_ne!(*c, new);
                }
            }

            self.focus_flags[idx] = new;
            self.areas[idx] = area;
            self.z_areas[idx] = z_areas.into();
            self.navigable[idx] = navigable;

            // no range change
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
            let end = start + container.focus_flags.len();

            self.focus_flags.splice(idx..idx, container.focus_flags);
            self.areas.splice(idx..idx, container.areas);
            self.z_areas.splice(idx..idx, container.z_areas);
            self.navigable.splice(idx..idx, container.navigable);

            // expand current ranges
            for (_, r) in &mut self.sub_containers {
                *r = Self::expand(start..end, r.clone());
            }
            // shift inserted ranges into place
            for (_, r) in &mut container.sub_containers {
                *r = Self::shift(start, r.clone());
            }

            let clen = container.sub_containers.len();

            self.sub_containers
                .splice(cidx..cidx, container.sub_containers);
            if let Some(c) = container.container {
                self.sub_containers.insert(cidx + clen, (c, start..end));
            }
        }

        /// Remove everything for the given container.
        pub(super) fn remove_container(&mut self, cidx: usize) {
            let crange = self.sub_containers[cidx].1.clone();

            // remove the container and all sub-containers in the range.
            self.sub_containers
                .retain(|(_, r)| !(crange.start <= r.start && crange.end >= r.end));

            self.focus_flags.drain(crange.clone());
            self.areas.drain(crange.clone());
            self.z_areas.drain(crange.clone());
            self.navigable.drain(crange.clone());

            // adjust the remaining sub-containers
            for (_, r) in &mut self.sub_containers {
                *r = Self::shrink(crange.start..crange.end, r.clone());
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
                if self.log.get() {
                    debug!("    -> manual focus {:?}", f.name());
                }
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
            if let Some(container) = &self.container {
                container.container_flag.set(false);
                for p in self.focus_flags.iter() {
                    if p.get() {
                        container.container_flag.set(true);
                        break;
                    }
                }
            }

            for (f, r) in &self.sub_containers {
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

        /// Reset all lost+gained flags.
        pub(super) fn reset_lost_gained(&self) {
            if let Some(container) = &self.container {
                container.container_flag.set_gained(false);
                container.container_flag.set_lost(false);
            }
            for f in self.focus_flags.iter() {
                f.set_lost(false);
                f.set_gained(false);
            }
            for (f, _) in self.sub_containers.iter() {
                f.container_flag.set_gained(false);
                f.container_flag.set_lost(false);
            }
        }

        /// Set the initial focus.
        pub(super) fn first(&self) {
            self.__start_change(true);
            if let Some(n) = self.first_navigable(0) {
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus_flags[n].name());
                }
                self.__focus(n, true);
            } else {
                if self.log.get() {
                    debug!("    -> no navigable widget");
                }
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
                    if self.log.get() {
                        debug!("    area-match {:?}", self.focus_flags[idx].name());
                    }

                    // check for split areas
                    if !self.z_areas[idx].is_empty() {
                        for z_area in &self.z_areas[idx] {
                            // use all matching areas. might differ in z.
                            if z_area.contains(pos) {
                                if self.log.get() {
                                    debug!(
                                        "    add z-area-match {:?} -> {:?}",
                                        self.focus_flags[idx].name(),
                                        z_area
                                    );
                                }
                                z_order.push((idx, z_area.z));
                            }
                        }
                    } else {
                        if self.log.get() {
                            debug!("    add area-match");
                        }
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
                    if self.log.get() {
                        debug!("    -> focus {:?}", self.focus_flags[*max_last].name());
                    }
                    return true;
                } else {
                    if self.log.get() {
                        debug!(
                            "    -> not mouse reachable {:?}",
                            self.focus_flags[*max_last].name()
                        );
                    }
                    return false;
                }
            }

            // look through the sub-containers
            for (sub, range) in &self.sub_containers {
                if sub.area.contains(pos) {
                    if self.log.get() {
                        debug!("    container area-match {:?}", sub.container_flag.name());
                    }

                    if let Some(n) = self.first_navigable(range.start) {
                        self.__start_change(true);
                        self.__focus(n, true);
                        self.__accumulate();
                        if self.log.get() {
                            debug!("    -> focus {:?}", self.focus_flags[n].name());
                        }
                        return true;
                    }
                }
            }

            // main container
            if let Some(con) = &self.container {
                if con.area.contains(pos) {
                    if self.log.get() {
                        debug!(
                            "    main container area-match {:?}",
                            con.container_flag.name()
                        );
                    }

                    if let Some(n) = self.first_navigable(0) {
                        self.__start_change(true);
                        self.__focus(n, true);
                        self.__accumulate();
                        if self.log.get() {
                            debug!("    -> focus {:?}", self.focus_flags[n].name());
                        }
                        return true;
                    }
                }
            }

            if self.log.get() {
                debug!("    -> no widget at pos");
            }

            false
        }

        /// Focus next.
        pub(super) fn next(&self) -> bool {
            self.__start_change(true);
            for (n, p) in self.focus_flags.iter().enumerate() {
                if p.lost() {
                    if self.log.get() {
                        debug!("    current {:?}", p.name());
                    }
                    let n = self.next_navigable(n);
                    if self.log.get() {
                        debug!("    -> focus {:?}", self.focus_flags[n].name());
                    }
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus_flags[n].name());
                }
                self.__focus(n, true);
                self.__accumulate();
                return true;
            }
            if self.log.get() {
                debug!("    -> no next");
            }
            false
        }

        /// Focus prev.
        pub(super) fn prev(&self) -> bool {
            self.__start_change(true);
            for (i, p) in self.focus_flags.iter().enumerate() {
                if p.lost() {
                    if self.log.get() {
                        debug!("    current {:?}", p.name());
                    }
                    let n = self.prev_navigable(i);
                    if self.log.get() {
                        debug!("    -> focus {:?}", self.focus_flags[n].name());
                    }
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus_flags[n].name());
                }
                self.__focus(n, true);
                self.__accumulate();
                return true;
            }
            if self.log.get() {
                debug!("    -> no prev");
            }
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
            if self.log.get() {
                debug!("first navigable {:?}", self.focus_flags[start].name());
            }
            for n in start..self.len() {
                if matches!(self.navigable[n], Navigation::Reach | Navigation::Regular) {
                    if self.log.get() {
                        debug!("    -> {:?}", self.focus_flags[n].name());
                    }
                    return Some(n);
                }
            }
            if self.log.get() {
                debug!("    -> no first");
            }
            None
        }

        /// Next navigable flag, starting at start.
        fn next_navigable(&self, start: usize) -> usize {
            if self.log.get() {
                debug!("next navigable {:?}", self.focus_flags[start].name());
            }

            let mut n = start;
            loop {
                n = if n + 1 < self.len() { n + 1 } else { 0 };
                if matches!(self.navigable[n], Navigation::Reach | Navigation::Regular) {
                    if self.log.get() {
                        debug!("    -> {:?}", self.focus_flags[n].name());
                    }
                    return n;
                }
                if n == start {
                    if self.log.get() {
                        debug!("    -> end at start");
                    }
                    return n;
                }
            }
        }

        /// Previous navigable flag, starting at start.
        fn prev_navigable(&self, start: usize) -> usize {
            if self.log.get() {
                debug!("prev navigable {:?}", self.focus_flags[start].name());
            }

            let mut n = start;
            loop {
                n = if n > 0 { n - 1 } else { self.len() - 1 };
                if matches!(self.navigable[n], Navigation::Reach | Navigation::Regular) {
                    if self.log.get() {
                        debug!("    -> {:?}", self.focus_flags[n].name());
                    }
                    return n;
                }
                if n == start {
                    if self.log.get() {
                        debug!("    -> end at start");
                    }
                    return n;
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::focus::core::FocusCore;
        use crate::{ContainerFlag, Focus, FocusFlag, Navigation};
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

            let mut core = FocusCore::default();
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(
                core.len(),
                b.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
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

            let mut core = FocusCore::default();
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(
                core.len(),
                b.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(
                core.len(),
                c.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            assert_eq!(core.focus_flags[0], a);
            assert_eq!(core.focus_flags[1], b);
            assert_eq!(core.focus_flags[2], c);

            let mut core = FocusCore::default();
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(
                core.len(),
                b.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(1, c.clone(), Rect::default(), &[], Navigation::Regular);
            assert_eq!(core.focus_flags[0], a);
            assert_eq!(core.focus_flags[1], c);
            assert_eq!(core.focus_flags[2], b);
            core.remove(1);
            assert_eq!(core.focus_flags[1], b);

            let cc = ContainerFlag::named("cc");
            let mut sub0 = Focus::new_container(cc.clone(), Rect::default());
            sub0.add_flag(d.clone(), Rect::default());
            sub0.add_flag(e.clone(), Rect::default());
            sub0.add_flag(f.clone(), Rect::default());

            let dd = ContainerFlag::named("dd");
            let mut sub1 = Focus::new_container(dd.clone(), Rect::default());
            sub1.add_flag(g.clone(), Rect::default());
            sub1.add_flag(h.clone(), Rect::default());
            sub1.add_flag(i.clone(), Rect::default());

            let mut core = FocusCore::default();
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert_container(core.len(), core.len_sub_containers(), sub0.core.clone());
            core.insert(
                core.len(),
                b.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert(
                core.len(),
                c.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            assert_eq!(core.focus_flags[0], a);
            assert_eq!(core.focus_flags[1], d);
            assert_eq!(core.focus_flags[2], e);
            assert_eq!(core.focus_flags[3], f);
            assert_eq!(core.focus_flags[4], b);
            assert_eq!(core.focus_flags[5], c);
            assert_eq!(core.sub_containers[0].1, 1..4);

            let mut core = FocusCore::default();
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert_container(core.len(), core.len_sub_containers(), sub0.core.clone());
            core.insert(1, b.clone(), Rect::default(), &[], Navigation::Regular);
            core.insert(5, c.clone(), Rect::default(), &[], Navigation::Regular);
            assert_eq!(core.focus_flags[0], a);
            assert_eq!(core.focus_flags[1], b);
            assert_eq!(core.focus_flags[2], d);
            assert_eq!(core.focus_flags[3], e);
            assert_eq!(core.focus_flags[4], f);
            assert_eq!(core.focus_flags[5], c);
            assert_eq!(core.sub_containers[0].1, 2..5);
            core.remove(1);
            assert_eq!(core.focus_flags[1], d);
            assert_eq!(core.sub_containers[0].1, 1..4);

            let mut core = FocusCore::default();
            core.insert(
                core.len(),
                a.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            core.insert_container(core.len(), core.len_sub_containers(), sub0.core.clone());
            core.insert_container(core.len(), core.len_sub_containers(), sub1.core.clone());
            core.insert(
                core.len(),
                b.clone(),
                Rect::default(),
                &[],
                Navigation::Regular,
            );
            assert_eq!(core.focus_flags[0], a);
            assert_eq!(core.focus_flags[1], d);
            assert_eq!(core.focus_flags[2], e);
            assert_eq!(core.focus_flags[3], f);
            assert_eq!(core.focus_flags[4], g);
            assert_eq!(core.focus_flags[5], h);
            assert_eq!(core.focus_flags[6], i);
            assert_eq!(core.focus_flags[7], b);
            assert_eq!(core.sub_containers[0].1, 1..4);
            assert_eq!(core.sub_containers[1].1, 4..7);
            core.remove(0);
            assert_eq!(core.focus_flags[0], d);
            assert_eq!(core.focus_flags[6], b);
            assert_eq!(core.sub_containers[0].1, 0..3);
            assert_eq!(core.sub_containers[1].1, 3..6);
            core.insert(0, a.clone(), Rect::default(), &[], Navigation::Regular);
            assert_eq!(core.focus_flags[1], d);
            assert_eq!(core.focus_flags[7], b);
            assert_eq!(core.sub_containers[0].1, 1..4);
            assert_eq!(core.sub_containers[1].1, 4..7);
            core.insert(0, c.clone(), Rect::default(), &[], Navigation::Regular);
            core.remove_container(1);
            assert_eq!(core.focus_flags[2], d);
            assert_eq!(core.focus_flags[5], b);
            assert_eq!(core.sub_containers[0].1, 2..5);
            assert_eq!(core.sub_containers.len(), 1);
            core.insert_container(2, 0, sub1.core.clone());
            assert_eq!(core.focus_flags[2], g);
            assert_eq!(core.focus_flags[5], d);
            assert_eq!(core.focus_flags[8], b);
            assert_eq!(core.sub_containers[0].1, 2..5);
            assert_eq!(core.sub_containers[1].1, 5..8);
            assert_eq!(core.sub_containers[0].0.container_flag.name(), "dd");
            assert_eq!(core.sub_containers[1].0.container_flag.name(), "cc");
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for Focus {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                if matches!(
                    self.navigation(),
                    Some(Navigation::Leave | Navigation::Regular)
                ) {
                    if self.core.log.get() {
                        debug!("Tab {:?}", self.focused().map(|v| v.name().to_string()));
                    }
                    let r = self.next().into();
                    if self.core.log.get() {
                        debug!("    -> {:?}", self.focused().map(|v| v.name().to_string()));
                    }
                    r
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                if matches!(
                    self.navigation(),
                    Some(Navigation::Leave | Navigation::Regular)
                ) {
                    if self.core.log.get() {
                        debug!("BackTab {:?}", self.focused().map(|v| v.name().to_string()));
                    }
                    let r = self.prev().into();
                    if self.core.log.get() {
                        debug!("    -> {:?}", self.focused().map(|v| v.name().to_string()));
                    }
                    r
                } else {
                    Outcome::Continue
                }
            }
            _ => self.handle(event, MouseOnly),
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for Focus {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse down Left for column, row) => {
                if self.core.log.get() {
                    debug!("mouse down {},{}", column, row);
                }
                if self.focus_at(*column, *row) {
                    if self.core.log.get() {
                        debug!("    -> {:?}", self.focused().map(|v| v.name().to_string()));
                    }
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

/// Handle only mouse-events.
pub fn handle_mouse_focus(focus: &mut Focus, event: &crossterm::event::Event) -> Outcome {
    HandleEvent::handle(focus, event, MouseOnly)
}
