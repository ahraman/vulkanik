#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing required command argument `{0}`")]
    MissingArg(&'static str),
}
