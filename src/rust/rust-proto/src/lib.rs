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
            pub(crate) mod organization_management {
                pub(crate) mod v1beta1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/graplinc.grapl.api.organization_management.v1beta1.rs"
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

pub mod organization_management;
pub mod pipeline;
pub mod plugin_bootstrap;
pub mod plugin_sdk;
pub mod types;
