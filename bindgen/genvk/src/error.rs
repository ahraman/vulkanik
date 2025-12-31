#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing required command argument `{0}`")]
    MissingArg(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Parse(#[from] genvk_parse::TextError),
}
