use proptest::prelude::*;
use rust_proto_new::{
    graplinc::{
        common::v1beta1::{
            Duration,
            SystemTime,
            Uuid,
        },
        grapl::pipeline::{
            v1beta1::{
                Envelope,
                Metadata,
                RawLog,
            },
            v1beta2::Envelope,
        },
    },
    SerDe,
    SerDeError,
};

fn todo() {
    todo!()
}
