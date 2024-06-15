# Theming support for rat-salsa.

Splits the theme in two parts:

* Scheme - The underlying color-scheme with enough colors to play
  around.
* One concrete DarkTheme that takes that scheme and produces Styles
  for widgets. This intentionally doesn't adhere to any trait, just
  provides some baselines for each widget type.
  You can use this as is, or copy it and adapt it for your applications
  needs.

        In the end I think this will be just some building-blocks for 
        an application defined theme. I think most applications will need
        more semantics than just 'some table', 'some list'. 