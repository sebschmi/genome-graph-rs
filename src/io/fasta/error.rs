use thiserror::Error;

#[derive(Debug, Error)]
pub enum FastaIoError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("error encountered while trying to format a structure as string: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("walk is empty")]
    EmptyWalkError,

    #[error("an edge has no mirror")]
    EdgeWithoutMirror,
}
