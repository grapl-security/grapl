use std::error::Error;

pub trait PayloadDecoder<E> {
    fn decode(&mut self, bytes: Vec<u8>) -> Result<E, Box<dyn Error>>;
}

impl<T, F> PayloadDecoder<T> for F
    where
        F: Fn(Vec<u8>) -> Result<T, Box<dyn std::error::Error>>,
{
    fn decode(&mut self, body: Vec<u8>) -> Result<T, Box<dyn std::error::Error>> {
        (self)(body)
    }
}
