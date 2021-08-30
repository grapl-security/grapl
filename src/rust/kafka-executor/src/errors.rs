use thiserror::Error;

#[derive(Eq, PartialEq, Debug)]
pub enum Recoverable {
    Transient,
    Persistent,
}

pub trait CheckedError: std::error::Error {
    fn error_type(&self) -> Recoverable;

    fn is_transient(&self) -> bool {
        self.error_type() == Recoverable::Transient
    }

    fn is_persistent(&self) -> bool {
        self.error_type() == Recoverable::Persistent
    }
}
