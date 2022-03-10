use std::time::SystemTimeError;

use bytes::{
    Buf,
    BufMut,
};
use prost::{
    DecodeError,
    EncodeError,
};
use thiserror::Error;

// This module hierarchy contains all the stubs generated by the protocol buffer
// compiler. They are not to be exported by this crate's public API, instead
// they will remain internal to this crate.
pub(crate) mod protobufs {
    pub(crate) mod graplinc {
        pub(crate) mod common {
            pub(crate) mod v1beta1 {
                include!(concat!(env!("OUT_DIR"), "/graplinc.common.v1beta1.rs"));
            }
        }

        pub(crate) mod grapl {
            pub(crate) mod api {
                pub(crate) mod graph {
                    pub(crate) mod v1beta1 {
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/graplinc.grapl.api.graph.v1beta1.rs"
                        ));
                    }
                }

                pub(crate) mod plugin_bootstrap {
                    pub(crate) mod v1beta1 {
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/graplinc.grapl.api.plugin_bootstrap.v1beta1.rs"
                        ));
                    }
                }

                pub(crate) mod plugin_registry {
                    pub(crate) mod v1beta1 {
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/graplinc.grapl.api.plugin_registry.v1beta1.rs"
                        ));
                    }
                }

                pub(crate) mod plugin_sdk {
                    pub(crate) mod generators {
                        pub(crate) mod v1beta1 {
                            include!(concat!(
                                env!("OUT_DIR"),
                                "/graplinc.grapl.api.plugin_sdk.generators.v1beta1.rs"
                            ));
                        }
                    }
                }

                pub(crate) mod plugin_work_queue {
                    pub(crate) mod v1beta1 {
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/graplinc.grapl.api.plugin_work_queue.v1beta1.rs"
                        ));
                    }
                }
            }

            pub(crate) mod pipeline {
                pub(crate) mod v1beta1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/graplinc.grapl.pipeline.v1beta1.rs"
                    ));
                }

                pub(crate) mod v1beta2 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/graplinc.grapl.pipeline.v1beta2.rs"
                    ));
                }
            }
        }
    }
}

// This module hierarchy closely emulates the one above, but contains our rust
// language representations of the wire format. This module hierarchy must be
// kept synchronized with the one above. This module hierarchy defines this
// crate's public API.
pub mod graplinc {
    pub mod common {
        pub mod v1beta1;
    }

    pub mod grapl {
        pub mod api {
            pub mod graph {
                pub mod v1beta1;
            }

            pub mod model_plugin_deployer {
                pub mod v1;
            }

            pub mod plugin_bootstrap {
                pub mod v1beta1;
            }

            pub mod plugin_registry {
                pub mod v1beta1;
            }

            pub mod plugin_sdk {
                pub mod generators {
                    pub mod v1beta1;
                }
            }

            pub mod plugin_work_queue {
                pub mod v1beta1;
            }
        }

        pub mod metrics {
            pub mod v1;
        }

        pub mod pipeline {
            pub mod v1beta1;
            pub mod v1beta2;
        }
    }
}

// This one's a little funky. See https://stackoverflow.com/a/53207767. The idea
// here is that we _do not want_ to expose the TYPE_URL as part of this crate's
// public API. However, in order to serialize a google.protobuf.Any message we
// require the TYPE_URL. Since prost does not support protocol buffer reflection
// features--descriptors, crucially--we are unable to determine the TYPE_URL for
// an arbitrary protocol buffer type at runtime. Therefore we resort to
// hard-coding them.
//
// The critical invariant here is that the TYPE_URL must be identically the same
// for any protocol buffer as that generated by the python implementation of
// google.protobuf.any_pb2.Any#Pack as used in python-proto. We maintain this
// invariant with round-trip regression tests which ensure every message can
// pass unmolested through all of our serialization and deserialization code.
//
// Should we find the need to work with protocol buffer stubs in a third
// language we'll add those stubs to the round-trip tests as well.
pub(crate) mod type_url {
    pub trait TypeUrl {
        const TYPE_URL: &'static str;
    }
}

#[derive(Error, Debug)]
pub enum SerDeError {
    #[error("failed to serialize {0}")]
    EncodingFailed(#[from] EncodeError),

    #[error("failed to deserialize {0}")]
    DecodingFailed(#[from] DecodeError),

    #[error("bad timestamp {0}")]
    BadTimestamp(#[from] SystemTimeError),

    #[error("missing message field {0}")]
    MissingField(String),
}

pub trait SerDe: type_url::TypeUrl {
    fn serialize<B>(self, buf: &mut B) -> Result<(), SerDeError>
    where
        B: BufMut;

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized;
}