use std::{error::Error,
          fmt::{Display,
                Formatter,
                Result as FmtResult}};

use sqs_executor::errors::{CheckedError,
                           Recoverable};

#[derive(Debug)]
pub struct GenericError {}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Error")
    }
}

impl Error for GenericError {}

impl CheckedError for GenericError {
    fn error_type(&self) -> Recoverable {
        Recoverable::Transient
    }
}
