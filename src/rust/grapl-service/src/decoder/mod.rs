pub mod zstd;
pub mod zstd_json;
pub mod zstd_ndjson;
pub mod zstd_proto;

pub use zstd_json::{ZstdJsonDecoder,
                    ZstdJsonDecoderError};
pub use zstd_ndjson::{ZstdNdjsonDecoder,
                      ZstdNdjsonDecoderError};
pub use zstd_proto::{ZstdProtoDecoder,
                     ZstdProtoDecoderError};

pub use self::zstd::{ZstdDecoder,
                     ZstdDecoderError};
