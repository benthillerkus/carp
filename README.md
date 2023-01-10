# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

If you're looking for a more fleshed-out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Todo

- use builder pattern for Deck
  - use typestate pattern to differentiate between single back or back sheet (instead of dynamic dispatch)
- render `%` as _____
  - render *italics*
  - render ­ correctly [soft hyphen](https://unicode-explorer.com/c/00AD)
- split into two crates
- generate lua
- add renderers for the other platforms `piet_common` supports
- fix error handling
  - use `color_eyre` for `Box<dyn Error>`
  - get rid of `unwrap`s
- index in Card trait? remove, replace with index from enumeration
- move file name generation from metadata into Artifact
- move file creation out of exporter
- overwrite directory
- remove tabletop simulator assumptions from render sheets
  - maybe make dimensions more powerful?
