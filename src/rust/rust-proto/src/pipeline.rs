pub use grapl_pipeline::{
    Envelope,
    Metadata,
    RawLog,
};

pub use crate::graplinc::grapl::pipeline::v1beta1 as grapl_pipeline;

pub trait ServiceMessage {
    const TYPE_NAME: &'static str;
}
