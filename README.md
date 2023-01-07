# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

If you're looking for a more fleshed out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Todo

- define export names in Deck
  - fix output cli
- use builder pattern for Deck
- render % as ______
 - render italics
- load quips
- split into two crates
 - remove Importer
- generate lua
- use trait for exporter aswell?
- clean up Deck
  - remove dimensions from Deck
   - maybe move into a seperate rendering related module?
- remove create_sheet and create_card from dimensions