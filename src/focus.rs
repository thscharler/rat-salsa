use crate::{FocusFlag, HasFocus, HasFocusFlag, ZRect};
use log::debug;
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly, Outcome};
use ratatui::layout::Rect;
use std::cell::Cell;
use std::iter::Zip;
use std::{ptr, vec};

/// Keeps track of the focus.
///
/// It works by adding a [FocusFlag] to the State of a widget.
/// Focus is constructed with a list of references to these flags
/// and switches the focus that way. Each widget stays separate otherwise
/// and can pull its focus state from this struct.
///
/// ```rust ignore
/// use rat_focus::Focus;
///
/// let f = Focus::new(&[
///     &widget1,
///     &widget2
/// ]).handle(evt, FocusKeys);
/// ```
///
/// The result `f` indicates whether the focus has changed.
#[derive(Debug, Default)]
pub struct Focus<'a> {
    /// Name for the cycle.
    name: String,
    /// Focus logging
    log: Cell<bool>,

    /// Area for the whole compound. Only valid if container_focus is Some().
    area: Rect,
    /// Area split in regions. Only valid if container_focus is Some().
    z_area: &'a [ZRect],
    /// Summarizes all the contained FocusFlags.
    /// If any of them has the focus set, this will be set too.
    /// This can help if you build compound widgets.
    container_focus: Option<&'a FocusFlag>,

    /// Areas for each widget.
    areas: Vec<Rect>,
    /// Areas for each widget split in regions.
    z_areas: Vec<&'a [ZRect]>,
    /// Keyboard navigable
    navigable: Vec<bool>,
    /// List of flags.
    focus: Vec<&'a FocusFlag>,

    /// List of sub-containers and their dependencies.
    ///
    /// This is filled if you call [Focus::add_focus]. The
    /// container_focus of the appended Focus and all its focus-flags
    /// are added. And all the sub_container's of it are appended too.
    sub_container: Vec<(&'a FocusFlag, Vec<&'a FocusFlag>)>,
}

impl<'a> IntoIterator for Focus<'a> {
    type Item = (&'a FocusFlag, Rect);
    type IntoIter = Zip<vec::IntoIter<&'a FocusFlag>, vec::IntoIter<Rect>>;

    fn into_iter(self) -> Self::IntoIter {
        self.focus.into_iter().zip(self.areas)
    }
}

impl<'a> Focus<'a> {
    /// Construct a new focus list.
    pub fn new(list: &[&'a dyn HasFocusFlag]) -> Self {
        let mut s = Focus::default();
        for f in list {
            s.focus.push(f.focus());
            s.areas.push(f.area());
            s.z_areas.push(f.z_areas());
            s.navigable.push(f.navigable());
        }
        s
    }

    /// Construct a new focus list for a container widget.
    ///
    /// The focus-flag for the container accumulates all the flags.
    /// If any has focus, the container has the focus too.
    /// Focus-gained and focus-lost are only set if the focus
    /// leaves the container.
    ///
    /// The container widget itself interacts with the mouse too.
    /// If no single widget is hit with the mouse, but the area of
    /// the container is, the first widget gets the focus.
    ///
    /// See `examples/focus_recursive` and `examples/focus_recursive2`
    pub fn new_accu(container: &'a dyn HasFocusFlag, list: &[&'a dyn HasFocusFlag]) -> Self {
        let mut s = Self {
            container_focus: Some(container.focus()),
            area: container.area(),
            z_area: container.z_areas(),
            ..Focus::default()
        };
        for f in list {
            s.focus.push(f.focus());
            s.areas.push(f.area());
            s.z_areas.push(f.z_areas());
            s.navigable.push(f.navigable());
        }
        s
    }

    /// Construct a new focus list with group accumulator.
    ///
    /// This is meant for some loose grouping of widgets, for which
    /// you want an overview.
    ///
    /// The same rules apply as for new_accu(), but with this one
    /// there is no overall area for mouse interaction.
    pub fn new_grp(grp: &'a FocusFlag, list: &[&'a dyn HasFocusFlag]) -> Self {
        let mut s = Self {
            container_focus: Some(grp),
            area: Default::default(),
            z_area: Default::default(),
            ..Focus::default()
        };
        for f in list {
            s.focus.push(f.focus());
            s.areas.push(f.area());
            s.z_areas.push(f.z_areas());
            s.navigable.push(f.navigable());
        }
        s
    }

    /// Add a single widget.
    /// This doesn't add any z_areas and assumes navigable is true.
    pub fn add_flag(&mut self, flag: &'a FocusFlag, area: Rect) -> &mut Self {
        self.focus.push(flag);
        self.areas.push(area);
        self.z_areas.push(&[]);
        self.navigable.push(true);
        self
    }

    /// Add a sub-focus cycle.
    ///
    /// All its widgets are appended to this list. If the sub-cycle
    /// has an accumulator it's added to the sub-accumulators. All
    /// sub-sub-accumulators are appended too.
    pub fn add_focus(&mut self, focus: Focus<'a>) -> &mut Self {
        for (focus, list) in focus.sub_container {
            self.sub_container.push((focus, list));
        }
        if let Some(accu) = focus.container_focus {
            self.sub_container.push((accu, focus.focus.clone()))
        }
        self.focus.extend(focus.focus);
        self.areas.extend(focus.areas);
        self.z_areas.extend(focus.z_areas);
        self.navigable.extend(focus.navigable);
        self
    }

    /// Add a container widget.
    pub fn add_container(&mut self, container: &'a dyn HasFocus) -> &mut Self {
        self.add_focus(container.focus());
        self
    }

    /// Add a single widget.
    pub fn add(&mut self, f: &'a dyn HasFocusFlag) -> &mut Self {
        self.focus.push(f.focus());
        self.areas.push(f.area());
        self.z_areas.push(f.z_areas());
        self.navigable.push(f.navigable());
        self
    }

    /// Add a list of widgets.
    pub fn add_all(&mut self, list: &[&'a dyn HasFocusFlag]) -> &mut Self {
        for f in list {
            self.focus.push(f.focus());
            self.areas.push(f.area());
            self.z_areas.push(f.z_areas());
            self.navigable.push(f.navigable());
        }
        self
    }

    /// Writes a log for each operation.
    pub fn enable_log(&self, log: bool) {
        self.log.set(log)
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
    /// This ensures that there is only one focused widget.
    /// The first widget in the list gets the focus.
    pub fn init(&self) {
        if self.log.get() {
            debug!("init focus");
        }
        self.core_init();
    }

    /// Clears the focus state for all widgets.
    /// This is useful, if part of your widgets are temporarily
    /// exempt from focus handling, and should therefore not
    /// have any focus-flags set to avoid problems with
    /// event-handling.
    pub fn clear(&self) {
        if self.log.get() {
            debug!("clear focus");
        }
        self.core_clear();
    }

    /// Sets the focus to the widget.
    ///
    /// Sets focus and gained but not lost. This can be used to prevent validation of the field.
    pub fn focus_widget_no_lost(&self, state: &'a dyn HasFocusFlag) {
        let flag = state.focus();
        self.focus_no_lost(flag);
    }

    /// Sets the focus to the widget with `tag`.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    pub fn focus_widget(&self, state: &'a dyn HasFocusFlag) {
        let flag = state.focus();
        self.focus(flag);
    }

    /// Sets the focus to the widget.
    ///
    /// Sets focus and gained but not lost. This can be used to prevent validation of the field.
    pub fn focus_no_lost(&self, flag: &FocusFlag) {
        if self.log.get() {
            debug!("focus_no_lost {:?}", flag);
        }
        self.core_focus_no_lost(flag);
    }

    /// Sets the focus to the widget with `tag`.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are not set.
    pub fn focus(&self, flag: &FocusFlag) {
        if self.log.get() {
            debug!("focus {:?}", flag);
        }
        self.core_focus(flag);
    }

    /// Returns the focused widget as FocusFlag.
    ///
    /// This is mainly for debugging purposes.
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    pub fn focused(&self) -> Option<&'a FocusFlag> {
        self.core_focused()
    }

    /// Returns the widget that lost the focus as FocusFlag.
    ///
    /// This is mainly for debugging purposes.
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    pub fn lost_focus(&self) -> Option<&'a FocusFlag> {
        self.core_lost_focus()
    }

    /// Returns the widget that gained the focus as FocusFlag.
    ///
    /// This is mainly for debugging purposes.
    /// For control-flow [crate::match_focus] or [crate::on_gained] or [crate::on_lost]
    /// will be nicer.
    pub fn gained_focus(&self) -> Option<&'a FocusFlag> {
        self.core_gained_focus()
    }

    /// Reset lost + gained flags.
    /// This is done automatically in `HandleEvent::handle()` for every event.
    pub fn reset_lost_gained(&self) {
        if self.log.get() {
            debug!("reset_lost_gained");
        }
        self.core_reset_lost_gained();
    }

    /// Change the focus.
    ///
    /// Sets the focus, gained and lost flags.
    ///
    /// If the field at idx has the focus all three are set.
    pub fn focus_idx(&self, idx: usize) {
        if self.log.get() {
            debug!("focus_idx {}", idx);
        }
        self.core_focus_idx(idx);
    }

    /// Change to focus to the given position.
    pub fn focus_at(&self, col: u16, row: u16) -> bool {
        if self.log.get() {
            debug!("focus_at {},{}", col, row);
        }
        self.core_focus_at(col, row)
    }

    /// Focus the next widget in the cycle.
    ///
    /// Sets the focus, gained and lost flags. If this ends up with the same widget as
    /// before focus, gained and lost flag are all set.
    ///
    /// If no field has the focus the first one gets it.
    pub fn next(&self) -> bool {
        if self.log.get() {
            debug!("next {:?}", self.core_focused());
        }
        self.core_next()
    }

    /// Focus the previous widget in the cycle.
    ///
    /// Sets the focus and lost flags. If this ends up with the same widget as
    /// before it returns *true* and sets the focus, gained and lost flag.
    ///
    /// If no field has the focus the first one gets it.
    pub fn prev(&self) -> bool {
        if self.log.get() {
            debug!("prev {:?}", self.core_focused());
        }
        self.core_prev()
    }
}

impl<'a> Focus<'a> {
    // reset flags for a new round.
    fn core_start_focus_change(&self, set_lost: bool) {
        if self.log.get() {
            debug!("start_focus_change {}", set_lost);
        }
        for p in self.focus.iter() {
            if set_lost {
                p.lost.set(p.focus.get());
            } else {
                p.lost.set(false);
            }
            p.gained.set(false);
            p.focus.set(false);
        }
    }

    // accumulate everything
    fn core_accumulate(&self) {
        if let Some(accu) = self.container_focus {
            accu.focus.set(false);
            for p in self.focus.iter() {
                if p.focus.get() {
                    accu.focus.set(true);
                    break;
                }
            }
        }

        for (f, list) in &self.sub_container {
            let mut any_gained = false;
            let mut any_lost = false;
            let mut any_focused = false;

            for f in list {
                any_gained |= f.gained.get();
                any_lost |= f.lost.get();
                any_focused |= f.focus.get();
            }

            f.focus.set(any_focused);
            f.lost.set(any_lost && !any_gained);
            f.gained.set(any_gained && !any_lost);
        }
    }

    fn core_init(&self) {
        if let Some(n) = self.core_first_navigable_idx() {
            self.core_focus_idx(n);
        }
    }

    fn core_clear(&self) {
        if let Some(v) = self.container_focus {
            v.clear()
        }
        for f in &self.focus {
            f.clear();
        }
        for (f, _) in &self.sub_container {
            f.clear();
        }
    }

    fn core_focus_no_lost(&self, flag: &FocusFlag) {
        self.core_start_focus_change(false);
        if let Some(f) = self.focus.iter().find(|f| ptr::eq(**f, flag)) {
            f.focus.set(true);
            f.gained.set(false);
        }
        self.core_accumulate();
    }

    fn core_focus(&self, flag: &FocusFlag) {
        self.core_start_focus_change(true);
        if let Some(f) = self.focus.iter().find(|f| ptr::eq(**f, flag)) {
            f.focus.set(true);
            if f.lost.get() {
                // lost == gained -> noop
                f.lost.set(false);
            } else {
                f.gained.set(true);
            }
        }
        self.core_accumulate();
    }

    fn core_reset_lost_gained(&self) {
        for p in self.focus.iter() {
            p.lost.set(false);
            p.gained.set(false);
        }
        for (p, _) in self.sub_container.iter() {
            p.gained.set(false);
            p.lost.set(false);
        }
    }

    fn core_focus_idx(&self, idx: usize) {
        self.core_start_focus_change(true);
        if let Some(f) = self.focus.get(idx) {
            f.focus.set(true);
            if f.lost.get() {
                // lost == gained -> noop
                f.lost.set(false);
            } else {
                f.gained.set(true);
            }
        }
        self.core_accumulate();
    }

    fn core_focus_at(&self, col: u16, row: u16) -> bool {
        let pos = (col, row).into();

        let mut z_order = Vec::new();
        for (idx, area) in self.areas.iter().enumerate() {
            if area.contains(pos) {
                if self.log.get() {
                    debug!(
                        "found area for {:?}, check {:?}",
                        self.focus[idx], self.z_areas[idx]
                    );
                }
                // check for split
                if !self.z_areas[idx].is_empty() {
                    for (z_idx, z) in self.z_areas[idx].into_iter().enumerate() {
                        // use all matching areas. might differ in z.
                        if z.contains(pos) {
                            if self.log.get() {
                                debug!("found z-area for {:?}+{:?}", self.focus[idx], z_idx);
                            }
                            z_order.push((idx, z.z));
                        }
                    }
                } else {
                    z_order.push((idx, 0));
                }
            }
        }

        // process in order, last is on top if more than one.
        if let Some(max_last) = z_order.iter().max_by(|v, w| v.1.cmp(&w.1)) {
            self.core_focus_idx(max_last.0);
            return true;
        }

        // todo: miss container focus for sub-containers???

        // todo: weakly reasoned
        if self.area.contains(pos.into()) {
            if self.log.get() {
                debug!("focus container {:?}", self.container_focus);
            }
            // if disjointed areas exist, value them.
            if !self.z_area.is_empty() {
                for (z_idx, z) in self.z_area.into_iter().enumerate() {
                    // use all matching areas. might differ in z.
                    if z.contains(pos) {
                        if self.log.get() {
                            debug!(
                                "focus container z-area {:?}+{:?}",
                                self.container_focus, z_idx
                            );
                        }
                        if let Some(n) = self.core_first_navigable_idx() {
                            self.core_focus_idx(n);
                            return true;
                        }
                    }
                }
                // if only the main area matched, this is a dud.
            } else {
                // only a main area, fine.
                if let Some(n) = self.core_first_navigable_idx() {
                    self.core_focus_idx(n);
                    return true;
                }
            }
        }

        false
    }

    fn core_next(&self) -> bool {
        for (i, p) in self.focus.iter().enumerate() {
            if p.lost.get() {
                let n = self.core_next_navigable_idx(i);
                self.core_focus_idx(n);
                return true;
            }
        }
        if let Some(n) = self.core_first_navigable_idx() {
            self.core_focus_idx(n);
            return true;
        }
        false
    }

    fn core_prev(&self) -> bool {
        for (i, p) in self.focus.iter().enumerate() {
            if p.lost.get() {
                let n = self.core_prev_navigable(i);
                self.core_focus_idx(n);
                return true;
            }
        }
        if let Some(n) = self.core_first_navigable_idx() {
            self.core_focus_idx(n);
            return true;
        }
        false
    }

    fn core_first_navigable_idx(&self) -> Option<usize> {
        let mut n = 0;
        loop {
            n = next_circular(n, self.focus.len());
            if self.navigable[n] {
                return Some(n);
            }
            if n == 0 {
                return None;
            }
        }
    }

    fn core_next_navigable_idx(&self, start: usize) -> usize {
        let mut n = start;
        loop {
            n = next_circular(n, self.focus.len());
            if self.navigable[n] {
                return n;
            }
            if n == start {
                return n;
            }
        }
    }

    fn core_prev_navigable(&self, start: usize) -> usize {
        let mut n = start;
        loop {
            n = prev_circular(n, self.focus.len());
            if self.navigable[n] {
                return n;
            }
            if n == start {
                return n;
            }
        }
    }

    fn core_focused(&self) -> Option<&'a FocusFlag> {
        self.focus.iter().find(|v| v.get()).copied()
    }

    fn core_lost_focus(&self) -> Option<&'a FocusFlag> {
        self.focus.iter().find(|v| v.lost()).copied()
    }

    fn core_gained_focus(&self) -> Option<&'a FocusFlag> {
        self.focus.iter().find(|v| v.gained()).copied()
    }
}

/// Next but circle around.
fn next_circular(select: usize, max: usize) -> usize {
    if select + 1 < max {
        select + 1
    } else {
        0
    }
}

/// Prev but circle around.
fn prev_circular(select: usize, max: usize) -> usize {
    if select > 0 {
        select - 1
    } else {
        max.saturating_sub(1)
    }
}

impl<'a> HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for Focus<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        match event {
            ct_event!(keycode press Tab) => {
                if self.log.get() {
                    debug!("Tab {:?}", self.core_focused());
                }
                self.core_next();
                if self.log.get() {
                    debug!("=> {:?}", self.core_focused());
                }
                Outcome::Changed
            }
            ct_event!(keycode press SHIFT-Tab) | ct_event!(keycode press SHIFT-BackTab) => {
                if self.log.get() {
                    debug!("BackTab {:?}", self.core_focused());
                }
                self.core_prev();
                if self.log.get() {
                    debug!("=> {:?}", self.core_focused());
                }
                Outcome::Changed
            }
            _ => self.handle(event, MouseOnly),
        }
    }
}

impl<'a> HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for Focus<'a> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse down Left for column, row) => {
                if self.log.get() {
                    debug!("mouse down {},{}", column, row);
                }
                if self.core_focus_at(*column, *row) {
                    if self.log.get() {
                        debug!("=> {:?}", self.core_focused());
                    }
                    Outcome::Changed
                } else {
                    if self.log.get() {
                        debug!("=> None");
                    }
                    self.core_reset_lost_gained();
                    Outcome::NotUsed
                }
            }
            _ => {
                self.core_reset_lost_gained();
                Outcome::NotUsed
            }
        }
    }
}
