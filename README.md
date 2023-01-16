# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

If you're looking for a more fleshed-out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Todo

- add renderers for the other platforms `piet_common` supports
- fix error handling
  - use `color_eyre` for `Box<dyn Error>`
  - get rid of `unwrap`s
- move file creation out of exporter
- overwrite directory
