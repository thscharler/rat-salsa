# 3.1.0

* Palette: add color2u32()
* add EverForest and Nord palettes.
* work on the other palettes too.

# 3.0.1

* add ColorInput widget.

# 3.0.0

Start rat-theme4. Simplify everything.
What is left is a map style-name -> style-constructor.

The style-constructor gets a reference to the theme and
can use other definitions to create a Style for a widget.

There is a bit of special handling for ratatui 'Style' structs.

And a few convenience functions for different ways to get
to a style. This is good enough to load/store styles to storage,
but no implementation for this.

# 2.3.0

* add 'Rust' palette. blue is a cobalt oxide, green is a chromium oxide,
  and purple is a purple ochre an iron oxide. matches up nicely:)
  And of course red orange and yellow are different iron-oxides.
  ![](svg\rust.jpg)

# 2.2.0

* upgrade rat-text to 2.0.0
* break: remove PagerStyle

# 2.1.1

* fix: wrong name

# 2.1.0

* break: add text-colors to the palette. The text-color functions use
  these instead of black/white.

* feature: Palette::normal_contrast_color() and Palette::high_contrast_color()
  given a background and a list of possible colors, this chooses the color
  with the second best/best contrast.

* add 'Rust' palette.

# 2.0.0

* Add 'Shell' theme.

--- from rat-theme2 (original) ---

# 1.2.0

* remove: CaptionStyle

# 1.1.0

* feature: add CaptionStyle
* fix: reorder by niceness

# 1.0.0

* stabilize api

# 0.29.1

* refactor: rename Scheme to Palette.

# 0.28.1

* feature: add Black&White color scheme and make it
  actually work

# 0.28.0

* feature: create a better designed color Scheme.

--- from rat-theme (original) ---

# 0.27.11

* bump version.

# 0.27.10

* fix: map true_dark colors to 0..63

# 0.27.9

* feature: more text-colors

# 0.27.8

* bump version of rat-widget

# 0.27.7

* fix: add some border-styles.
* fix: split style

# 0.27.6

* fix: rat-widget version

# 0.27.5

* update rat-focus

# 0.27.4

* moved all rat-crates to one repo

# 0.27.3

* refactor: DarkTheme has become an incoherent mess. Do some cleanup.
* feature: add Scheme::true_dark_color() which reduces any given color
  to something that can pass as a dark background. This works for
  base-colors and indexed colors too. It took up the pain and copied
  the RGB values from wikipedia :)
* feature: add Schema::grey_color() which does a decent graying of
  any color.
* feature: a few styles gained a border_style. add here where applicable.

# 0.27.2

* add dialog_arrow() style for Scrollbars in popups/dialogs.
* add dialog_scroll_style() for Scrollbars in popups/dialogs.
* sync choice/radio/checkbox styles with text-input styles.
* make choice-popups look nicer with a new border
* add label-style to PagerStyle and ClipperStyle.
* add style() function that creates a style from a background color.
* remove divider from PagerStyle
* rename nav to navigation in PagerStyle

# 0.27.0

** upgrade to ratatui 0.29 **

* break: tried to implement some naming scheme upon DarkTheme.
* feat: add styles for new widgets in rat-widget

# 0.26.2

* fix: add missing styles.

# 0.26.1

* fix: name changes in styles.

# 0.26.0

* break: final renames in rat-focus.

# 0.25.1

* update dependencies

# 0.25.0

Sync version for beta.

* Add Base16 and Base16Relaxed

# 0.12.2

* docs fix

# 0.12.1

* fix: changes in rat-scroll
* fix: styles for buttons and file-dialog

# 0.12.0

* update ratatui 0.28

# 0.11.0

* break: added and renamed various styles.

* add: ocean, vscode_dark

# 0.10.2

* Add: Oxocarbon
* add color_schemes(), dark_themes() to get all current themes

# 0.10.1

* Add color-schemes: Tundra, Monekai, Monochrome
* Fix: Minor bugs

# 0.10.0

Initial release. 
