use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::client::GraphMutationClient;
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::messages::CreateNodeRequest;
use rust_proto_new::graplinc::grapl::common::v1beta1::NodeType;

struct LensManager {
    graph_mutation_client: GraphMutationClient,
}

impl LensManager {

    async fn create_lens(&self, tenant_id: uuid::Uuid) -> Result<Uid, Box<dyn std::error::Error>> {
        self.graph_mutation_client.create_node(
            CreateNodeRequest {
                tenant_id,
                NodeType { value: "Lens".to_owned() },
            }
        ).await?;

        // todo: Publish update to lens-updates topic

        todo!()
    }
}


fn main() {
    println!("Hello, world!");
}
