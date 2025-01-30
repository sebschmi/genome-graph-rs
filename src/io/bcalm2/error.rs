use thiserror::Error;

#[derive(Debug, Error)]
pub enum BCalm2IoError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("error encountered while trying to format a structure as string: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("invalid node id: '{id:?}'")]
    BCalm2IdError { id: String },

    #[error("the length in the description of a node ({length}) does not match the length of its sequence {sequence_length}")]
    BCalm2LengthError {
        length: usize,
        sequence_length: usize,
    },

    #[error("unknown parameter: '{parameter:?}'")]
    BCalm2UnknownParameterError { parameter: String },

    #[error("duplicate parameter: '{parameter:?}'")]
    BCalm2DuplicateParameterError { parameter: String },

    #[error("malformed parameter: '{parameter:?}'")]
    BCalm2MalformedParameterError { parameter: String },

    #[error("missing parameter: '{parameter:?}'")]
    BCalm2MissingParameterError { parameter: String },

    #[error("node id is out of range (usize) for displaying")]
    BCalm2NodeIdOutOfPrintingRange,

    #[error("node has no mirror")]
    BCalm2NodeWithoutMirror,

    #[error("edge has no mirror")]
    BCalm2EdgeWithoutMirror,
}
