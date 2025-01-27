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
