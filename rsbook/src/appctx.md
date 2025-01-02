
# AppContext

The AppContext gives access to application wide services.

There are some builtins:

* add_timer(): Define a timer that will send TimeOut events.

* spawn(): Spawn long running tasks in the thread-pool.
  You can work with some shared memory model to get the
  results, but the preferred method is to return a
  Control::Message from the thread.  

* spawn_async(): Spawn async tasks in the tokio runtime. 
  The result of the async task can be returned as a
  Control::Message too. 
  
* spawn_async_ext(): Gives you an extra channel to return
  multiple results from the async task.
  
* queue(): Add results to the event-handling queue if 
  a single return value from event-handling is not enough.
  
* focus(): An instance of Focus can be stored here. 
  Setting up the focus is the job of the application.
  
* count: Gives you the frame-counter of the last render.

All application wide stuff goes into `g` which is an
instance of your Global state.   
    
  
# RenderContext

Rendercontext is limited compared to AppContext. 

It gives you `g` as your Global state.

And it lets you set the screen-cursor position. 
  

 
