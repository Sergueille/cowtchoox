
Additional things to do (other ars scattered in the source file):

FIXME: path of cowx file are currently relative to the working dir or something, but should be relative to the FILE path   

TODO: <%tags> in math

TODO: pagebreak
TODO: headers and footers

HACK: currently, when failing to parse a tag in maths, it represents it a regular text
    - because if in maths there is for example <=>, it shouldn't be a tag
    - problematic because if user uses large tags, they can't see where is the error

TODO: change the way code is parsed
    -> allow full `<code>` block with attributes
    -> escape n x \` with (n+1) * \`

TODO: integrate highlight.js
TODO: the plots

TODO: ship a version of [mini-chromium](https://github.com/chromium/mini_chromium)
    -> will be always same version, and lighter than chrome

TODO: some error reporting for css
    - units should be only em, mm, or %
    - class names for custom tags

TODO: report error if custom tag created with invalid names ("div", "span", etc)

TODO: make SVG for forall, exists, belongsto, etc...

TODO: make nested /* */ comments ?

TODO: prevent circular dependencies of custom tags

TODO: better docs script
