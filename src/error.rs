use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("bcalm2 io error: {0}")]
    BCalm2IoError(#[from] crate::io::bcalm2::error::BCalm2IoError),

    #[error("fasta io error: {0}")]
    FastaIoError(#[from] crate::io::fasta::error::FastaIoError),

    #[error("wtdbg2 io error: {0}")]
    Wtdbg2IoError(#[from] crate::io::wtdbg2::error::Wtdbg2IoError),

    #[error("dot io error: {0}")]
    DotIoError(#[from] crate::io::wtdbg2::dot::error::DotIoError),

    #[error("gfa io error: {0}")]
    GfaIoError(#[from] crate::io::gfa::error::GfaIoError),
}
