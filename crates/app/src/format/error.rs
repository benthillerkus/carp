use color_eyre::{eyre::eyre, Report};

#[derive(Debug, PartialEq)]
pub enum Kind {
    MissingDeckName,
    UnexpectedTag {
        expected: String,
        actual: String,
    },
    InvalidAttribueValue {
        tag: String,
        attribute: String,
        value: String,
        allowed: &'static [&'static str],
    },
    Parse,
}

#[derive(Debug)]
pub struct Error {
    pub kind: Kind,
    source: Report,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}

impl From<roxmltree::Error> for Error {
    fn from(error: roxmltree::Error) -> Self {
        use roxmltree::Error as XmlError;

        let kind = match error.clone() {
            XmlError::UnexpectedCloseTag {
                expected, actual, ..
            } => Kind::UnexpectedTag { expected, actual },
            _ => Kind::Parse,
        };

        Self {
            kind,
            source: error.into(),
        }
    }
}

impl From<Kind> for Error {
    fn from(kind: Kind) -> Self {
        Self {
            source: eyre!(kind.to_string()),
            kind,
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDeckName => {
                write!(
                    f,
                    r#"the deck has no name attribute set like <deck name="example">...</deck>"#
                )
            }
            Self::UnexpectedTag { expected, actual } => {
                write!(f, "expected {expected}, but got {actual}")
            }
            Self::Parse => write!(f, "couldn't parse the xml file"),
            Self::InvalidAttribueValue {
                tag,
                attribute,
                value,
                allowed,
            } => write!(
                f,
                r#"invalid value for attribute: <{tag} ... {attribute}="{value}" ... />, allowed values are {allowed:?}"#
            ),
        }
    }
}
