use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum MotorError {
    InvalidArgument(String),
    Io(String),
    Timeout(String),
    Protocol(String),
    Unsupported(String),
}

impl Display for MotorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(msg)
            | Self::Io(msg)
            | Self::Timeout(msg)
            | Self::Protocol(msg)
            | Self::Unsupported(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for MotorError {}

pub type Result<T> = std::result::Result<T, MotorError>;

impl From<std::io::Error> for MotorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
