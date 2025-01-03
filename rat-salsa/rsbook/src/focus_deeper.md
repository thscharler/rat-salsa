
# Details, details

## Focus

### Navigation

* first(): Focus the first widget.
* next()/prev(): Change the focus. 
* focus(): Focus a specific widget. 
* focus_at(): Focus the widget at a position.

### Debugging

* You can construct the FocusFlag with a name. 
* Call Focus::enable_log()
* You might find something useful in your log-file.

### Dynamic changes

You might come to a situation where

* Your state changed
  * which changes the widget structure/focus order/...
    * everything should still work
    
then you can use one of

* remove_container
* update_container
* replace_container

to change Focus without completely rebuilding it. 

They reset the focus state for all widgets that are no longer
part of Focus, so there is no confusion who currently owns the
focus. You can call some focus function to set the new focus
afterwards.





    
