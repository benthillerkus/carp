#[derive(Debug, PartialEq)]
pub enum Error {
    MissingDeckName,
    UnexpectedTag {
        expected: &'static str,
        actual: String,
    },
    InvalidAttribueValue {
        tag: String,
        attribute: &'static str,
        value: String,
        allowed: &'static [&'static str],
    },
    Parse(roxmltree::Error),
}

impl std::error::Error for Error {}

impl From<roxmltree::Error> for Error {
    fn from(error: roxmltree::Error) -> Self {
        Self::Parse(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingDeckName => {
                write!(
                    f,
                    r#"the deck has no name attribute set like <deck name="example">...</deck>"#
                )
            }
            Error::UnexpectedTag { expected, actual } => {
                write!(f, "expected {expected}, but got {actual}")
            }
            Error::Parse(e) => e.fmt(f),
            Error::InvalidAttribueValue {
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
