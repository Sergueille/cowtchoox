
Additional things to do (other ars scattered in the source file):

TODO: Create a module for emitting log, warnings, and errors. Maybe print them all in a nice and sorted way at the end (it's becoming urgent!)
TODO: parse math (maybe at the same time as the base parser, to easily support tags in maths)

TODO: Important: remember to make a CSS file to remove ALL default styling, to prevent differences between browsers engines/versions
      -> should we ship a version of chromium with cowtchoox to make sure the same version is always used?

BUG: Looks like that if two tags ar directly followed (`<p></p><p></p>`), it fails to parse
BUG: Line numbers are doubled on widows...

- The font doesn't look right. It's not exactly the sam as tex...

