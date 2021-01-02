
pub struct Tags {

}

pub struct Compression<'a> {
    // ex: zstd
    algorithm: &'a str,
    dictionary: Option<&'a str>,
}

pub struct EventMeta<'a> {
    compression: Compression<'a>,
    encoding: &'a str,
}

#[derive(thiserror::Error, Debug)]
pub enum EventMetaDecoderError {

}

#[derive(Debug, Default, Clone)]
pub struct EventMetaDecoder {

}

impl EventMetaDecoder {

    /// Grapl's s3 keys are of the form:
    ///
    pub fn decode_s3_key(s3_key: &str) -> Result<EventMeta, EventMetaDecoderError> {

        unimplemented!()
    }
}

pub struct DynamicDecoder {

}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn decode_s3_keys() {
        let decoder = EventMetaDecoder{};
    }
}