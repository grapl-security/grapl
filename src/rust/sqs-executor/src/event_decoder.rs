use crate::errors::CheckedError;
use std::error::Error;

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
