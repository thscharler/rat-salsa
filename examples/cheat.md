# Github Flavored Markup

# General

Up to three leading spaces still work as expected. The fourth
makes everything a code block.

# Thematic break {#fff .as .ddf fjfjj}

At least 3 *, - or _ form a thematic break. Spaces inbetween
are ok.

# Headings I

One to six # make a heading.

# Headings II

Underlining with = makes H1, underlining with - makes a H2.

# Code blocks

* Indent at least by 4.

* Use a fence

```
CODE
```

or

~~~
CODE
~~~

Or inline `code`.

    CODE CODE
    CODE CODE

# HTML blocks

Use tags.

# Links

## Links

[link](/link_to "title")  
[link](</link to> "title")  
<mail:anonymous@example.org>  
<http://other_link.example.org>

## Links to reference

[use_ref][some]  
[some]

## Link references

[some]: /links-somewhere

## Images

Same as links with leading !

![foo](/link_to_image)  
...

## Footnotes

[^1]

[^1]: Footnote

# Tables

| header   | :left      | right: | :center: |
|----------|------------|--------|----------|
| text     | text       | text   | text     |
| **bold** | _emphasis_ |        |          |

# Block quote

> Quote Quote Quote

and also

> [!NOTE]
> [!TIP]  
> [!IMPORTANT]  
> [!WARNING]  
> [!CAUTION]

# List

* Bullet
* Bullet
* Bullet

+ Bullet
+ Bullet
+ Bullet

- Bullet

- [ ] Unchecked

- [x] Checked

1) Numbered
2) Numbered

1. Indent
    1. More Indent
        1. Even more
2. Indent
3. Indent

Definition
: Define things here.  
