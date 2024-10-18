[![crates.io](https://img.shields.io/crates/v/rat-ftable.svg)](https://crates.io/crates/rat-ftable)
[![Documentation](https://docs.rs/rat-ftable/badge.svg)](https://docs.rs/rat-ftable)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-ftable)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-ftable GitHub][refGitHubFTable].

* [Changes](https://github.com/thscharler/rat-ftable/blob/master/changes.md)

# Table widget for ratatui

Can be used as a drop-in replacement for the ratatui table. But
that's not the point of this widget.

This widget uses the [TableData](crate::TableData) trait instead
of rendering all the table-cells and putting them into a Vec.
This way rendering time only depends on the screen-size not on
the size of your data.

There is a second trait [TableDataIter](crate::TableDataIter) that
works better if you only have an Iterator over your data.

> Caveat: If the Iterator doesn't have an efficient skip() or if you
> can't give the number of rows this will iterate all your data
> for the necessary information. This might slow down everything
> a bit.

![image](https://github.com/thscharler/rat-ftable/blob/master/ftable.gif?raw=true)

More bullet points:

* Row and Column scrolling.
* Pluggable selection with [TableSelection](crate::TableSelection)
    * Allows row/column/cell selection.
    * Row/column/cell selection + Header/Footer selection each
      with its own style.
* Key/mouse handling present.

Eventhandling is currently crossterm only.

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refGitHubFTable]:  https://github.com/thscharler/rat-ftable/tree/master/examples
