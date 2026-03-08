use crate::core::{Container, FocusCore};
use crate::{Focus, FocusFlag, HasFocus, Navigation};
use fxhash::FxBuildHasher;
use ratatui_core::layout::Rect;
use std::cell::Cell;
use std::collections::HashSet;
use std::ops::Range;

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

/// Builder for the Focus.
#[derive(Debug, Default)]
pub struct FocusBuilder {
    last: FocusCore,

    log: Cell<bool>,
    insta_panic: Cell<bool>,

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
                insta_panic: Default::default(),
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
                insta_panic: Default::default(),
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
    pub fn enable_panic(&self) {
        self.insta_panic.set(true);
    }

    /// Disable insta-panic for certain failures.
    pub fn disable_panic(&self) {
        self.insta_panic.set(false);
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
    pub fn widget_navigate(&mut self, widget: &dyn HasFocus, navigation: Navigation) -> &mut Self {
        widget.build_nav(navigation, self);

        let widget_flag = widget.focus();
        // override navigation for the widget
        if let Some(idx) = self.focus_flags.iter().position(|v| *v == widget_flag) {
            focus_debug!(
                self,
                "override navigation for {:?} with {:?}",
                widget_flag,
                navigation
            );

            self.navigable[idx] = navigation;
        } else {
            if self.container_ids.contains(&widget_flag.widget_id()) {
                focus_fail!(
                    self,
                    "FAIL to override navigation for {:?}. This is a container.",
                    widget_flag,
                );
            } else {
                focus_fail!(
                    self,
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
        focus_debug!(self, "end container {:?}", tag);
        assert!(self.container_ids.contains(&tag.widget_id()));

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
        focus_debug!(self, "start container {:?}", container_flag);

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
        let insta_panic = self.last.insta_panic.get();

        Focus {
            last: self.last,
            core: FocusCore {
                log: Cell::new(log),
                insta_panic: Cell::new(insta_panic),
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
