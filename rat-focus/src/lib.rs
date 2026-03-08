#![doc = include_str!("../readme.md")]

pub mod doc {
    #![doc = include_str!("../doc.md")]
}
mod builder;
mod core;
mod flag;
mod focus;

/// Reexport of types used by a macro.
pub mod ratatui {
    pub mod layout {
        pub use ratatui_core::layout::Rect;
    }
}

pub use crate::builder::FocusBuilder;
pub use crate::flag::FocusFlag;
pub use crate::focus::{Focus, handle_focus};

/// Focus navigation for widgets.
///
/// The effects that hinder focus-change (`Reach*`, `Lock`) only work
/// when navigation changes via next()/prev()/focus_at().
///
/// Programmatic focus changes are always possible.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Navigation {
    /// Widget is not reachable with normal keyboard or mouse navigation.
    None,
    /// Focus is locked to stay with this widget. No mouse or keyboard navigation
    /// can change that.
    Lock,
    /// Widget is not reachable with keyboard navigation, but can be focused with the mouse.
    Mouse,
    /// Widget cannot be reached with normal keyboard navigation, but can be left.
    /// (e.g. Tabs, Split-Divider)
    Leave,
    /// Widget can be reached with normal keyboard navigation, but not left.
    /// (e.g. TextArea)
    Reach,
    /// Widget can be reached with normal keyboard navigation, but only be left with
    /// backward navigation.
    /// (e.g. some widget with internal structure)
    ReachLeaveFront,
    /// Widget can be reached with normal keyboard navigation, but only be left with
    /// forward navigation.
    /// (e.g. some widget with internal structure)
    ReachLeaveBack,
    /// Widget can be reached and left with normal keyboard and mouse navigation.
    #[default]
    Regular,
}

/// Trait for a widget that takes part of focus handling.
///
/// When used for a simple widget implement
/// - build()
/// - focus()
/// - area()
///
/// and optionally
///
/// - area_z() and navigable()
///
/// ```rust no_run
/// use rat_focus::ratatui::layout::Rect;
/// use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
///
/// struct MyWidgetState { pub focus: FocusFlag, pub area: Rect }
///
/// impl HasFocus for MyWidgetState {
///     fn build(&self, builder: &mut FocusBuilder) {
///         builder.leaf_widget(self);
///     }
///
///     fn focus(&self) -> FocusFlag {
///         self.focus.clone()
///     }
///
///     fn area(&self) -> Rect {
///         self.area
///     }
/// }
/// ```
///
///
/// When used for a container widget implement
/// - build()
/// ```rust no_run
/// use rat_focus::ratatui::layout::Rect;
/// use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
///
/// struct MyWidgetState { pub focus: FocusFlag, pub area: Rect }
/// # impl HasFocus for MyWidgetState {
/// #     fn build(&self, builder: &mut FocusBuilder) {
/// #         builder.leaf_widget(self);
/// #     }
/// #
/// #     fn focus(&self) -> FocusFlag {
/// #         self.focus.clone()
/// #     }
/// #
/// #     fn area(&self) -> Rect {
/// #         self.area
/// #     }
/// # }
/// struct SomeWidgetState { pub focus: FocusFlag, pub area: Rect, pub component_a: MyWidgetState, pub component_b: MyWidgetState }
///
/// impl HasFocus for SomeWidgetState {
///     fn build(&self, builder: &mut FocusBuilder) {
///         let tag = builder.start(self);
///         builder.widget(&self.component_a);
///         builder.widget(&self.component_b);
///         builder.end(tag);
///     }
///
///     fn focus(&self) -> FocusFlag {
///         self.focus.clone()
///     }
///
///     fn area(&self) -> Rect {
///         self.area
///     }
/// }
/// ```
/// Creates a container with an identity.
///
/// Or
/// ```rust no_run
/// use rat_focus::ratatui::layout::Rect;
/// use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
///
/// struct MyWidgetState { pub focus: FocusFlag, pub area: Rect }
/// # impl HasFocus for MyWidgetState {
/// #     fn build(&self, builder: &mut FocusBuilder) {
/// #         builder.leaf_widget(self);
/// #     }
/// #
/// #     fn focus(&self) -> FocusFlag {
/// #         self.focus.clone()
/// #     }
/// #
/// #     fn area(&self) -> Rect {
/// #         self.area
/// #     }
/// # }
/// struct SomeWidgetState { pub focus: FocusFlag, pub area: Rect, pub component_a: MyWidgetState, pub component_b: MyWidgetState }
///
/// impl HasFocus for SomeWidgetState {
///     fn build(&self, builder: &mut FocusBuilder) {
///         builder.widget(&self.component_a);
///         builder.widget(&self.component_b);
///     }
///
///     fn focus(&self) -> FocusFlag {
///         unimplemented!("not in use")
///     }
///
///     fn area(&self) -> Rect {
///         unimplemented!("not in use")
///     }
/// }
/// ```
/// for an anonymous container.
///
/// focus(), area() and area_z() are only used for the first case.
/// navigable() is ignored for containers, leave it at the default.
///
pub trait HasFocus {
    /// Build the focus-structure for the container/widget.
    fn build(&self, builder: &mut FocusBuilder);

    /// Build the focus-structure for the container/widget.
    ///
    /// This function is called when the default navigation is
    /// overridden by calling [FocusBuilder::widget_navigate].
    /// You only need to implement this function, if you have
    /// a container-widget, that wants to react to an
    /// alternate navigation.
    ///
    /// For regular widgets this will be called too, but
    /// the overridden flag will be used by Focus, regardless
    /// of what you do. It's only useful to get a notification
    /// of an alternate navigation.
    ///
    /// It defaults to calling build. If you don't have very
    /// specific requirements, you need not concern with this;
    /// just implement [HasFocus::build].
    #[allow(unused_variables)]
    fn build_nav(&self, navigable: Navigation, builder: &mut FocusBuilder) {
        self.build(builder);
    }

    /// Access to the focus flag.
    fn focus(&self) -> FocusFlag;

    /// Provide a unique id for the widget.
    fn id(&self) -> usize {
        self.focus().widget_id()
    }

    /// Area for mouse focus.
    ///
    /// Generally, this area shouldn't overlap with other areas.
    /// If it does, you can use `area_z()` to give an extra z-value
    /// for mouse interactions. Default is 0, higher values mean
    /// `above`.
    /// If two areas with the same z overlap, the last one will
    /// be used.
    fn area(&self) -> ratatui::layout::Rect;

    /// Z-value for the area.
    ///
    /// When testing for mouse interactions the z-value is taken into account.
    fn area_z(&self) -> u16 {
        0
    }

    /// Declares how the widget interacts with focus.
    ///
    /// Default is [Navigation::Regular].
    fn navigable(&self) -> Navigation {
        Navigation::Regular
    }

    /// Does this widget have the focus.
    /// Or, if the flag is used for a container, does any of
    /// widget inside the container have the focus.
    ///
    /// This flag is set by [Focus::handle].
    fn is_focused(&self) -> bool {
        self.focus().get()
    }

    /// This widget just lost the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_lost!](crate::on_lost!)
    fn lost_focus(&self) -> bool {
        self.focus().lost()
    }

    /// This widget just gained the focus. This flag is set by [Focus::handle]
    /// if there is a focus transfer, and will be reset by the next
    /// call to [Focus::handle].
    ///
    /// See [on_gained!](crate::on_gained!)
    fn gained_focus(&self) -> bool {
        self.focus().gained()
    }

    /// This flag is set by [Focus::handle], if a mouse-event
    /// matches one of the areas associated with a widget.
    ///
    /// > It searches all containers for an area-match. All
    /// matching areas will have the flag set.
    /// If an area with a higher z is found, all previously
    /// found areas are discarded.
    ///
    /// > The z value for the last container is taken as a baseline.
    /// Only widgets with a z greater or equal are considered.
    /// If multiple widget areas are matching, the last one
    /// will get the flag set.
    ///
    /// This rules enable popup-windows with complex ui's.
    /// The popup-container starts with a z=1 and all widgets
    /// within also get the same z. With the given rules, all
    /// widgets underneath the popup are ignored.
    ///
    /// * This flag starts with a default `true`. This allows
    ///   widgets to work, even if Focus is not used.
    /// * Mouse drag events are not bound to any area.
    ///   Instead, they set the mouse-focus to true for all
    ///   widgets and containers.
    fn has_mouse_focus(&self) -> bool {
        self.focus().mouse_focus()
    }
}

/// Does a match on the state struct of a widget. If `widget_state.lost_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocus],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_lost;
///
/// on_lost!(
///     state.field1 => {
///         // do checks
///     },
///     state.field2 => {
///         // do checks
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_lost {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocus;
        $(if $field.lost_focus() { _ = $validate })*
    }};
}

/// Does a match on the state struct of a widget. If `widget_state.gained_focus()` is true
/// the block is executed. This requires that `widget_state` implements [HasFocus],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::on_gained;
///
/// on_gained!(
///     state.field1 => {
///         // do prep
///     },
///     state.field2 => {
///         // do prep
///     }
/// );
/// ```
#[macro_export]
macro_rules! on_gained {
    ($($field:expr => $validate:expr),*) => {{
        use $crate::HasFocus;
        $(if $field.gained_focus() { _ = $validate })*
    }};
}

/// Does a match on several fields and can return a result.
/// Does a `widget_state.is_focused()` for each field and returns
/// the first that is true. There is an `else` branch too.
///
/// This requires that `widget_state` implements [HasFocus],
/// but that's the basic requirement for this whole crate.
///
/// ```rust ignore
/// use rat_focus::match_focus;
///
/// let res = match_focus!(
///     state.field1 => {
///         // do this
///         true
///     },
///     state.field2 => {
///         // do that
///         true
///     },
///     else => {
///         false
///     }
/// );
///
/// if res {
///     // react
/// }
/// ```
///
#[macro_export]
macro_rules! match_focus {
    ($($field:expr => $block:expr),* $(, else => $final:expr)?) => {{
        use $crate::HasFocus;
        if false {
            unreachable!();
        }
        $(else if $field.is_focused() { $block })*
        $(else { $final })?
    }};
}

/// Create the implementation of HasFocus for the
/// given list of struct members.
///
/// Create a container with no identity.
/// ```
/// # use rat_focus::{impl_has_focus, FocusFlag};
/// # struct MyState { field1: FocusFlag, field2: FocusFlag, field3: FocusFlag }
/// impl_has_focus!(field1, field2, field3 for MyState);
/// ```
///
/// Create a container with an identity.
/// ```
/// # use rat_focus::{impl_has_focus, FocusFlag};
/// # struct MyState { container: FocusFlag, field1: FocusFlag, field2: FocusFlag, field3: FocusFlag }
/// impl_has_focus!(container: field1, field2, field3 for MyState);
/// ```
///
/// Create a container with an identity and an area that will react to mouse clicks.
/// ```
/// # use rat_focus::ratatui::layout::Rect;
/// # use rat_focus::{impl_has_focus, FocusFlag};
/// # struct MyState { container: FocusFlag, area: Rect, field1: FocusFlag, field2: FocusFlag, field3: FocusFlag }
/// impl_has_focus!(container:area: field1, field2, field3 for MyState);
/// ```
#[macro_export]
macro_rules! impl_has_focus {
    ($cc:ident:$area:ident: $($n:ident),* for $ty:ty) => {
        impl $crate::HasFocus for $ty {
            fn build(&self, builder: &mut $crate::FocusBuilder) {
                let tag = builder.start(self);
                $(builder.widget(&self.$n);)*
                builder.end(tag);
            }

            fn focus(&self) -> $crate::FocusFlag {
                self.$cc.clone()
            }

            fn area(&self) -> $crate::ratatui::layout::Rect {
                self.$area
            }
        }
    };
    ($cc:ident: $($n:ident),* for $ty:ty) => {
        impl $crate::HasFocus for $ty {
            fn build(&self, builder: &mut $crate::FocusBuilder) {
                let tag = builder.start(self);
                $(builder.widget(&self.$n);)*
                builder.end(tag);
            }

            fn focus(&self) -> $crate::FocusFlag {
                self.$cc.clone()
            }

            fn area(&self) -> $crate::ratatui::layout::Rect {
                $crate::ratatui::layout::Rect::default()
            }
        }
    };
    ($($n:ident),* for $ty:ty) => {
        impl $crate::HasFocus for $ty {
            fn build(&self, builder: &mut $crate::FocusBuilder) {
                $(builder.widget(&self.$n);)*
            }

            fn focus(&self) -> $crate::FocusFlag {
                unimplemented!("not defined")
            }

            fn area(&self) -> $crate::ratatui::layout::Rect {
                unimplemented!("not defined")
            }
        }
    };
}
