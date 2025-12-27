//!
//! SalsaTheme is the main structure for themes.
//!
//! It holds one [Palette] that has the color-table
//! and a list of aliases for colors in the color-table.
//! These aliases allow the palette to give a bit more
//! semantics to its plain color-array.
//!
//! SalsaTheme is on the other end and has a hashmap
//! of style-names that map to
//! * a [Style]
//! * a Fn closure that creates a widget-specific xxxStyle struct.
//!   this closure can use the palette or any previously defined
//!   styles to create the xxxStyle struct.
//!
//! In between is a create-fn that links all of this together.
//!
//! In your application you can use one of the defined Themes/Palettes
//! and modify/extend before you hand it off to your UI code.
//!
//! __Rationale__
//!
//! - Colors are separated from styles. There is an editor `pal-edit`
//!   to create a palette + aliases. It can generate rust code
//!   that can be used as `statíc` data.
//! - There is a `.pal` file-format for this. This format could
//!   be used to load the palette from some configuration.
//! - If you prefer something else, Palette supports serde too.
//!
//! - Themes and xxxStyle structs can contain other things than
//!   colors. `Block` is used often. Alignment and related flags
//!   are available. And there are some flags that modify the
//!   behaviour of widgets.
//! - xxxStyle combine everything in one package, and can be
//!   set with one function call when rendering. You don't need
//!   20 lines of styling functions for each widget.
//!
use crate::is_log_style_define;
use crate::palette::Palette;
use crate::themes::create_fallback;
use log::info;
use ratatui_core::style::Style;
use std::any::{Any, type_name};
use std::collections::{HashMap, hash_map};
use std::fmt::{Debug, Formatter};

trait StyleValue: Any + Debug {}
impl<T> StyleValue for T where T: Any + Debug {}

type Entry = Box<dyn Fn(&SalsaTheme) -> Box<dyn StyleValue> + 'static>;
type Modify = Box<dyn Fn(Box<dyn Any>, &SalsaTheme) -> Box<dyn StyleValue> + 'static>;

///
/// SalsaTheme holds any predefined styles for the UI.  
///
/// The foremost usage is as a store of named [Style](ratatui::style::Style)s.
/// It can also hold the structured styles used by rat-widget's.
/// Or really any value that can be produced by a closure.
///
/// It uses a flat naming scheme and doesn't cascade upwards at all.
pub struct SalsaTheme {
    pub name: String,
    pub theme: String,
    pub p: Palette,
    styles: HashMap<&'static str, Entry>,
    modify: HashMap<&'static str, Modify>,
}

impl Default for SalsaTheme {
    fn default() -> Self {
        create_fallback(Palette::default())
    }
}

impl Debug for SalsaTheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Theme")
            .field("name", &self.name)
            .field("theme", &self.theme)
            .field("palette", &self.p)
            .field("styles", &self.styles.keys().collect::<Vec<_>>())
            .field("modify", &self.modify.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl SalsaTheme {
    /// Create an empty theme with the given color palette.
    pub fn new(p: Palette) -> Self {
        Self {
            name: p.theme_name.as_ref().into(),
            theme: p.theme.as_ref().into(),
            p,
            styles: Default::default(),
            modify: Default::default(),
        }
    }

    /// Some display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Define a style as a plain [Style].
    pub fn define_style(&mut self, name: &'static str, style: Style) {
        let boxed = Box::new(move |_: &SalsaTheme| -> Box<dyn StyleValue> { Box::new(style) });
        self.define(name, boxed);
    }

    /// Define a style a struct that will be cloned for every query.
    pub fn define_clone(&mut self, name: &'static str, sample: impl Clone + Any + Debug + 'static) {
        let boxed = Box::new(move |_th: &SalsaTheme| -> Box<dyn StyleValue> {
            Box::new(sample.clone()) //
        });
        self.define(name, boxed);
    }

    /// Define a style as a call to a constructor fn.
    ///
    /// The constructor gets access to all previously defined styles.
    pub fn define_fn<O: Any + Debug>(
        &mut self,
        name: &'static str,
        create: impl Fn(&SalsaTheme) -> O + 'static,
    ) {
        let boxed = Box::new(move |th: &SalsaTheme| -> Box<dyn StyleValue> {
            Box::new(create(th)) //
        });
        self.define(name, boxed);
    }

    /// Define a style as a call to a constructor fn.
    ///
    /// This one takes no arguments, this is nice to set WidgetStyle::default
    /// as the style-fn.
    pub fn define_fn0<O: Any + Debug>(
        &mut self,
        name: &'static str,
        create: impl Fn() -> O + 'static,
    ) {
        let boxed = Box::new(move |_th: &SalsaTheme| -> Box<dyn StyleValue> {
            Box::new(create()) //
        });
        self.define(name, boxed);
    }

    fn define(&mut self, name: &'static str, boxed: Entry) {
        if is_log_style_define() {
            info!("salsa-style: {:?}", name);
        }
        match self.styles.insert(name, boxed) {
            None => {}
            Some(_) => {
                if is_log_style_define() {
                    info!("salsa-style: OVERWRITE {:?}", name);
                }
            }
        };
    }

    /// Add a modification of a defined style.
    ///
    /// This function is applied to the original style every time the style is queried.
    ///
    /// Currently only a single modification is possible. If you set a second one
    /// it will overwrite the previous.
    ///
    /// __Panic__
    ///
    /// * When debug_assertions are enabled the modifier will panic if
    ///   it gets a type other than `O`.
    /// * Otherwise it will fall back to the default value of `O`.
    ///
    pub fn modify<O: Any + Default + Debug + Sized + 'static>(
        &mut self,
        name: &'static str,
        modify: impl Fn(O, &SalsaTheme) -> O + 'static,
    ) {
        let boxed = Box::new(
            move |v: Box<dyn Any>, th: &SalsaTheme| -> Box<dyn StyleValue> {
                if cfg!(debug_assertions) {
                    let v = match v.downcast::<O>() {
                        Ok(v) => *v,
                        Err(e) => {
                            panic!(
                                "downcast fails for '{}' to {}. Is {:?}",
                                name,
                                type_name::<O>(),
                                e
                            );
                        }
                    };

                    let v = modify(v, th);

                    Box::new(v)
                } else {
                    let v = match v.downcast::<O>() {
                        Ok(v) => *v,
                        Err(_) => O::default(),
                    };

                    let v = modify(v, th);

                    Box::new(v)
                }
            },
        );

        match self.modify.entry(name) {
            hash_map::Entry::Occupied(mut entry) => {
                if is_log_style_define() {
                    info!("salsa-style: overwrite modifier for {:?}", name);
                }
                _ = entry.insert(boxed);
            }
            hash_map::Entry::Vacant(entry) => {
                if is_log_style_define() {
                    info!("salsa-style: set modifier for {:?}", name);
                }
                entry.insert(boxed);
            }
        };
    }

    /// Get one of the defined ratatui-Styles.
    ///
    /// This is the same as the single [style] function, it just
    /// fixes the return-type to [Style]. This is useful if the
    /// receiver is defined as `impl Into<Style>`.
    ///
    /// This may fail:
    ///
    /// __Panic__
    ///
    /// * When debug_assertions are enabled it will panic when
    ///   called with an unknown style name, or if the downcast
    ///   to the out type fails.
    /// * Otherwise, it will return the default value of the out type.
    pub fn style_style(&self, name: &str) -> Style
    where
        Self: Sized,
    {
        self.style::<Style>(name)
    }

    /// Get any of the defined styles.
    ///
    /// It downcasts the stored value to the required out type.
    ///
    /// This may fail:
    ///
    /// __Panic__
    ///
    /// * When debug_assertions are enabled it will panic when
    ///   called with an unknown style name, or if the downcast
    ///   to the out type fails.
    /// * Otherwise, it will return the default value of the out type.
    pub fn style<O: Default + Sized + 'static>(&self, name: &str) -> O
    where
        Self: Sized,
    {
        if cfg!(debug_assertions) {
            let style = match self.dyn_style(name) {
                Some(v) => v,
                None => {
                    panic!("unknown widget {:?}", name)
                }
            };
            let any_style = style as Box<dyn Any>;
            let style = match any_style.downcast::<O>() {
                Ok(v) => v,
                Err(_) => {
                    let style = self.dyn_style(name).expect("style");
                    panic!(
                        "downcast fails for '{}' to {}: {:?}",
                        name,
                        type_name::<O>(),
                        style
                    );
                }
            };
            *style
        } else {
            let Some(style) = self.dyn_style(name) else {
                return O::default();
            };
            let any_style = style as Box<dyn Any>;
            let Ok(style) = any_style.downcast::<O>() else {
                return O::default();
            };
            *style
        }
    }

    /// Get a style struct or the modified variant of it.
    #[allow(clippy::collapsible_else_if)]
    fn dyn_style(&self, name: &str) -> Option<Box<dyn StyleValue>> {
        if let Some(entry_fn) = self.styles.get(name) {
            let mut style = entry_fn(self);
            if let Some(modify) = self.modify.get(name) {
                style = modify(style, self);
            }
            Some(style)
        } else {
            if cfg!(debug_assertions) {
                panic!("unknown style {:?}", name)
            } else {
                None
            }
        }
    }
}
