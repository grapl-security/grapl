use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::GraphDescription,
    pipeline::v1beta1::Envelope,
};

pub fn generator_produce_graph_description(
    graph_description: GraphDescription,
) -> Envelope<GraphDescription> {
    // We'd likely want to go look up in Plugin-Registry which Tenant ID this
    // Plugin belongs to... and store that in an LRU cache... yikes!
    let tenant_id = uuid::Uuid::new_v4(); // FIXME // TODO
    let trace_id = uuid::Uuid::new_v4(); // FIXME // TODO
    let event_source_id = uuid::Uuid::new_v4(); // FIXME // TODO
    Envelope::new(tenant_id, trace_id, event_source_id, graph_description)
}
