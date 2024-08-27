use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    /// There was an error running verity
    #[error("Verity error: {0}")]
    Verity(#[from] anyhow::Error),
    /// Error from the underlying reqwest client
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
