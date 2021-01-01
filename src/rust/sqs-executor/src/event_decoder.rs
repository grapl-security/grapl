use std::error::Error;
use crate::errors::CheckedError;

pub trait PayloadDecoder<E> {
    type DecoderError: CheckedError;
    fn decode(&mut self, bytes: Vec<u8>) -> Result<E, Self::DecoderError>;
}

impl<T, F, E> PayloadDecoder<T> for F
where
    F: Fn(Vec<u8>) -> Result<T, E>,
    E: CheckedError,
{
    type DecoderError = E;
    fn decode(&mut self, body: Vec<u8>) -> Result<T, Self::DecoderError> {
        (self)(body)
    }
}
