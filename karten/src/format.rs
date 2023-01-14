use std::{borrow::Cow, fmt::Display};

use roxmltree::{Document, Node};
mod error;
use error::Error;

#[derive(Debug, PartialEq)]
pub struct Deck<'a> {
    pub name: Cow<'a, str>,
    pub cards: Vec<Card<'a>>,
    pub theme: Theme,
    pub back: Back,
}

#[derive(Debug, PartialEq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, PartialEq, Default)]
pub enum Back {
    #[default]
    Shared,
    Individual,
}

impl std::error::Error for Error {}

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
                .map(|s| match s {
                    "light" | "Light" | "LIGHT" => Theme::Light,
                    "dark" | "Dark" | "DARK" => Theme::Dark,
                    _ => Default::default(),
                })
                .unwrap_or_default(),
            back: node
                .attribute("theme")
                .map(|s| match s {
                    "shared" | "Shared" | "SHARED" => Back::Shared,
                    "individual" | "Individual" | "INDIVIDUAL" => Back::Individual,
                    _ => Default::default(),
                })
                .unwrap_or_default(),
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

#[derive(Debug, PartialEq)]
pub struct Card<'a> {
    pub content: Vec<Markup<'a>>,
}

impl Display for Card<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for markup in &self.content {
            write!(f, "{}", markup)?;
        }
        Ok(())
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
                content: node.children().map(Markup::from).collect(),
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Markup<'a> {
    Plain(Cow<'a, str>),
    Blank,
    Italic(Vec<Markup<'a>>),
    Unknown {
        tag: Cow<'a, str>,
        content: Vec<Markup<'a>>,
    },
}

impl Display for Markup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Markup::Plain(text) => write!(f, "{}", text),
            Markup::Blank => write!(f, "____"),
            Markup::Italic(content) => {
                write!(f, "*")?;
                for markup in content {
                    write!(f, "{}", markup)?;
                }
                write!(f, "*")
            }
            Markup::Unknown { tag, content } => {
                write!(f, "<{}>", tag)?;
                for markup in content {
                    write!(f, "{}", markup)?;
                }
                write!(f, "</{}>", tag)
            }
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

    const TEST_DATA: &str = include_str!("format/sample.xml");

    #[test]
    fn it_works() {
        let deck: Deck = TEST_DATA.try_into().unwrap();

        assert_eq!(
            deck,
            Deck {
                name: "My Name".into(),
                theme: Default::default(),
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
}
