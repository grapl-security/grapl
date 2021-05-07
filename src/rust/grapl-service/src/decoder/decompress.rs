use std::{
    io::{
        Cursor,
        Read,
    },
    str::FromStr,
    sync::Arc,
};

use libflate::gzip::Decoder as GzDecoder;
use sqs_executor::errors::{
    CheckedError,
    Recoverable,
};

#[derive(Debug, Clone, PartialEq)]
pub enum PayloadDecompression {
    Gzip,
    None,
    Zstd,
}

impl FromStr for PayloadDecompression {
    type Err = ();

    fn from_str(input: &str) -> Result<PayloadDecompression, Self::Err> {
        match input.to_lowercase().as_str() {
            "gzip" => Ok(PayloadDecompression::Gzip),
            "none" => Ok(PayloadDecompression::None),
            "zstd" => Ok(PayloadDecompression::Zstd),
            _ => Err(()),
        }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum PayloadDecompressionError {
    #[error("DecompressionError")]
    DecompressionError(Arc<std::io::Error>),
}

impl From<std::io::Error> for PayloadDecompressionError {
    fn from(err: std::io::Error) -> Self {
        PayloadDecompressionError::DecompressionError(Arc::new(err))
    }
}

impl CheckedError for PayloadDecompressionError {
    fn error_type(&self) -> Recoverable {
        match self {
            Self::DecompressionError(_) => Recoverable::Persistent,
        }
    }
}

pub fn maybe_decompress(input: &[u8]) -> Result<Vec<u8>, PayloadDecompressionError> {
    let value = grapl_config::source_compression();
    match PayloadDecompression::from_str(value.as_str()).expect("PayloadDecompression") {
        PayloadDecompression::Gzip => {
            let mut decoder = GzDecoder::new(input)?;
            let mut decoded_data = Vec::with_capacity(input.len());

            decoder.read_to_end(&mut decoded_data)?;

            Ok(decoded_data)
        }
        PayloadDecompression::None => Ok(input.to_vec()),
        PayloadDecompression::Zstd => {
            let body = Cursor::new(input);

            zstd::stream::decode_all(body).map_err(|e| e.into())
        }
    }
}
