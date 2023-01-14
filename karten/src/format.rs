use std::{borrow::Cow, fmt::Display, ops::Range};

mod error;
mod xml;

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

pub type Content<'a> = Vec<Markup<'a>>;

#[derive(Debug, PartialEq)]
pub struct Card<'a> {
    pub content: Content<'a>,
}

impl Display for Card<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for markup in &self.content {
            write!(f, "{}", markup)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Markup<'a> {
    Plain(Cow<'a, str>),
    Blank,
    Italic(Content<'a>),
    Unknown {
        tag: Cow<'a, str>,
        content: Content<'a>,
    },
}

impl Display for Markup<'_> {
    /// This will render the card with human readable markup.
    /// This is not meant to be parsed back into a card.
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

impl Card<'_> {
    /// Returns the content of the card as a string of plain text and a list of style annotations.
    /// This is useful for rendering the card with a GUI toolkit.
    pub fn styled_segments(&self) -> (String, Vec<StyleAnnotation>) {
        let mut render = String::new();
        let mut annotations = Vec::new();

        fn styled_segments(
            content: &[Markup<'_>],
            render: &mut String,
            annotations: &mut Vec<StyleAnnotation>,
        ) {
            content.iter().for_each(|markup| match markup {
                Markup::Plain(text) => render.push_str(text),
                Markup::Blank => {
                    render.push_str("____");
                }
                Markup::Italic(content) => {
                    let start = render.len();
                    styled_segments(content, render, annotations);
                    let end = render.len();
                    annotations.push(StyleAnnotation::Italic(start..end));
                }
                Markup::Unknown { tag: _, content } => {
                    styled_segments(content, render, annotations);
                }
            });
        }

        styled_segments(&self.content, &mut render, &mut annotations);

        (render, annotations)
    }
}

#[derive(Debug, PartialEq)]
pub enum StyleAnnotation {
    Italic(Range<usize>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn styled_segments() {
        let card = Card {
            content: vec![
                Markup::Plain("Hello".into()),
                Markup::Blank,
                Markup::Italic(vec![Markup::Plain("World".into())]),
            ],
        };

        let (render, annotations) = card.styled_segments();

        assert_eq!(render, "Hello____World");
        assert_eq!(&"Hello____World"[9..14], "World");
        assert_eq!(annotations, vec![StyleAnnotation::Italic(9..14)]);
    }
}
