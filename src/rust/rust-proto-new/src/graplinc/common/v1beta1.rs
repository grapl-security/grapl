pub use std::time::{
    Duration,
    SystemTime,
};
use std::time::{
    SystemTimeError,
    UNIX_EPOCH,
};

use bytes::{
    Buf,
    BufMut,
};
use prost::Message;
pub use uuid::Uuid;

use crate::{
    protobufs::graplinc::common::v1beta1::{
        Duration as _Duration,
        Timestamp as _Timestamp,
        Uuid as _Uuid,
    },
    type_url,
    SerDe,
    SerDeError,
};

//
// Uuid
//

impl From<_Uuid> for Uuid {
    fn from(uuid_proto: _Uuid) -> Self {
        Uuid::from_u128_le(u128::from(uuid_proto.lsb) + (u128::from(uuid_proto.msb) << 64))
    }
}

impl From<Uuid> for _Uuid {
    fn from(uuid: Uuid) -> Self {
        let bytes = uuid.as_bytes();

        let mut lower: [u8; 8] = Default::default();
        lower.copy_from_slice(&bytes[..8]);

        let mut upper: [u8; 8] = Default::default();
        upper.copy_from_slice(&bytes[8..]);

        _Uuid {
            lsb: u64::from_le_bytes(lower),
            msb: u64::from_le_bytes(upper),
        }
    }
}

impl type_url::TypeUrl for Uuid {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.common.v1beta1.Uuid";
}

impl SerDe for Uuid {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _Uuid::from(self).encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let uuid_proto: _Uuid = Message::decode(buf)?;
        Ok(uuid_proto.into())
    }
}

//
// Duration
//

impl From<_Duration> for Duration {
    fn from(duration_proto: _Duration) -> Self {
        Duration::new(duration_proto.seconds, duration_proto.nanos)
    }
}

impl From<Duration> for _Duration {
    fn from(duration: Duration) -> Self {
        _Duration {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
    }
}

impl type_url::TypeUrl for Duration {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.common.v1beta1.Duration";
}

impl SerDe for Duration {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _Duration::from(self).encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let duration_proto: _Duration = Message::decode(buf)?;
        Ok(duration_proto.into())
    }
}

//
// SystemTime (a.k.a. Timestamp)
//

impl TryFrom<_Timestamp> for SystemTime {
    type Error = SerDeError;

    fn try_from(timestamp_proto: _Timestamp) -> Result<Self, Self::Error> {
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
            None => Err(SerDeError::MissingField("duration".to_string())),
        }
    }
}

impl TryFrom<SystemTime> for _Timestamp {
    type Error = SystemTimeError;

    fn try_from(timestamp: SystemTime) -> Result<Self, Self::Error> {
        if timestamp > UNIX_EPOCH {
            let duration = timestamp.duration_since(UNIX_EPOCH)?;
            let duration_proto: _Duration = duration.into();
            Ok(_Timestamp {
                duration: Some(
                    crate::protobufs::graplinc::common::v1beta1::timestamp::Duration::SinceEpoch(
                        duration_proto,
                    ),
                ),
            })
        } else {
            let duration = UNIX_EPOCH.duration_since(timestamp)?;
            let duration_proto: _Duration = duration.into();
            Ok(_Timestamp {
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

impl SerDe for SystemTime {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut,
    {
        _Timestamp::try_from(self)?.encode(buf)?;
        Ok(())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let timestamp_proto: _Timestamp = Message::decode(buf)?;
        Ok(timestamp_proto.try_into()?)
    }
}
