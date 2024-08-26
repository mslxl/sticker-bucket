use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("error when init config framework: {0}")]
    ConfigCorrupt(String),
    #[error("the library has been locked, does other instance running?")]
    LibraryLocked,
    #[error("unknown error")]
    Unknown,
}
