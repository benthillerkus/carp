# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

If you're looking for a more fleshed out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Todo

- define export names in Deck
  - fix output cli
- use builder pattern for Deck
- render `%` as _____
 - render *italics*
- load quips
- split into two crates
 - remove Importer
- generate lua
- use trait for exporter aswell?
- add renderers for the other platforms `piet_common` supports
- fix error handling
 - use `color_eyre` for `Box<dyn Error>`
 - get rid of `unwrap`s