# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

Pretty much all board games with cards (such as *Monopoly*, *Yu-Gi-Oh!* *Take 6!* / *6 Nimmt!* or *Cards Against Humanity*) feature decks of cards that share the same basic layout, but varying data.
Authoring such decks can be rather annoying using traditional graphics programs, even if they allow for templates and overrides like Figma or InDesign, as they'd still require some custom scripts for bulk importing the card data, and then exporting the files.

This repo includes a library for rendering cards ([`carp`](crates/carp)) utilising the [`piet`](https://github.com/linebender/piet) PostScript / Canvas style API.<br>
Additionally there is a sample [App](crates/app) that can load `xml` files describing your decks, render the cards, upload them into an S3 (compatible) bucket and immediately import into the [Tabletop Simulator](https://www.tabletopsimulator.com).

If you're looking for a more fleshed-out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Remaining Issues

- add renderers for the other platforms `piet_common` supports
- error handling is still not pretty
- lots of unnecessary copying, both in xml code and pipeline code
- allow disk exporter to clear the directory before exporting
- the s3 exporter fails too silent / it's not clear at all if it a) succeeded uploading and b) if it can actually upload, prior to actually trying
