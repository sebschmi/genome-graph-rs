use thiserror::Error;

#[derive(Debug, Error)]
pub enum Wtdbg2IoError {
    #[error("unknown node direction: {direction}")]
    UnknownNodeDirection { direction: String },

    #[error("unknown line start: {line_start}")]
    UnknownLineStart { line_start: String },
}
