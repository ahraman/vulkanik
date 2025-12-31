use xml::common::{Position, TextPosition};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("duplicate unique tag `<{0}>`")]
    Duplicate(String),

    #[error("unexpected text \"{0}\"")]
    Text(String),
    #[error("unknown tag start `<{0}>")]
    UnknownStart(String),
    #[error("unknown tag end `</{0}>`")]
    UnknownEnd(String),
    #[error("unexpected tag start `<{0}>`, expected `<{1}>`")]
    UnexpectedStart(String, String),
    #[error("unexpected tag end `<{0}>`, expected `<{1}>`")]
    UnexpectedEnd(String, String),
    #[error("unexpected end of file")]
    Eof,

    #[error(transparent)]
    XmlRead(#[from] xml::reader::Error),
}

#[derive(Debug, thiserror::Error)]
pub struct TextError {
    source: Error,
    position: TextPosition,
}

impl std::fmt::Display for TextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.position, self.source)
    }
}

impl From<TextError> for Error {
    fn from(value: TextError) -> Self {
        value.source
    }
}

impl Position for TextError {
    fn position(&self) -> TextPosition {
        self.position
    }
}

impl TextError {
    pub fn new(source: impl Into<Error>, position: impl Position) -> Self {
        Self {
            source: source.into(),
            position: position.position(),
        }
    }
}
