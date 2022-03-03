use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Represents all possible errors that can occur when parsing Sysmon XML events.
#[non_exhaustive]
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("event is not `{0}`")]
    ExpectEventType(&'static str),
    #[error("failed to parse IP address `{value}` at position `{position}` with `{source}`")]
    ParseIpAddress {
        value: String,
        position: usize,
        source: std::net::AddrParseError,
    },
    #[error("invalid XML character reference `{0}`")]
    InvalidXmlCharacterReference(String),
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error("failed to parse bool `{value}` at position `{position}` with `{source}`")]
    ParseBool {
        value: String,
        position: usize,
        source: std::str::ParseBoolError,
    },
    #[error("failed to parse datetime `{value}` with format `{format:?}` at position `{position}` with `{source}`")]
    ParseDateTime {
        value: String,
        position: usize,
        format: Option<String>,
        source: chrono::ParseError,
    },
    #[error("failed to parse integer `{value}` at position `{position}` with `{source}`")]
    ParseInt {
        value: String,
        position: usize,
        source: std::num::ParseIntError,
    },
    #[error("failed to parse uuid `{value}` at position `{position}` with `{source}`")]
    ParseUuid {
        value: String,
        position: usize,
        source: uuid::Error,
    },
    // these errors are useful just as they are
    #[error(transparent)]
    XmlError(#[from] xmlparser::Error),
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Error {
        unreachable!()
    }
}
