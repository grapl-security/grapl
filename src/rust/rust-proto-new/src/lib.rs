pub mod pipeline;

use bytes::{Buf, BufMut};

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
        }
    }
}

pub enum SerDeError {
    FOO
}

pub trait SerDe {
    fn serialize(&self, buf: &mut dyn BufMut) -> Result<(), SerDeError>;
    fn deserialize(buf: &dyn Buf) -> Result<Self, SerDeError> where Self: Sized;
}
