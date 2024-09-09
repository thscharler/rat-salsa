[![crates.io](https://img.shields.io/crates/v/rat-ftable.svg)](https://crates.io/crates/rat-ftable)
[![Documentation](https://docs.rs/rat-ftable/badge.svg)](https://docs.rs/rat-ftable)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-ftable)

## Table widget based on ratatui.

Could be used as a drop-in replacement for the ratatui table. But
that's not the point of this widget.

This widget uses the [TableData](crate::TableData) trait instead
of rendering all the table-cells and putting them into a Vec.
This way rendering time only depends on the screen-size not on
the size of your data.

There is a variant that takes an Iterator of
[TableRowData](crate::TableData). It has as few traps though.
If the Iterator doesn't have an efficient skip() or if you can'
t give the number of rows this will iterate all your data for the
necessary information. This might slow down everything a bit.

![image](https://github.com/thscharler/rat-ftable/blob/master/ftable.gif?raw=true)

More bullet points:

* Row and Column offset for rendering.
* Pluggable selection with [TableSelection](crate::TableSelection)
    * Allows row/column/cell selection.
    * Row/column/cell selection + Header/Footer selection each
      with its own style.
* Basic key/mouse handling present.

Eventhandling is currently crossterm only. In practice
event-handling is calling 1 or 2 functions on the state, so this
should be easy to map to other systems. (Contributions welcome :)
