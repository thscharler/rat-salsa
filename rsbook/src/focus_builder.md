
# Focus builder

The functions widget() and container() add widgets to Focus. 
They will be traversed in the order given. 

The two other important functions are 

* build_focus()
  
  Takes a container widget and returns a Focus.
  
* rebuild_focus()

  Does the same, but takes the previous Focus too. 
  
  What it does is, it builds the new Focus and checks which
  widgets are __no longer__ part of it. It resets all 
  FocusFlags for those widgets. 
  
  A bonus is it reuses the allocations too.
  
  
  
