# What is this?

This is a template project for generating decks for the [Tabletop Simulator](https://tabletopsimulator.com) from structured data.

Pretty much all board games with cards (such as *Monopoly*, *Yu-Gi-Oh!* *Take 6!* / *6 Nimmt!* or *Cards Against Humanity*) feature decks of cards that share the same basic layout, but varying data.
Authoring such decks can be rather annoying using traditional graphics programs, even if they allow for templates and overrides like Figma or InDesign, as they'd still require some custom scripts for bulk importing the card data, and then exporting the files.

This repo includes a library for rendering cards ([`carp`](crates/carp)) utilising the [`piet`](https://github.com/linebender/piet) PostScript / Canvas style API.<br>
Additionally there is a sample [App](crates/app) that can load `xml` files describing your decks, render the cards, upload them into an S3 (compatible) bucket and immediately import into the [Tabletop Simulator](https://www.tabletopsimulator.com).

If you're looking for a more fleshed-out tool, check out [eldstal/cardcinogen](https://github.com/eldstal/cardcinogen) or perhaps [decker](https://splizard.com/magic/decker)

# Features

## lib

- traits for decks and cards
- render cards with any aspect ratio into grids of 70 cards
- compress cards, backsides and sheets to PNGs and store on disk or on s3
- modular and multi-threadable design

## app

- custom XML based format for decks of cards
- load cards from files & directories
- per default *Cards Against Humanity* style rendering of cards

Configuration can be done via command line arguments, environment variables and `.env` files.

This is the shape of a `.env` file:
```env
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
S3_BUCKET=
S3_REGION=
S3_ENDPOINT=
S3_PATH_STYLE= // bool, false is subdomain style
RUST_LIB_BACKTRACE=full
RUST_LOG=info
INPUT= // path to folder with xml files
```

(note to self): You can start the app by running `cargo run --release -- -s s3` in the workspace main directory!

# XML

The deck files used by the app are defined as such:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE deck [
  <!ENTITY shy "­">
]> <!-- &shy; marks places where the text layout is allowed to break text -->
<deck name="this is required" back="shared"> <!--shared = each card in the deck has the same back (the opposite would be *unique*)-->
  <card>Text can be <i>italic and in a <font family="Roboto">specific Font</font></i></card>
  <card>You can have text on top<bottom>and on the bottom</bottom></card>
  <card>Lines can be broken like
this
or like<br>this and very long words like Donau&shy;dampf&shy;schiffahrts&shy;kapitän can be broken as such</card>
</deck>
```


As of right now, this format is implemented for the default app and unrelated to the `carp` library. If you want specific attributes or use something like JSON, you'll have to modify the code -- this is a template and not a ready-made product after all.

# Remaining Issues

- add renderers for the other platforms `piet_common` supports
- error handling is still not pretty
- lots of unnecessary copying, both in xml code and pipeline code
- allow disk exporter to clear the directory before exporting
- the s3 exporter fails too silent / it's not clear at all if it a) succeeded uploading and b) if it can actually upload, prior to actually trying
