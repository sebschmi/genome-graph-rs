use thiserror::Error;

#[derive(Debug, Error)]
pub enum DotIoError {
    #[error("unexpected token '{actual}', expected '{expected}'")]
    UnexpectedToken { expected: String, actual: String },

    #[error("expected empty line, but got: '{actual}'")]
    MissingEmptyLine { actual: String },

    #[error("duplicate node id: '{name}'")]
    DuplicateNodeId { name: String },
}

impl DotIoError {
    pub fn expect_token(actual: &str, expected: &str) -> Result<(), DotIoError> {
        if actual == expected {
            Ok(())
        } else {
            Err(Self::UnexpectedToken {
                expected: expected.to_string(),
                actual: actual.to_string(),
            })
        }
    }

    pub fn expect_line_start(line: &str, expected: &str) -> Result<(), DotIoError> {
        let actual = &line[..expected.len()];
        Self::expect_token(actual, expected)
    }

    pub fn expect_empty_line(line: &str) -> Result<(), DotIoError> {
        if line.is_empty() {
            Ok(())
        } else {
            Err(Self::MissingEmptyLine {
                actual: line.to_string(),
            })
        }
    }
}
