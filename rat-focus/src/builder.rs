use crate::core::{Container, FocusCore};
use crate::{Focus, FocusFlag, HasFocus, Navigation};
use fxhash::FxBuildHasher;
use ratatui_core::layout::Rect;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;

macro_rules! focus_debug {
    ($core:expr, $($arg:tt)+) => {
        if $core.log.get() {
            log::log!(log::Level::Debug, $($arg)+);
        }
    }
}

/// Builder for the Focus.
#[derive(Debug, Default)]
pub struct FocusBuilder {
    last: Option<FocusCore>,

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
    tmp_widget: Vec<(usize, u16)>,
    tmp_container: Vec<(usize, u16)>,
}

impl FocusBuilder {
    /// Create a FocusBuilder with old data directly from a FocusCore.
    pub(crate) fn new_from_core(last: FocusCore) -> FocusBuilder {
        Self {
            last: Some(last),
            ..Default::default()
        }
    }

    /// Create a new FocusBuilder.
    ///
    /// This can take the previous Focus and ensures that
    /// widgets that are no longer part of the focus list
    /// have their focus-flag cleared.
    ///
    /// It will also recycle the storage of the old Focus.
    pub fn new(last: Option<Focus>) -> FocusBuilder {
        if let Some(last) = last {
            // flip the cores
            let (last, mut core) = {
                let Focus { last, core, .. } = last;
                (core.take(), last.take())
            };

            // clear storage, don't reset anything.
            core.clear();

            Self {
                last: Some(last),
                log: Default::default(),
                z_base: 0,
                focus_ids: core.focus_ids,
                focus_flags: core.focus_flags,
                duplicate: core.duplicate,
                areas: core.areas,
                navigable: core.navigable,
                container_ids: core.container_ids,
                containers: core.containers,
                tmp_widget: core.tmp_widget.take(),
                tmp_container: core.tmp_container.take(),
            }
        } else {
            Self::default()
        }
    }

    /// The same as build_for but with logs enabled.
    pub fn log_build_for(container: &dyn HasFocus) -> Focus {
        let mut b = FocusBuilder::new(None);
        b.enable_log();
        b.widget(container);
        b.build()
    }

    /// Shortcut for building the focus for a container
    /// that implements [HasFocus].
    ///
    /// This creates a fresh Focus.
    ///
    /// __See__
    ///
    /// Use [rebuild_for](FocusBuilder::rebuild_for) if you want
    /// to ensure that widgets that are no longer in the widget
    /// structure have their focus flag reset properly. If you
    /// don't have some logic to conditionally add widgets to
    /// the focus, this function is probably fine.
    pub fn build_for(container: &dyn HasFocus) -> Focus {
        let mut b = FocusBuilder::new(None);
        b.widget(container);
        b.build()
    }

    /// The same as rebuild_for but with logs enabled.
    pub fn log_rebuild_for(container: &dyn HasFocus, old: Option<Focus>) -> Focus {
        let mut b = FocusBuilder::new(old);
        b.enable_log();
        b.widget(container);
        b.build()
    }

    /// Shortcut function for building the focus for a container
    /// that implements [HasFocus]
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

    /// Enable insta-panic for certain failures.
    #[deprecated(since = "2.3.0", note = "no longer in use")]
    pub fn enable_panic(&self) {}

    /// Disable insta-panic for certain failures.
    #[deprecated(since = "2.3.0", note = "no longer in use")]
    pub fn disable_panic(&self) {}

    /// Add a widget by calling its `build` function.
    pub fn widget(&mut self, widget: &dyn HasFocus) -> &mut Self {
        widget.build_nav(widget.navigable(), self);
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
    pub fn widget_navigate(&mut self, widget: &dyn HasFocus, navigation: Navigation) -> &mut Self {
        if widget.build_nav(navigation, self) {
            self.override_navigation(widget.focus(), navigation);
        }
        self
    }

    fn override_navigation(&mut self, flag: FocusFlag, navigation: Navigation) {
        // override navigation for the widget
        if let Some(idx) = self.focus_flags.iter().position(|v| *v == flag) {
            focus_debug!(
                self,
                "override navigation for {:?} with {:?}",
                flag,
                navigation
            );
            self.navigable[idx] = navigation;
        } else {
            if self.container_ids.contains(&flag.widget_id()) {
                focus_debug!(
                    self,
                    "fail to override navigation for {:?}. This is a container.",
                    flag,
                );
            } else {
                focus_debug!(
                    self,
                    "fail to override navigation for {:?}. Widget doesn't use this focus-flag",
                    flag,
                );
            }
        }
    }

    /// Add a bunch of widgets.
    #[inline]
    pub fn widgets<const N: usize>(&mut self, widgets: [&dyn HasFocus; N]) -> &mut Self {
        for widget in widgets {
            widget.build_nav(widget.navigable(), self);
        }
        self
    }

    /// Start a container widget.
    ///
    /// Uses focus(), area() and z_area() of the given container.
    /// navigable() is not used for containers.
    ///
    /// Containers can be nested but start/end calls must work
    /// like braces.
    ///
    /// __Return__
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

    /// Start a container widget with manual flags.
    ///
    /// Returns the FocusFlag of the container. This flag must
    /// be used to close the container with [end](Self::end).
    ///
    /// __Return__
    ///
    /// Use the returned value when calling [end](Self::end).
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
        focus_debug!(self, "start container {:?}", container_flag);

        // no duplicates allowed for containers.
        if self.container_ids.contains(&container_flag.widget_id()) {
            panic!("Can't add the same container twice.");
        }

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

    /// End a container widget.
    ///
    /// Must match the corresponding start().
    ///
    /// Containers can be nested, start and end calls must
    /// work like braces.
    ///
    /// __Panic__
    /// If the container has not been started.
    /// If the matchup with start() fails.
    ///
    pub fn end(&mut self, tag: FocusFlag) {
        focus_debug!(self, "end container {:?}", tag);

        if !self.container_ids.contains(&tag.widget_id()) {
            panic!("Container has not been started");
        }

        for (c, r) in self.containers.iter_mut().rev() {
            if c.container_flag != tag {
                if !c.complete {
                    panic!("FocusBuilder: Unclosed container {:?}", c.container_flag);
                }
            } else {
                r.end = self.focus_flags.len();
                c.complete = true;

                focus_debug!(self, "container range {:?}", r);

                self.z_base -= c.delta_z;

                break;
            }
        }
    }

    /// Directly adds the given widgets flags.
    ///
    /// Use this when implementing a widget, and you would write
    /// `builder.widget(self)` which would recurse infinitely.
    ///
    /// In all other situations it's better to use [widget](FocusBuilder::widget)
    /// or [widget_navigate](FocusBuilder::widget_navigate)
    ///
    /// __Panic__
    ///
    /// Panics if the same focus-flag is added twice.
    pub fn leaf_widget(&mut self, widget: &dyn HasFocus) -> &mut Self {
        self.leaf_with_flags(
            widget.focus(),
            widget.area(),
            widget.area_z(),
            widget.navigable(),
        );
        self
    }

    #[deprecated(since = "2.0.2", note = "use leaf_with_flags instead")]
    pub fn widget_with_flags(
        &mut self,
        focus: FocusFlag,
        area: Rect,
        area_z: u16,
        navigable: Navigation,
    ) {
        self.leaf_with_flags(focus, area, area_z, navigable)
    }

    /// Directly adds the given widgets flags.
    ///
    /// Use this when implementing a widget, and you would write
    /// `builder.widget(self)` which would recurse infinitely.
    ///
    /// In all other situations it's better to use [widget](FocusBuilder::widget)
    /// or [widget_navigate](FocusBuilder::widget_navigate)
    ///
    /// __Panic__
    ///
    /// Panics if the same focus-flag is added twice.
    /// Except that it is allowable to add the flag a second time with
    /// Navigation::Mouse or Navigation::None
    pub fn leaf_with_flags(
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
            if !matches!(navigable, Navigation::Mouse | Navigation::None) {
                focus_debug!(self, "{:#?}", self);
                panic!(
                    "Duplicate flag is only allowed if the second call uses Navigation::Mouse|Navigation::None. Was {:?}.",
                    focus
                )
            }
        }

        focus_debug!(self, "widget {:?}", focus);

        self.focus_ids.insert(focus.widget_id());
        self.focus_flags.push(focus);
        self.duplicate.push(duplicate);
        self.areas.push((area, self.z_base + area_z));
        self.navigable.push(navigable);
    }

    // Cleanup widgets no longer in the Focus.
    pub(crate) fn cleanup(&mut self) {
        if let Some(last) = self.last.as_mut() {
            for f in &last.focus_flags {
                if !self.focus_ids.contains(&f.widget_id()) {
                    f.set(false);
                    f.set_lost(false);
                    f.set_gained(false);
                    f.set_obscured(false);
                    f.set_focus_core(None);
                }
            }
            for (f, _) in &last.containers {
                if !self.container_ids.contains(&f.container_flag.widget_id()) {
                    f.container_flag.set(false);
                    f.container_flag.set_lost(false);
                    f.container_flag.set_gained(false);
                    f.container_flag.set_obscured(false);
                    f.container_flag.set_focus_core(None);
                }
            }

            last.clear();
        }
    }

    // Build the new core.
    pub(crate) fn build_core(self) -> FocusCore {
        // check new tree.
        for (c, _) in self.containers.iter().rev() {
            if !c.complete {
                panic!("FocusBuilder: Unclosed container {:?}", c.container_flag);
            }
        }

        // build new core
        FocusCore {
            log: Default::default(),
            focus_ids: self.focus_ids,
            focus_flags: self.focus_flags,
            duplicate: self.duplicate,
            areas: self.areas,
            navigable: self.navigable,
            container_ids: self.container_ids,
            containers: self.containers,
            tmp_widget: Cell::new(self.tmp_widget),
            tmp_container: Cell::new(self.tmp_container),
        }
    }

    /// Build the Focus.
    ///
    /// If the previous Focus is known, this will also
    /// reset the FocusFlag for any widget no longer part of
    /// the Focus.
    pub fn build(mut self) -> Focus {
        // cleanup outcasts.
        self.cleanup();

        let last = if let Some(last) = self.last.take() {
            Rc::new(RefCell::new(last))
        } else {
            Default::default()
        };
        let core = Rc::new(RefCell::new(self.build_core()));
        {
            // set back references
            let borrowed_core = core.borrow();
            for f in &borrowed_core.focus_flags {
                f.set_focus_core(Some(Rc::clone(&core)));
            }
            for (f, _) in &borrowed_core.containers {
                f.container_flag.set_focus_core(Some(Rc::clone(&core)));
            }
        }

        Focus { last, core }
    }
}
