# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

If you're looking for a more fleshed-out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Remaining Issues

- add renderers for the other platforms `piet_common` supports
- error handling is still not pretty
- lots of unnecessary copying, both in xml code and pipeline code
- allow disk exporter to clear the directory before exporting
- the s3 exporter fails too silent / it's not clear at all if it a) succeeded uploading and b) if it can actually upload, prior to actually trying
