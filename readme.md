[![crates.io](https://img.shields.io/crates/v/rat-reloc.svg)](https://crates.io/crates/rat-reloc)
[![Documentation](https://docs.rs/rat-reloc/badge.svg)](https://docs.rs/rat-reloc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-reloc)

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-reloc/blob/master/changes.md)

# Rat-Reloc(ate)

RelocatableState enables rendering StatefulWidget's to a temporary buffer.

After rendering a stateful widget all areas derived from the
render area will be wrong. This trait defines a
[relocate](RelocatableState::relocate) function that corrects
the areas afterwards.

* Doesn't impact normal rendering of the widget.
  It can just use the area and be done with it.

* Straightforward

    ```rust
      use rat_reloc::{RelocatableState, relocate_area};
      use ratatui::layout::Rect;;

      # struct ButtonState{ area:Rect, inner:Rect}

      impl RelocatableState for ButtonState {
          fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
              self.area = relocate_area(self.area, shift, clip);
              self.inner = relocate_area(self.inner, shift, clip);
          }
      }
    ```
* Decent to implement for a view widget

    ```rust
      use ratatui::layout::Rect;
      use ratatui::buffer::Buffer;
      use ratatui::widgets::StatefulWidget;
      use rat_reloc::RelocatableState;

      pub struct RelocatedRender;

      impl RelocatedRender {
          fn render<W, S>(widget: W, area: Rect, state: &mut S)
          where
              W: StatefulWidget<State = S>,
              S: RelocatableState,
          {
              // remap area
              let area = Rect::default();
              // use inner buffer
              let mut buf = Buffer::default();

              // render to buffer
              widget.render(area, &mut buf, state);

              // calculate shift and clip
              let shift = (-1, -1);
              let clip = Rect::default();

              // correct state
              state.relocate(shift, clip);
          }
      }
    ```

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

