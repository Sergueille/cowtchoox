
Additional things to do (other ars scattered in the source file):

HACK: currently, when failing to parse a tag in maths, it represents it a regular text
    - because if in maths there is for example <=>, it shouldn't be a tag
    - problematic because if user uses large tags, they can't see where is the error

TODO: split text tags with JS

TODO: add the possibility to add attributes to a custom tag

TODO: change the way code is parsed
    -> allow full `<code>` block with attributes
    -> escape `

TODO: integrate highlight.js

TODO: Important: remember to make a CSS file to remove ALL default styling, to prevent differences between browsers engines/versions
      -> should we ship a version of chromium with cowtchoox to make sure the same version is always used?
TODO: some error reporting for css
    - units should be only em, mm, or %
    - class names for custom tags

TODO: report error if custom tag created with invalid names ("div", "span", etc)

TODO: make SVG for arrows, infinity, forall, exists, belongsto, etc...

FIXME: error reporting inside tags noes not take source files spaces into account, resulting in incorrect error positions

