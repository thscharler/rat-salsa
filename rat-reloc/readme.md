![semver](https://img.shields.io/badge/semver-☑-FFD700)
![stable](https://img.shields.io/badge/stability-stable-8A2BE2)
[![crates.io](https://img.shields.io/crates/v/rat-reloc.svg)](https://crates.io/crates/rat-reloc)
[![Documentation](https://docs.rs/rat-reloc/badge.svg)](https://docs.rs/rat-reloc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-salsa)

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-salsa/blob/master/rat-reloc/changes.md)

# Rat-Reloc(ate)

This crate defines the trait
[RelocatableState](https://docs.rs/rat-reloc/latest/rat_reloc/trait.RelocatableState.html)

# Why?

Many widgets in rat-widget store one or more areas for mouse interaction.

And there are widgets that render other widgets to a temp Buffer and later
dump parts of it to the main render Buffer. And then all the stored areas
in the widget-state are wrong.

The RelocatableState trait gives the widgets that use such temp Buffers
a hook to correct for any movement and clipping that has happened.

# Why so complicated?

* This doesn't affect normal rendering of a widget, it's just
  and afterthought.
* The widget doesn't need to know what other widgets exist,
  it just has to provide the function to relocate its areas
  after rendering.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

