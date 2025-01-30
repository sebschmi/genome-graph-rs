use thiserror::Error;

#[derive(Debug, Error)]
pub enum GfaIoError {
    #[error("an L-line was encountered, but the overlap pattern is unknown: '{pattern}'")]
    UnknownOverlapPattern { pattern: String },

    #[error("an L-line was encountered, but the overlap pattern is missing")]
    MissingOverlapPattern,

    #[error("an L-line was encountered, at least one of the nodes is missing")]
    MissingNode,
}
