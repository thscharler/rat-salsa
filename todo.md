* extend FocusFlag by `transfer`.
  The FocusFlag can be set to focus=false, transfer=true.
  The next eventhandling run will check for transfer, set the focus
  to the following widget and do __what exactly?__
    * ... probably not useable 