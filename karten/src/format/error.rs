#[derive(Debug, PartialEq)]
pub enum Error {
    MissingDeckName,
    UnexpectedTag {
        expected: &'static str,
        actual: String,
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
        }
    }
}