use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Represents all possible errors that can occur when parsing Sysmon XML events.
#[non_exhaustive]
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("event is not `{0}`")]
    ExpectEventType(&'static str),
    #[error(transparent)]
    IpAddrParseError(#[from] std::net::AddrParseError),
    #[error("invalid XML character reference `{0}`")]
    InvalidXmlCharacterReference(String),
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error(transparent)]
    ParseBoolError(#[from] std::str::ParseBoolError),
    #[error(transparent)]
    ParseDateTime(#[from] chrono::ParseError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseUuid(#[from] uuid::Error),
    #[error(transparent)]
    XmlError(#[from] xmlparser::Error),
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Error {
        unreachable!()
    }
}
