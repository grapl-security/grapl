use std::sync::Arc;

use rust_proto_new::{
    graplinc::grapl::{
        api::{
            graph::v1beta1::Property,
            graph_mutation::v1beta1::{
                messages::{
                    CreateEdgeRequest,
                    CreateEdgeResponse,
                    CreateNodeRequest,
                    CreateNodeResponse,
                    MutationRedundancy,
                    SetNodePropertyRequest,
                    SetNodePropertyResponse,
                },
                server::GraphMutationApi,
            },
            uid_allocator::v1beta1::client::UidAllocatorServiceClientError,
        },
        common::v1beta1::types::Uid,
    },
    protocol::status::Status,
};
use scylla::Session;
use uid_allocator::client::CachingUidAllocatorServiceClient as UidAllocatorClient;

use crate::{
    prepared_statements::{
        PreparedStatements,
        PreparedStatementsError,
    },
    reverse_edge_resolver::{
        ReverseEdgeResolver,
        ReverseEdgeResolverError,
    },
    write_dropper::WriteDropper,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationManagerError {
    #[error("UidAllocatorServiceClientError {0}")]
    UidAllocatorServiceClientError(#[from] UidAllocatorServiceClientError),
    #[error("Allocated Zero Uid")]
    ZeroUid,
    #[error("Scylla Error {0}")]
    ScyllaError(#[from] scylla::transport::errors::QueryError),
    #[error("PreparedStatementsError {0}")]
    PreparedStatementsError(#[from] PreparedStatementsError),
    #[error("ReverseEdgeResolverError {0}")]
    ReverseEdgeResolverError(#[from] ReverseEdgeResolverError),
}

impl From<GraphMutationManagerError> for Status {
    fn from(e: GraphMutationManagerError) -> Self {
        match e {
            GraphMutationManagerError::UidAllocatorServiceClientError(
                UidAllocatorServiceClientError::SerDeError(e),
            ) => Status::internal(format!(
                "Failed to deserialize response from UidAllocator {:?}",
                e
            )),
            GraphMutationManagerError::UidAllocatorServiceClientError(
                UidAllocatorServiceClientError::Status(e),
            ) => e,
            GraphMutationManagerError::UidAllocatorServiceClientError(
                UidAllocatorServiceClientError::ConnectError(e),
            ) => Status::internal(format!("Failed to connect to UidAllocator {:?}", e)),
            GraphMutationManagerError::ZeroUid => Status::failed_precondition("Allocated Zero Uid"),
            e => Status::internal(e.to_string()),
        }
    }
}

pub struct GraphMutationManager {
    scylla_client: Arc<Session>,
    prepared_statements: PreparedStatements,
    uid_allocator_client: UidAllocatorClient,
    reverse_edge_resolver: ReverseEdgeResolver,
    write_dropper: WriteDropper,
}

impl GraphMutationManager {
    pub fn new(
        scylla_client: Arc<Session>,
        uid_allocator_client: UidAllocatorClient,
        reverse_edge_resolver: ReverseEdgeResolver,
    ) -> Self {
        Self {
            scylla_client,
            prepared_statements: PreparedStatements::new(),
            uid_allocator_client,
            reverse_edge_resolver,
            write_dropper: WriteDropper::default(),
        }
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_u64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: u64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_max_u64(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                property_value,
                || async move {
                    let property_value = property_value as i64;
                    // Create a prepared statement, and then execute it
                    let mut statement = self
                        .prepared_statements
                        .prepare_max_u64(&self.scylla_client, tenant_keyspace)
                        .await?;

                    statement.set_timestamp(Some(property_value));

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_min_u64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: u64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_min_u64(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                property_value,
                || async move {
                    let property_value = property_value as i64;
                    // Create a prepared statement, and then execute it
                    let mut statement = self
                        .prepared_statements
                        .prepare_min_u64(&self.scylla_client, tenant_keyspace)
                        .await?;

                    statement.set_timestamp(Some(-property_value));

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_immutable_u64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: u64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_imm_u64(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                || async move {
                    let property_value = property_value as i64;
                    // todo: We should only prepare statements once
                    let mut statement = self
                        .prepared_statements
                        .prepare_imm_u64(&self.scylla_client, tenant_keyspace)
                        .await?;

                    // Immutable values can never be overwritten
                    statement.set_timestamp(Some(1i64));

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_i64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: i64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_max_i64(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                property_value,
                || async move {
                    // Create a prepared statement, and then execute it
                    let mut statement = self
                        .prepared_statements
                        .prepare_max_i64(&self.scylla_client, tenant_keyspace)
                        .await?;

                    statement.set_timestamp(Some(property_value));

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_min_i64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: i64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_min_i64(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                property_value,
                || async move {
                    // Create a prepared statement, and then execute it
                    let mut statement = self
                        .prepared_statements
                        .prepare_min_i64(&self.scylla_client, tenant_keyspace)
                        .await?;

                    statement.set_timestamp(Some(-property_value));

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_immutable_i64(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: i64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_imm_i64(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                || async move {
                    // todo: We should only prepare statements once
                    let statement = self
                        .prepared_statements
                        .prepare_imm_i64(&self.scylla_client, tenant_keyspace)
                        .await?;

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn set_node_type(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_node_type(tenant_keyspace, uid.as_u64(), || async move {
                // todo: Should we only prepare statements once?
                let statement = self
                    .prepared_statements
                    .prepare_node_type(&self.scylla_client, tenant_keyspace)
                    .await?;

                self.scylla_client
                    .execute(&statement, (uid.as_i64(), node_type))
                    .await?;
                Ok(())
            })
            .await
    }

    async fn upsert_immutable_string(
        &self,
        tenant_keyspace: uuid::Uuid,
        uid: Uid,
        node_type: String,
        property_name: String,
        property_value: String,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_imm_string(
                tenant_keyspace,
                node_type.to_owned(),
                property_name.to_owned(),
                || async move {
                    // todo: Should we only prepare statements once?
                    let statement = self
                        .prepared_statements
                        .prepare_imm_string(&self.scylla_client, tenant_keyspace)
                        .await?;

                    self.scylla_client
                        .execute(
                            &statement,
                            (uid.as_i64(), node_type, property_name, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }

    async fn upsert_edges(
        &self,
        tenant_keyspace: uuid::Uuid,
        from_uid: Uid,
        to_uid: Uid,
        f_edge_name: String,
        r_edge_name: String,
        source_node_type: String,
        dest_node_type: String,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_edges(
                tenant_keyspace,
                from_uid.as_u64(),
                to_uid.as_u64(),
                f_edge_name.clone(),
                r_edge_name.clone(),
                || async move {
                    let f_statement = self
                        .prepared_statements
                        .prepare_edge(&self.scylla_client, tenant_keyspace)
                        .await?;
                    let r_statement = f_statement.clone();

                    let mut batch: scylla::batch::Batch = Default::default();
                    batch.append_statement(f_statement);
                    batch.append_statement(r_statement);
                    batch.set_is_idempotent(true);

                    self.scylla_client
                        .batch(
                            &batch,
                            (
                                (
                                    from_uid.as_i64(),
                                    &f_edge_name,
                                    &r_edge_name,
                                    to_uid.as_i64(),
                                    &source_node_type,
                                    &dest_node_type,
                                ),
                                (
                                    to_uid.as_i64(),
                                    &r_edge_name,
                                    &f_edge_name,
                                    from_uid.as_i64(),
                                    &dest_node_type,
                                    &source_node_type,
                                ),
                            ),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
    }
}

#[async_trait::async_trait]
impl GraphMutationApi for GraphMutationManager {
    type Error = GraphMutationManagerError;

    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(
        &self,
        request: CreateNodeRequest,
    ) -> Result<CreateNodeResponse, Self::Error> {
        let uid = self
            .uid_allocator_client
            .allocate_id(request.tenant_id)
            .await?;
        let uid = Uid::from_u64(uid).ok_or_else(|| GraphMutationManagerError::ZeroUid)?;

        self.set_node_type(request.tenant_id, uid, request.node_type.value)
            .await?;

        Ok(CreateNodeResponse { uid })
    }

    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(
        &self,
        request: SetNodePropertyRequest,
    ) -> Result<SetNodePropertyResponse, Self::Error> {
        let SetNodePropertyRequest {
            tenant_id,
            uid,
            node_type,
            property_name,
            property,
        } = request;
        match property.property {
            Property::IncrementOnlyUintProp(property) => {
                self.upsert_max_u64(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
            Property::DecrementOnlyUintProp(property) => {
                self.upsert_min_u64(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
            Property::ImmutableUintProp(property) => {
                self.upsert_immutable_u64(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
            Property::IncrementOnlyIntProp(property) => {
                self.upsert_max_i64(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
            Property::DecrementOnlyIntProp(property) => {
                self.upsert_min_i64(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
            Property::ImmutableIntProp(property) => {
                self.upsert_immutable_i64(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
            Property::ImmutableStrProp(property) => {
                self.upsert_immutable_string(
                    tenant_id,
                    uid,
                    node_type.value,
                    property_name.value,
                    property.prop,
                )
                .await?;
            }
        };

        Ok(SetNodePropertyResponse {
            // todo: At this point we can't tell if the update was redundant
            //       but it is always safe (albeit suboptimal) to assume that
            //       it was not.
            mutation_redundancy: MutationRedundancy::Maybe,
        })
    }

    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(
        &self,
        request: CreateEdgeRequest,
    ) -> Result<CreateEdgeResponse, Self::Error> {
        let CreateEdgeRequest {
            edge_name,
            tenant_id,
            from_uid,
            to_uid,
            source_node_type,
            dest_node_type,
        } = request;

        let reverse_edge_name = self
            .reverse_edge_resolver
            .resolve_reverse_edge(
                tenant_id,
                source_node_type.value.clone(),
                edge_name.value.clone(),
            )
            .await?;

        self.upsert_edges(
            tenant_id,
            from_uid,
            to_uid,
            edge_name.value,
            reverse_edge_name,
            source_node_type.value,
            dest_node_type.value,
        )
        .await?;

        Ok(CreateEdgeResponse {
            // todo: At this point we can't tell if the update was redundant
            //       but it is always safe (albeit suboptimal) to assume that
            //       it was not.
            mutation_redundancy: MutationRedundancy::Maybe,
        })
    }
}
