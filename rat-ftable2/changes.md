# 0.2

Replaced the trait with separated closures for row-height, 
row-render and cell-render. 

Removed more dubiously useless functions.
Added proper support for varying row-heights.

# 0.1

Initial version. Reused the rendering logic from rat-ftable but
removed a bit of cruft.

Table-rendering is now based on a plain-old Iterator for the data.
The actual rendering is still handed out to some implementation,
but the previous TableData/TableDataIter traits where fused into
RenderTableRow and stripped of everything but rendering of one
row.

__Renames__

|---------------|----------------|
| TableContext | RenderContext |
| TableData | RenderTableRow |
| TableDataIter | RenderTableRow |

__Removed__

EditableTable has been removed. It has been way too complicated
to use.

## Table

* Removed support for ratatui-style Tables.
* Now uses a standard Iterator for the data.
* Header/Footer now uses Text instead of Row+Cell.
* Layout-width changed to enum to cover all cases.
    * replaces separate layout_width(), layout_column_widths() and auto_layout_width()

## TableVec

* New widget that holds the table-data as part of its state.

## TableEditVec

* Replaces EditableTableVec.
* Holds the table-data in a Vec as part of its state. 

