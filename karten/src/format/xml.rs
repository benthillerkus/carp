use super::{error::Error, *};
use roxmltree::{Document, Node};
use std::{borrow::Cow, collections::VecDeque};

impl<'a> TryFrom<&'a str> for Deck<'a> {
    type Error = Error;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        Document::parse(input)?.try_into()
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
                .ok_or(Error::MissingDeckName)?,
            theme: node
                .attribute("theme")
                .map_or(Ok(Default::default()), |s| match s {
                    "light" | "Light" | "LIGHT" => Ok(Theme::Light),
                    "dark" | "Dark" | "DARK" => Ok(Theme::Dark),
                    value => Err(Error::InvalidAttribueValue {
                        tag: String::from("deck"),
                        attribute: "theme",
                        value: String::from(value),
                        allowed: &["light", "dark"],
                    }),
                })?,
            back: node
                .attribute("back")
                .map_or(Ok(Default::default()), |s| match s {
                    "shared" | "Shared" | "SHARED" => Ok(Back::Shared),
                    "individual" | "Individual" | "INDIVIDUAL" => Ok(Back::Individual),
                    value => Err(Error::InvalidAttribueValue {
                        tag: String::from("deck"),
                        attribute: "back",
                        value: String::from(value),
                        allowed: &["shared", "individual"],
                    }),
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
            Err(Error::UnexpectedTag {
                expected: "card",
                actual: node.tag_name().name().to_owned(),
            })
        } else {
            Ok(Self {
                content: {
                    let mut content: VecDeque<Markup> = node.children().map(Markup::from).collect();

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

impl<'a> From<Node<'_, 'a>> for Markup<'a> {
    fn from(node: Node<'_, 'a>) -> Self {
        if node.is_text() {
            return Self::Plain(Cow::Owned(node.text().unwrap_or("").to_owned()));
        }
        match node.tag_name().name() {
            "blank" => Self::Blank,
            "italic" | "i" => Self::Italic(node.children().map(Markup::from).collect()),
            unknown => Self::Unknown {
                tag: Cow::Owned(unknown.to_owned()),
                content: node.children().map(Markup::from).collect(),
            },
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
}