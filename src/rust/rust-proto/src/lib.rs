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
            pub(crate) mod plugin_registry {
                pub(crate) mod v1beta1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/graplinc.grapl.api.plugin_registry.v1beta1.rs"
                    ));
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
            pub mod v1beta1 {
                include!(concat!(
                    env!("OUT_DIR"),
                    "/graplinc.grapl.pipeline.v1beta1.rs"
                ));
            }
        }
    }
}

pub mod graph_descriptions;
pub use graph_descriptions::node_property;

pub mod pipeline;
pub mod types;
