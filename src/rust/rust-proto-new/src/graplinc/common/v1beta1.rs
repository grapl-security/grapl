pub use std::time::{
    Duration,
    SystemTime,
};
use std::time::{
    SystemTimeError,
    UNIX_EPOCH,
};

pub use uuid::Uuid;

use crate::{
    protobufs::graplinc::common::v1beta1::{
        Duration as DurationProto,
        Timestamp as TimestampProto,
        Uuid as UuidProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// Uuid
//

impl From<UuidProto> for Uuid {
    fn from(uuid_proto: UuidProto) -> Self {
        Uuid::from_u128_le(u128::from(uuid_proto.lsb) + (u128::from(uuid_proto.msb) << 64))
    }
}

impl From<Uuid> for UuidProto {
    fn from(uuid: Uuid) -> Self {
        let bytes = uuid.as_bytes();

        let mut lower: [u8; 8] = Default::default();
        lower.copy_from_slice(&bytes[..8]);

        let mut upper: [u8; 8] = Default::default();
        upper.copy_from_slice(&bytes[8..]);

        UuidProto {
            lsb: u64::from_le_bytes(lower),
            msb: u64::from_le_bytes(upper),
        }
    }
}

impl type_url::TypeUrl for Uuid {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.common.v1beta1.Uuid";
}

impl serde_impl::ProtobufSerializable<Uuid> for Uuid {
    type ProtobufMessage = UuidProto;
}

//
// Duration
//

impl From<DurationProto> for Duration {
    fn from(duration_proto: DurationProto) -> Self {
        Duration::new(duration_proto.seconds, duration_proto.nanos)
    }
}

impl From<Duration> for DurationProto {
    fn from(duration: Duration) -> Self {
        DurationProto {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }
}

impl type_url::TypeUrl for Duration {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.common.v1beta1.Duration";
}

impl serde_impl::ProtobufSerializable<Duration> for Duration {
    type ProtobufMessage = DurationProto;
}

//
// SystemTime (a.k.a. Timestamp)
//

impl TryFrom<TimestampProto> for SystemTime {
    type Error = SerDeError;

    fn try_from(timestamp_proto: TimestampProto) -> Result<Self, Self::Error> {
        match timestamp_proto.duration {
            Some(
                crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::BeforeEpoch(
                    duration_proto,
                ),
            ) => {
                let duration: Duration = duration_proto.into();
                Ok(UNIX_EPOCH - duration)
            }
            Some(crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::SinceEpoch(
                duration_proto,
            )) => {
                let duration: Duration = duration_proto.into();
                Ok(UNIX_EPOCH + duration)
            }
            None => Err(SerDeError::MissingField("duration")),
        }
    }
}

impl TryFrom<SystemTime> for TimestampProto {
    type Error = SystemTimeError;

    fn try_from(timestamp: SystemTime) -> Result<Self, Self::Error> {
        if timestamp >= UNIX_EPOCH {
            let duration = timestamp.duration_since(UNIX_EPOCH)?;
            let duration_proto: DurationProto = duration.into();
            Ok(TimestampProto {
                duration: Some(
                    crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::SinceEpoch(
                        duration_proto,
                    ),
                ),
            })
        } else {
            let duration = UNIX_EPOCH.duration_since(timestamp)?;
            let duration_proto: DurationProto = duration.into();
            Ok(TimestampProto {
                duration: Some(
                    crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::BeforeEpoch(
                        duration_proto,
                    ),
                ),
            })
        }
    }
}

impl type_url::TypeUrl for SystemTime {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.common.v1beta1.Timestamp";
}

impl serde_impl::ProtobufSerializable<SystemTime> for SystemTime {
    type ProtobufMessage = TimestampProto;
}

#[cfg(test)]
mod tests {
    use super::UNIX_EPOCH;
    use crate::protobufs::graplinc::common::v1beta1::Timestamp as TimestampProto;

    // Check that when a SystemTime is exactly 1970-01-01T00:00:00.000000000Z it
    // is converted into a "since_epoch" protobuf Timestamp. We might state this
    // circumstance in words "it has been 0ns since epoch".
    #[test]
    fn test_epoch_timestamp_is_since_variant() {
        let timestamp = TimestampProto::try_from(UNIX_EPOCH).expect("invalid timestamp");
        match timestamp.duration {
            Some(crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::SinceEpoch(
                _,
            )) => {
                // 👍 great success 👍
            }
            Some(
                crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::BeforeEpoch(_),
            ) => {
                panic!("unix epoch must convert to a \"since_epoch\" timestamp (encountered \"before_epoch\")")
            }
            None => {
                panic!("unix epoch must convert to a \"since_epoch\" timestamp (encountered None)")
            }
        }
    }
}
