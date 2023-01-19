use std::{borrow::Cow, fmt::Display, ops::Range};

use carp::Backside;

mod error;
mod xml;

#[derive(Debug, PartialEq)]
pub struct Deck<'a> {
    pub name: Cow<'a, str>,
    pub cards: Vec<Card<'a>>,
    pub theme: Theme,
    pub back: Backside,
}

#[derive(Debug, PartialEq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
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
    Font(Cow<'a, str>, Content<'a>),
    Tiny(Content<'a>),
    Bottom(Content<'a>),
    Unknown {
        tag: Cow<'a, str>,
        attributes: Vec<(Cow<'a, str>, Cow<'a, str>)>,
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
            Markup::Italic(content) | Markup::Tiny(content) => {
                write!(f, "*")?;
                for markup in content {
                    write!(f, "{}", markup)?;
                }
                write!(f, "*")
            }
            Markup::Bottom(content) => {
                writeln!(f)?;
                writeln!(f)?;
                for markup in content {
                    write!(f, "{}", markup)?;
                }
                Ok(())
            }
            Markup::Unknown {
                tag,
                content,
                attributes,
            } => {
                write!(f, "<{tag}")?;
                for (key, value) in attributes {
                    write!(f, r#" {key}="{value}""#)?;
                }
                for markup in content {
                    write!(f, "{}", markup)?;
                }
                write!(f, "</{}>", tag)
            }
            Markup::Font(_, content) => {
                for markup in content {
                    write!(f, "{}", markup)?;
                }
                Ok(())
            }
        }
    }
}

impl Card<'_> {
    /// Returns the content of the card as a string of plain text and a list of style annotations.
    /// This is useful for rendering the card with a GUI toolkit.
    pub fn annotated_top(&self) -> (String, Vec<StyleAnnotation>) {
        let mut render = String::new();
        let mut annotations = Vec::new();

        Card::styled_segments(&self.content, &mut render, &mut annotations);

        (render, annotations)
    }

    pub fn annotated_bottom(&self) -> Option<(String, Vec<StyleAnnotation>)> {
        let mut render = String::new();
        let mut annotations = Vec::new();

        if let Some(Markup::Bottom(content)) = self.content.last() {
            Card::styled_segments(content, &mut render, &mut annotations);
        }

        Some((render, annotations))
    }

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
            Markup::Bottom(_) => {}
            Markup::Italic(content) => {
                let start = render.len();
                Card::styled_segments(content, render, annotations);
                let end = render.len();
                annotations.push((start..end).annotate_with(Style::Italic));
            }
            Markup::Font(family, content) => {
                let start = render.len();
                Card::styled_segments(content, render, annotations);
                let end = render.len();
                annotations.push((start..end).annotate_with(Style::Font(family.to_string())));
            }
            Markup::Tiny(content) => {
                let start = render.len();
                Card::styled_segments(content, render, annotations);
                let end = render.len();
                annotations.push((start..end).annotate_with(Style::Size(0.5)));
            }
            Markup::Unknown { content, .. } => {
                Card::styled_segments(content, render, annotations);
            }
        });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Style {
    Italic,
    Font(String),
    Size(f64),
}

trait AnnotateWithStyle {
    fn annotate_with(self, style: Style) -> StyleAnnotation;
}

impl AnnotateWithStyle for Range<usize> {
    fn annotate_with(self, style: Style) -> StyleAnnotation {
        StyleAnnotation { range: self, style }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StyleAnnotation {
    pub range: Range<usize>,
    pub style: Style,
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

        let (render, annotations) = card.annotated_top();

        assert_eq!(render, "Hello____World");
        assert_eq!(&"Hello____World"[9..14], "World");
        assert_eq!(annotations, vec![(9..14).annotate_with(Style::Italic)]);
    }

    #[test]
    fn styled_segments2() {
        let card = Card {
            content: vec![
                Markup::Plain("Hello".into()),
                Markup::Bottom(vec![Markup::Tiny(vec![Markup::Plain("There".into())])]),
            ],
        };

        let (render, _) = card.annotated_top();
        assert_eq!(render, "Hello");

        let (render, annotations) = card.annotated_bottom().unwrap();
        assert_eq!(render, "There");
        assert_eq!(annotations, vec![(0..5).annotate_with(Style::Size(0.5))]);
    }
}
