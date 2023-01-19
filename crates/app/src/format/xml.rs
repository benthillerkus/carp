use super::{
    error::{Error, ErrorKind},
    *,
};
use roxmltree::{Document, Node};
use std::{borrow::Cow, collections::VecDeque};

impl<'a> TryFrom<&'a str> for Deck<'a> {
    type Error = Error;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        Document::parse_with_options(
            input,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                nodes_limit: u32::MAX,
            },
        )?
        .try_into()
    }
}

impl<'a> TryFrom<Document<'a>> for Deck<'a> {
    type Error = Error;

    fn try_from(document: Document<'a>) -> Result<Self, Self::Error> {
        document.root_element().try_into()
    }
}

impl<'a, 'input> TryFrom<Node<'a, 'input>> for Deck<'input> {
    type Error = Error;

    fn try_from(node: Node<'a, 'input>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: node
                .attribute("name")
                .map(|name| Cow::Owned(name.to_owned()))
                .ok_or(Error::from(ErrorKind::MissingDeckName))?,
            theme: node
                .attribute("theme")
                .map_or(Ok(Default::default()), |s| match s {
                    "light" | "Light" | "LIGHT" => Ok(Theme::Light),
                    "dark" | "Dark" | "DARK" => Ok(Theme::Dark),
                    value => Err(Error::from(ErrorKind::InvalidAttribueValue {
                        tag: "deck".into(),
                        attribute: "theme".into(),
                        value: String::from(value),
                        allowed: &["light", "dark"],
                    })),
                })?,
            back: node
                .attribute("back")
                .map_or(Ok(Default::default()), |s| match s {
                    "shared" => Ok(Backside::Shared),
                    "unique" => Ok(Backside::Unique),
                    value => Err(Error::from(ErrorKind::InvalidAttribueValue {
                        tag: "deck".into(),
                        attribute: "back".into(),
                        value: value.into(),
                        allowed: &["shared", "unique"],
                    })),
                })?,
            cards: {
                let mut result = Vec::new();
                let cards = node
                    .children()
                    .filter(|child| child.has_tag_name("card"))
                    .map(Card::try_from);
                for card in cards {
                    match card {
                        Ok(card) => result.push(card),
                        Err(error) => return Err(error),
                    }
                }
                result
            },
        })
    }
}

impl<'a> TryFrom<Node<'_, 'a>> for Card<'a> {
    type Error = Error;

    fn try_from(node: Node<'_, 'a>) -> Result<Self, Self::Error> {
        if !node.has_tag_name("card") {
            Err(Error::from(ErrorKind::UnexpectedTag {
                expected: "card".into(),
                actual: node.tag_name().name().to_owned(),
            }))
        } else {
            Ok(Self {
                content: {
                    let mut content: VecDeque<Markup> = {
                        let mut result = VecDeque::new();

                        for child in node.children() {
                            result.push_back(Markup::try_from(child)?);
                        }
                        result
                    };

                    // Remove leading and trailing blank lines at the start and end of a card
                    if let Some(mut first) = content.pop_front() {
                        match first {
                            Markup::Plain(Cow::Borrowed(ref mut b)) => {
                                *b = b.trim_start();
                                if b.is_empty() {
                                    content.push_front(first)
                                }
                            }
                            Markup::Plain(Cow::Owned(ref mut s)) => {
                                let initial = s.len();
                                if initial > 0 {
                                    s.drain(0..(initial - s.trim_start().len()));
                                    content.push_front(first)
                                }
                            }
                            _ => content.push_front(first),
                        }
                    }
                    if let Some(mut first) = content.pop_back() {
                        match first {
                            Markup::Plain(Cow::Borrowed(ref mut b)) => {
                                *b = b.trim_end();
                                if b.is_empty() {
                                    content.push_back(first)
                                }
                            }
                            Markup::Plain(Cow::Owned(ref mut s)) => {
                                let initial = s.len();
                                if initial > 0 {
                                    for _ in 0..(initial - s.trim_end().len()) {
                                        s.pop();
                                    }
                                    content.push_back(first)
                                }
                            }
                            _ => content.push_back(first),
                        }
                    }

                    content.into()
                },
            })
        }
    }
}

impl<'a> TryFrom<Node<'_, 'a>> for Markup<'a> {
    type Error = Error;

    fn try_from(node: Node<'_, 'a>) -> Result<Self, Error> {
        if node.is_text() {
            return Ok(Self::Plain(Cow::Owned(
                node.text().unwrap_or("").to_owned(),
            )));
        }
        match node.tag_name().name() {
            "blank" => Ok(Self::Blank),
            "br" => Ok(Self::Plain("\n".into())),
            "italic" | "i" => Ok(Self::Italic({
                let mut result = Vec::new();

                for child in node.children() {
                    result.push(Markup::try_from(child)?);
                }
                result
            })),
            "tiny" => Ok(Self::Tiny({
                let mut result = Vec::new();

                for child in node.children() {
                    result.push(Markup::try_from(child)?);
                }
                result
            })),
            "bottom" => Ok(Self::Bottom({
                let mut result = Vec::new();

                for child in node.children() {
                    result.push(Markup::try_from(child)?);
                }
                result
            })),
            "font" => {
                let family = Cow::Owned(
                    node.attribute("family")
                        .ok_or(ErrorKind::MissingFontFamily)?
                        .to_string(),
                );

                let content = {
                    let mut result = Vec::new();

                    for child in node.children() {
                        result.push(Markup::try_from(child)?);
                    }
                    result
                };

                Ok(Self::Font(family, content))
            }
            unknown => Ok(Self::Unknown {
                tag: Cow::Owned(unknown.to_owned()),
                attributes: node
                    .attributes()
                    .map(|attr| {
                        (
                            Cow::Owned(attr.name().to_owned()),
                            Cow::Owned(attr.value().to_owned()),
                        )
                    })
                    .collect(),
                content: {
                    let mut result = Vec::new();

                    for child in node.children() {
                        result.push(Markup::try_from(child)?);
                    }
                    result
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DATA: &str = include_str!("sample.xml");

    #[test]
    fn it_works() {
        let deck: Deck = TEST_DATA.try_into().unwrap();

        assert_eq!(
            deck,
            Deck {
                name: "My Name".into(),
                theme: Theme::Dark,
                back: Default::default(),
                cards: vec![
                    Card {
                        content: vec![
                            Markup::Plain("The first card contains a ".into()),
                            Markup::Blank,
                            Markup::Plain(".".into())
                        ],
                    },
                    Card {
                        content: vec![Markup::Italic(vec![Markup::Blank])]
                    },
                    Card {
                        content: vec![
                            Markup::Plain("This is ".into()),
                            Markup::Italic(vec![Markup::Plain("very".into())]),
                            Markup::Plain(" good.".into()),
                        ],
                    },
                ]
            }
        );
    }

    #[test]
    fn display() {
        let deck: Deck = TEST_DATA.try_into().unwrap();

        assert_eq!(
            format!("{}", deck.cards[0]),
            "The first card contains a ____."
        );
        assert_eq!(format!("{}", deck.cards[1]), "*____*");
        assert_eq!(format!("{}", deck.cards[2]), "This is *very* good.");
    }

    #[test]
    fn trimming() {
        let deck: Deck = r#"<deck name="hi"><card>
        Hallo!!!!!<blank/> </card></deck>"#
            .try_into()
            .unwrap();

        assert_eq!(format!("{}", deck.cards[0]), "Hallo!!!!!____");
    }

    #[test]
    fn bottom() {
        let deck: Deck = r#"<deck name="hi"><card>
        Hallo!!!!!<blank/><bottom>ASDF</bottom>
        </card></deck>"#
            .try_into()
            .unwrap();

        assert_eq!(format!("{}", deck.cards[0]), "Hallo!!!!!____\n\nASDF");
    }
}
