# Pal-Edit

Editor for rat-theme4 palettes. 
With preview for most rat-widget's.

## Parameters

pal-edit [pal-file] [--alias aliases.ini]

## UI

* Theme Name: Name for the created SalsaTheme.
* Theme: The type of theme that will be created.
  Choose one or use your custom theme. 
* Name: Name for the Palette. Just informational. 
  You can use the same color-palette to create a light
  and a dark theme. Giving both the same palette name
  is a hint that those should be the same.
* Dark: Palette uses a [Color;8] to store the colors.
  pal-edit uses 0..=3 for the base colors and 4..=7 
  for the dark variants. This gives the scaling factor
  to calculate the dark variant. 

### Color palette

Color input
* Can show colors in RGB, hex and HSV. 
* Use 'm' and 'M' to switch.
* Tab jumps between the components.
* '+', '-', ALT-'+', ALT-'-' changes the component at
  the cursor.
* Mouse-wheel changes the component pointed at.
* ALT+Mouse-wheel uses a larger step size. 
* Ctrl-C, Ctrl-X, Ctrl-V work.

### Color aliases

Logical color names refer to some color in the palette.
There is one extra 'None-0' that maps to Color::Reset.

### Extra aliases

You can configure aliases in the config or set them at
startup with --alias. This is just a list of alias names.

### Preview

Gives a life preview for most rat-widgets.

### Foreign

You can import color palettes from an external file. 
Anything that vaguely resembles the 'name=color' pattern
will be loaded here. There is a special path for neo-vims
base46 lua files. 

You can 'Extern/Use Base46 colors' to copy a known subset
to the palette.

### Menu

* Palette
  * New ...
  * Load: Load one or more .pal files. .json are also fine.
  * Save ...
  * Save as ...
  * Export .rs: Export the palette as rust source code.
  
* Patch 
  It may be useful to patch an existing palette with
  additional color-aliases instead of copying it wholesale.
  
  * Auto-Load from: Choose a directory where you store
    the .ppal files. When you load a palette it will 
    load the patch too.
  * Export .rs: Export only the patch as rust source code.
  
* Extern
  Import palettes from other sources.
  
  * Import colors: Import some file. Anything that
    resembles 'name=color' will be loaded.
  * Use Base 46 colors: Use known base46 names to map
    colors.
    
* List
  * Next/Prev: Change palette.    
  
    
    
    
  
















  
  
  





