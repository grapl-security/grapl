//<prefix>-<event-type>/<day>/<seconds>/<serialization>/<compression>-<capability>

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

pub trait KeyGenerator {
    fn generate_key(&mut self, capability: u64) -> String;
}

pub struct S3KeyGenerator {
    bucket: String,
    serialization: String,
    compression: String,
}

impl S3KeyGenerator {
    pub fn new(
        bucket: impl Into<String>,
        serialization: impl Into<String>,
        compression: impl Into<String>,
    ) -> Self {
        Self {
            bucket: bucket.into(),
            serialization: serialization.into(),
            compression: compression.into(),
        }
    }
}

impl KeyGenerator for S3KeyGenerator {
    fn generate_key(&mut self, capability: u64) -> String {
        let cur_secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let cur_day = cur_secs - (cur_secs % 86400);

        format!(
            "{bucket}/{day}/{seconds}/{serialization}/{compression}-{capability}",
            bucket = self.bucket,
            day = cur_day,
            seconds = cur_secs,
            serialization = self.serialization,
            compression = self.compression,
            capability = capability,
        )
    }
}

pub struct ZstdProtoKeyGenerator(S3KeyGenerator);

impl ZstdProtoKeyGenerator {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self(S3KeyGenerator::new(bucket, "proto_v0", "zstd_nodict"))
    }
}

impl KeyGenerator for ZstdProtoKeyGenerator {
    fn generate_key(&mut self, capability: u64) -> String {
        self.0.generate_key(capability)
    }
}
