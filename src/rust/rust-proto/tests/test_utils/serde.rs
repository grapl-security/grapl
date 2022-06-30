use std::fmt::Debug;

use rust_proto::{
    SerDe,
    SerDeError,
};

// helper function to define a simple encode-decode invariant
// see: https://hypothesis.works/articles/encode-decode-invariant/
pub fn check_encode_decode_invariant<T>(serializable: T)
where
    T: SerDe + PartialEq + Clone + Debug,
{
    let cloned = serializable.clone();
    let serialized = serializable.serialize().expect("serialization failed");
    let deserialized = T::deserialize(serialized).expect("deserialization failed");
    assert!(cloned == deserialized);
}

pub fn expect_serde_error<T>(serializable: T) -> SerDeError
where
    T: SerDe + PartialEq + Clone + Debug,
{
    let round_trip = serializable.serialize().and_then(T::deserialize);
    round_trip.expect_err("Expected a SerDeError")
}
