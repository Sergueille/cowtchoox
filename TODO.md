
Additional things to do (other ars scattered in the source file):

TODO: custom tags
    -> definition in cowx `<!tag_name>contents, and <inner/> to say where thing in the tag should go</tag_name>`
    -> usage `<!tag_name/>` or `<!tag_name>things inside</tag_name>`

HACK: currently, when failing to parse a tag in maths, it represents it a regular text
    - because if in maths there is for example <=>, it shouldn't be a tag
    - problematic because if user uses large tags, they can't see where is the error

TODO: change the way code is parsed
    -> allow full `<code>` block with attributes
    -> escape `

TODO: integrate highlight.js

TODO: ship a version [mini-chromium](https://github.com/chromium/mini_chromium)
    -> will be always same version, and lighter than chrome

TODO: some error reporting for css
    - units should be only em, mm, or %
    - class names for custom tags

TODO: report error if custom tag created with invalid names ("div", "span", etc)

TODO: make SVG for arrows, infinity, forall, exists, belongsto, etc...

FIXME: error reporting inside tags noes not take source files spaces into account, resulting in incorrect error positions

