use crate::focus::core::FocusCore;
use crate::{FocusFlag, HasFocus, HasFocusFlag};
use log::debug;
use rat_event::{ct_event, HandleEvent, MouseOnly, Outcome, Regular};
use ratatui::layout::Rect;

/// Focus deals with all focus-related issues.
///
/// It must be constructed at least after each render(), as it holds
/// copies of the widget-areas for mouse-handling.
/// In practice, construct it, when you first need it.
#[derive(Debug, Default)]
pub struct Focus {
    /// Name for the cycle.
    name: String,
    /// core impl
    core: FocusCore,
}

impl Focus {
    /// Construct a new focus list.
    pub fn new(list: &[&'_ dyn HasFocusFlag]) -> Self {
        let mut ff = Focus::default();
        for f in list {
            ff.core.add(
                f.focus(),
                f.area(),
                f.z_areas(),
                f.navigable(),
                f.primary_keys(),
            );
        }
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
    pub fn new_container(c: &'_ dyn HasFocusFlag, list: &[&'_ dyn HasFocusFlag]) -> Self {
        let mut ff = Focus::default();
        ff.core.set_container(c.focus(), c.area(), c.z_areas());
        for f in list {
            ff.core.add(
                f.focus(),
                f.area(),
                f.z_areas(),
                f.navigable(),
                f.primary_keys(),
            );
        }
        ff
    }

    /// Construct a new focus list with group accumulator.
    ///
    /// This is meant for some loose grouping of widgets, for which
    /// you want an overview.
    ///
    /// The same rules apply as for new_accu(), but with this one
    /// there is no overall area for mouse interaction.
    pub fn new_grp(grp: &'_ FocusFlag, list: &[&'_ dyn HasFocusFlag]) -> Self {
        let mut ff = Focus::default();
        ff.core.set_container(grp, Rect::ZERO, &[]);
        for f in list {
            ff.core.add(
                f.focus(),
                f.area(),
                f.z_areas(),
                f.navigable(),
                f.primary_keys(),
            );
        }
        ff
    }

    /// Add a single widget.
    /// This doesn't add any z_areas and assumes navigable is true.
    pub fn add_flag(&mut self, flag: &'_ FocusFlag, area: Rect) -> &mut Self {
        self.core.add(flag, area, &[], true, true);
        self
    }

    /// Add a sub-focus cycle.
    ///
    /// All its widgets are appended to this list. If the sub-cycle
    /// has an accumulator it's added to the sub-accumulators. All
    /// sub-sub-accumulators are appended too.
    pub fn add_focus(&mut self, focus: Focus) -> &mut Self {
        self.core.add_focus(focus.core);
        self
    }

    /// Add a container widget.
    pub fn add_container(&mut self, c: &'_ dyn HasFocus) -> &mut Self {
        let ff = c.focus();
        self.core.add_focus(ff.core);
        self
    }

    /// Add a single widget.
    pub fn add(&mut self, f: &'_ dyn HasFocusFlag) -> &mut Self {
        self.core.add(
            f.focus(),
            f.area(),
            f.z_areas(),
            f.navigable(),
            f.primary_keys(),
        );
        self
    }

    /// Add a list of widgets.
    pub fn add_all(&mut self, list: &[&'_ dyn HasFocusFlag]) -> &mut Self {
        for f in list {
            self.core.add(
                f.focus(),
                f.area(),
                f.z_areas(),
                f.navigable(),
                f.primary_keys(),
            );
        }
        self
    }

    /// Writes a log for each operation.
    pub fn enable_log(self, log: bool) -> Self {
        self.core.log.set(log);
        self
    }

    /// Set a name for debugging.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Name for debugging.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Set the initial state for all widgets.
    ///
    /// This ensures that there is only one focused widget.
    /// The first widget in the list gets the focus.
    pub fn initial(&self) {
        if self.core.log.get() {
            debug!("init focus");
        }
        self.core.focus_init();
    }

    /// Sets the focus to the widget.
    ///
    /// Sets the focus, but doesn't set lost or gained.
    /// This can be used to prevent validation of the field.
    pub fn focus_no_lost(&self, widget_state: &'_ dyn HasFocusFlag) {
        if self.core.log.get() {
            debug!("focus_no_lost {:?}", widget_state.focus());
        }
        if let Some(n) = self.core.index_of(widget_state.focus()) {
            self.core.focus_no_lost(n);
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
            debug!("focus {:?}", widget_state.focus());
        }
        if let Some(n) = self.core.index_of(widget_state.focus()) {
            self.core.focus_idx(n);
        }
    }

    /// Sets the focus to the widget with the given flag.
    ///
    /// Sets focus and gained but not lost.
    /// This can be used to prevent validation of the field.
    pub fn focus_flag_no_lost(&self, flag: &FocusFlag) {
        if self.core.log.get() {
            debug!("focus_no_lost {:?}", flag);
        }
        if let Some(n) = self.core.index_of(flag) {
            self.core.focus_no_lost(n);
        }
    }

    /// Sets the focus to the widget with the given flag.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If this ends up with the same widget as
    /// before gained and lost flags are not set.
    pub fn focus_flag(&self, flag: &FocusFlag) {
        if self.core.log.get() {
            debug!("focus {:?}", flag);
        }
        if let Some(n) = self.core.index_of(flag) {
            self.core.focus_idx(n);
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
        self.core.focus_idx(idx);
    }

    /// Change to focus to the given position.
    pub fn focus_at(&self, col: u16, row: u16) -> bool {
        if self.core.log.get() {
            debug!("focus_at {},{}", col, row);
        }
        self.core.focus_at(col, row)
    }

    /// Does the current focus use the primary focus keys or
    /// the secondary focus keys to leave the widget.
    /// Some widgets (textarea) might want to use the primary keys
    /// tab/shift tab themselves.
    pub fn use_primary_keys(&self) -> bool {
        self.core.use_primary_keys()
    }

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    ///
    /// If no field has the focus the first one gets it.
    pub fn next(&self) -> bool {
        if self.core.log.get() {
            debug!("next {:?}", self.core.focused());
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
            debug!("prev {:?}", self.core.focused());
        }
        self.core.prev()
    }
}

mod core {
    use crate::{FocusFlag, ZRect};
    use log::debug;
    use ratatui::layout::Rect;
    use std::cell::Cell;
    use std::rc::Rc;

    /// Struct for the data of the focus-container itself.
    #[derive(Debug, Clone)]
    struct Container {
        /// Area for the whole compound. Only valid if container_focus is Some().
        area: Rect,
        /// Area split in regions. Only valid if container_focus is Some().
        z_area: Vec<ZRect>,
        /// Summarizes all the contained FocusFlags.
        /// If any of them has the focus set, this will be set too.
        /// This can help if you build compound widgets.
        focus: FocusFlag,
    }

    /// Focus core.
    #[derive(Debug, Default)]
    pub(super) struct FocusCore {
        /// Focus logging
        pub(super) log: Cell<bool>,

        /// Summary of all focus-flags in one container focus flag.
        container: Option<Container>,

        /// Areas for each widget.
        areas: Vec<Rect>,
        /// Areas for each widget split in regions.
        z_areas: Vec<Vec<ZRect>>,
        /// Keyboard navigable
        navigable: Vec<bool>,
        /// Primary or secondary focus keys to leave the widget.
        primary_keys: Vec<bool>,
        /// List of flags.
        focus: Vec<FocusFlag>,

        /// List of sub-containers and their dependencies.
        ///
        /// This is filled if you call [crate::Focus::add_focus]. The
        /// container_focus of the appended Focus and all its focus-flags
        /// are added. And all the sub_container's of it are appended too.
        sub_container: Vec<(Container, Vec<FocusFlag>)>,
    }

    impl FocusCore {
        /// Set the container itself.
        pub(super) fn set_container(
            &mut self,
            focus: &'_ FocusFlag,
            area: Rect,
            z_area: &'_ [ZRect],
        ) {
            self.container = Some(Container {
                area,
                z_area: Vec::from(z_area),
                focus: focus.clone(),
            })
        }

        /// Add focus data.
        pub(super) fn add(
            &mut self,
            focus: &'_ FocusFlag,
            area: Rect,
            z_areas: &'_ [ZRect],
            navigable: bool,
            primary_keys: bool,
        ) {
            self.focus.push(focus.clone());
            self.areas.push(area);
            self.z_areas.push(Vec::from(z_areas));
            self.navigable.push(navigable);
            self.primary_keys.push(primary_keys);
        }

        /// Append another focus to this one.
        pub(super) fn add_focus(&mut self, focus: FocusCore) {
            // container area probably overlaps with the areas of sub-containers.
            // search those first.
            for v in focus.sub_container {
                self.sub_container.push(v);
            }
            if let Some(container) = focus.container {
                self.sub_container.push((container, focus.focus.clone()));
            }

            self.focus.extend(focus.focus);
            self.areas.extend(focus.areas);
            self.z_areas.extend(focus.z_areas);
            self.navigable.extend(focus.navigable);
            self.primary_keys.extend(focus.primary_keys);
        }

        /// Find the given focus-flag.
        pub(super) fn index_of(&self, flag: &FocusFlag) -> Option<usize> {
            if let Some((n, _)) = self
                .focus
                .iter()
                .enumerate() //
                .find(|(_, f)| Rc::ptr_eq(&f.0, &flag.0))
            // todo: will not hold
            {
                Some(n)
            } else {
                None
            }
        }

        /// Reset the flags for a new round.
        /// set_lost - copy the current focus to the lost flag.
        fn __start_change(&self, set_lost: bool) {
            for p in self.focus.iter() {
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
            if let Some(f) = self.focus.get(n) {
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
                container.focus.set(false);
                for p in self.focus.iter() {
                    if p.get() {
                        container.focus.set(true);
                        break;
                    }
                }
            }

            for (f, list) in &self.sub_container {
                let mut any_gained = false;
                let mut any_lost = false;
                let mut any_focused = false;

                for f in list {
                    any_gained |= f.gained();
                    any_lost |= f.lost();
                    any_focused |= f.get();
                }

                f.focus.set(any_focused);
                f.focus.set_lost(any_lost && !any_gained);
                f.focus.set_gained(any_gained && !any_lost);
            }
        }

        /// Reset all lost+gained flags.
        pub(super) fn reset_lost_gained(&self) {
            if let Some(container) = &self.container {
                container.focus.set_gained(false);
                container.focus.set_lost(false);
            }
            for p in self.focus.iter() {
                p.set_lost(false);
                p.set_gained(false);
            }
            for (p, _) in self.sub_container.iter() {
                p.focus.set_gained(false);
                p.focus.set_lost(false);
            }
        }

        /// Set the initial focus.
        pub(super) fn focus_init(&self) {
            if self.log.get() {
                debug!("first init");
            }
            self.__start_change(true);
            if let Some(n) = self.first_navigable(0) {
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus[n]);
                }
                self.__focus(n, true);
            }
            self.__accumulate();
        }

        /// Set the focus but not the lost flag.
        pub(super) fn focus_no_lost(&self, n: usize) {
            if self.log.get() {
                debug!("focus_no_lost {}", n);
            }
            self.__start_change(false);
            if self.log.get() {
                debug!("    -> focus {:?}", self.focus[n]);
            }
            self.__focus(n, false);
            self.__accumulate();
        }

        /// Set the focus at the given index.
        pub(super) fn focus_idx(&self, n: usize) {
            if self.log.get() {
                debug!("focus_idx {}", n);
            }
            self.__start_change(true);
            if self.log.get() {
                debug!("    -> focus {:?}", self.focus[n]);
            }
            self.__focus(n, true);
            self.__accumulate();
        }

        /// Set the focus at the given screen position.
        /// Traverses the list to find the matching widget.
        /// Checks the area and the z-areas.
        pub(super) fn focus_at(&self, col: u16, row: u16) -> bool {
            if self.log.get() {
                debug!("focus_at {}:{}", col, row);
            }

            let pos = (col, row).into();

            let mut z_order = Vec::new();
            for (idx, area) in self.areas.iter().enumerate() {
                if area.contains(pos) {
                    if self.log.get() {
                        debug!("    area-match {:?}", self.focus[idx]);
                    }

                    // check for split areas
                    if !self.z_areas[idx].is_empty() {
                        for z_area in &self.z_areas[idx] {
                            // use all matching areas. might differ in z.
                            if z_area.contains(pos) {
                                if self.log.get() {
                                    debug!(
                                        "    add z-area-match {:?} -> {:?}",
                                        self.focus[idx], z_area
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
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus[*max_last]);
                }
                self.__start_change(true);
                self.__focus(*max_last, true);
                self.__accumulate();
                return true;
            }

            // look through the sub-containers
            let mut z_order = Vec::new();
            for (sub, focus) in &self.sub_container {
                if sub.area.contains(pos) {
                    if self.log.get() {
                        debug!("    container area-match {:?}", sub.focus);
                    }

                    // check for split areas
                    if !sub.z_area.is_empty() {
                        for z_area in &sub.z_area {
                            // use all matching areas. might differ in z.
                            if z_area.contains(pos) {
                                if self.log.get() {
                                    debug!("    add z-area-match {:?} -> {:?}", sub.focus, z_area);
                                }
                                z_order.push((focus.first(), z_area.z));
                                break;
                            }
                        }
                    } else {
                        z_order.push((focus.first(), 0));
                    }

                    // process in order, last is on top if more than one.
                    if let Some((max_last, _)) = z_order.iter().max_by(|v, w| v.1.cmp(&w.1)) {
                        if self.log.get() {
                            debug!("    -> focus {:?}", max_last);
                        }
                        if let Some(max_last) = max_last {
                            if let Some(max_last) = self.index_of(max_last) {
                                if let Some(n) = self.first_navigable(max_last) {
                                    self.__start_change(true);
                                    self.__focus(n, true);
                                    self.__accumulate();
                                    return true;
                                }
                            }
                        }
                    }
                }
            }

            // main container
            // look through the sub-containers

            if let Some(con) = &self.container {
                let mut change = false;

                if con.area.contains(pos) {
                    if self.log.get() {
                        debug!("    main container area-match {:?}", con.focus);
                    }

                    // check for split areas
                    if !con.z_area.is_empty() {
                        for z_area in &con.z_area {
                            // use all matching areas. might differ in z.
                            if z_area.contains(pos) {
                                if self.log.get() {
                                    debug!("    add z-area-match {:?} -> {:?}", con.focus, z_area);
                                }
                                change = true;
                                break;
                            }
                        }
                    } else {
                        change = true;
                    }

                    if change {
                        if let Some(n) = self.first_navigable(0) {
                            self.__start_change(true);
                            self.__focus(n, true);
                            self.__accumulate();
                            return true;
                        }
                    }
                }
            }

            false
        }

        /// Use primary focus keys.
        pub(super) fn use_primary_keys(&self) -> bool {
            if let Some((idx, _)) = self.focus.iter().enumerate().find(|v| v.1.get()) {
                self.primary_keys[idx]
            } else {
                true
            }
        }

        /// Focus next.
        pub(super) fn next(&self) -> bool {
            if self.log.get() {
                debug!("next");
            }
            self.__start_change(true);
            for (n, p) in self.focus.iter().enumerate() {
                if p.lost() {
                    if self.log.get() {
                        debug!("    current {:?}", p);
                    }
                    let n = self.next_navigable(n);
                    if self.log.get() {
                        debug!("    -> focus {:?}", self.focus[n]);
                    }
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus[n]);
                }
                self.__focus(n, true);
                self.__accumulate();
                return true;
            }
            false
        }

        /// Focus prev.
        pub(super) fn prev(&self) -> bool {
            if self.log.get() {
                debug!("prev");
            }
            self.__start_change(true);
            for (i, p) in self.focus.iter().enumerate() {
                if p.lost() {
                    if self.log.get() {
                        debug!("    current {:?}", p);
                    }
                    let n = self.prev_navigable(i);
                    if self.log.get() {
                        debug!("    -> focus {:?}", self.focus[n]);
                    }
                    self.__focus(n, true);
                    self.__accumulate();
                    return true;
                }
            }
            if let Some(n) = self.first_navigable(0) {
                if self.log.get() {
                    debug!("    -> focus {:?}", self.focus[n]);
                }
                self.__focus(n, true);
                self.__accumulate();
                return true;
            }
            false
        }

        /// Currently focused.
        pub(super) fn focused(&self) -> Option<FocusFlag> {
            self.focus.iter().find(|v| v.get()).cloned()
        }

        /// Last lost focus.
        pub(super) fn lost_focus(&self) -> Option<FocusFlag> {
            self.focus.iter().find(|v| v.lost()).cloned()
        }

        /// Current gained focus.
        pub(super) fn gained_focus(&self) -> Option<FocusFlag> {
            self.focus.iter().find(|v| v.gained()).cloned()
        }

        /// Count.
        fn len(&self) -> usize {
            self.focus.len()
        }

        /// First navigable flag starting at n.
        fn first_navigable(&self, start: usize) -> Option<usize> {
            if self.log.get() {
                debug!("first navigable {:?}", self.focus[start].name());
            }
            for n in start..self.len() {
                if self.navigable[n] {
                    if self.log.get() {
                        debug!("first navigable -> {:?}", self.focus[n].name());
                    }
                    return Some(n);
                }
            }
            if self.log.get() {
                debug!("first navigable -> None");
            }
            None
        }

        /// Next navigable flag, starting at start.
        fn next_navigable(&self, start: usize) -> usize {
            if self.log.get() {
                debug!("next navigable {:?}", self.focus[start].name());
            }

            let mut n = start;
            loop {
                n = if n + 1 < self.len() { n + 1 } else { 0 };
                if self.navigable[n] {
                    if self.log.get() {
                        debug!("next navigable -> {:?}", self.focus[n].name());
                    }
                    return n;
                }
                if n == start {
                    if self.log.get() {
                        debug!("next navigable -> same as start");
                    }
                    return n;
                }
            }
        }

        /// Previous navigable flag, starting at start.
        fn prev_navigable(&self, start: usize) -> usize {
            if self.log.get() {
                debug!("prev navigable {:?}", self.focus[start].name());
            }

            let mut n = start;
            loop {
                n = if n > 0 { n - 1 } else { self.len() - 1 };
                if self.navigable[n] {
                    if self.log.get() {
                        debug!("prev navigable -> {:?}", self.focus[n].name());
                    }
                    return n;
                }
                if n == start {
                    if self.log.get() {
                        debug!("prev navigable -> same as start");
                    }
                    return n;
                }
            }
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for Focus {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        match event {
            ct_event!(keycode press Esc) => {
                if !self.use_primary_keys() {
                    if self.core.log.get() {
                        debug!("Esc {:?}", self.focused());
                    }
                    let r = self.next().into();
                    if self.core.log.get() {
                        debug!("=> {:?}", self.focused());
                    }
                    r
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(keycode press Tab) => {
                if self.use_primary_keys() {
                    if self.core.log.get() {
                        debug!("Tab {:?}", self.focused());
                    }
                    let r = self.next().into();
                    if self.core.log.get() {
                        debug!("=> {:?}", self.focused());
                    }
                    r
                } else {
                    Outcome::Continue
                }
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                if self.use_primary_keys() {
                    if self.core.log.get() {
                        debug!("BackTab {:?}", self.focused());
                    }
                    let r = self.prev().into();
                    if self.core.log.get() {
                        debug!("=> {:?}", self.focused());
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
                        debug!("=> {:?}", self.focused());
                    }
                    Outcome::Changed
                } else {
                    if self.core.log.get() {
                        debug!("=> None");
                    }
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
