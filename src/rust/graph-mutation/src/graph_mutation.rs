use std::sync::Arc;

use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::{
    api::{
        client::ClientError,
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
        protocol::status::Status,
    },
    common::v1beta1::types::{
        EdgeName,
        NodeType,
        PropertyName,
        Uid,
    },
};
use scylla::{
    query::Query,
    CachingSession,
};
use tracing::Instrument;
use uid_allocator::client::CachingUidAllocatorClient as UidAllocatorClient;

use crate::{
    reverse_edge_resolver::{
        ReverseEdgeResolver,
        ReverseEdgeResolverError,
    },
    table_names::{
        IMM_I_64_TABLE_NAME,
        IMM_STRING_TABLE_NAME,
        IMM_U_64_TABLE_NAME,
        MAX_I_64_TABLE_NAME,
        MAX_U_64_TABLE_NAME,
        MIN_I_64_TABLE_NAME,
        MIN_U_64_TABLE_NAME,
    },
    write_dropper::WriteDropper,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationManagerError {
    #[error("gRPC client error {0}")]
    ClientError(#[from] ClientError),

    #[error("Allocated Zero Uid")]
    ZeroUid,

    #[error("Scylla Error: {0}")]
    ScyllaError(#[from] scylla::transport::errors::QueryError),

    #[error("ReverseEdgeResolverError: {0}")]
    ReverseEdgeResolverError(#[from] ReverseEdgeResolverError),
    #[error("Scylla Insert Timeout: {tenant_id:?} {insert_type:?}")]
    ScyllaInsertTimeout {
        tenant_id: uuid::Uuid,
        insert_type: &'static str,
    },
}

impl From<GraphMutationManagerError> for Status {
    fn from(e: GraphMutationManagerError) -> Self {
        match e {
            GraphMutationManagerError::ClientError(ClientError::SerDe(e)) => Status::internal(
                format!("Failed to deserialize response from UidAllocator {:?}", e),
            ),
            GraphMutationManagerError::ClientError(ClientError::Status(e)) => e,
            GraphMutationManagerError::ClientError(_) => {
                Status::internal(format!("UidAllocatorClient error: {e:?}"))
            }
            GraphMutationManagerError::ZeroUid => Status::failed_precondition("Allocated Zero Uid"),
            e => Status::internal(e.to_string()),
        }
    }
}

pub struct GraphMutationManager {
    scylla_client: Arc<CachingSession>,
    uid_allocator_client: UidAllocatorClient,
    reverse_edge_resolver: ReverseEdgeResolver,
    write_dropper: WriteDropper,
}

impl GraphMutationManager {
    pub fn new(
        scylla_client: Arc<CachingSession>,
        uid_allocator_client: UidAllocatorClient,
        reverse_edge_resolver: ReverseEdgeResolver,
        max_write_drop_size: u64,
    ) -> Self {
        Self {
            scylla_client,
            uid_allocator_client,
            reverse_edge_resolver,
            write_dropper: WriteDropper::new(max_write_drop_size),
        }
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_u64(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: u64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_max_u64(
                tenant_id,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || async move {
                    let property_value = property_value as i64;
                    let mut query = Query::new(format!(
                        "INSERT INTO tenant_graph_ks.{MAX_U_64_TABLE_NAME} \
                        (tenant_id, uid, populated_field, value) \
                        VALUES (?, ?, ?, ?)"
                    ));
                    query.set_timestamp(Some(property_value));

                    self.scylla_client
                        .execute(
                            query,
                            &(tenant_id, uid.as_i64(), property_name.value, property_value),
                        )
                        .await?;
                    Ok(())
                },
            )
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_min_u64(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: u64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_min_u64(
                tenant_id,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || {
                    async move {
                        let property_value = property_value as i64;
                        let mut query = Query::new(format!(
                            "INSERT INTO tenant_graph_ks.{MIN_U_64_TABLE_NAME} \
                            (tenant_id, uid, populated_field, value) \
                            VALUES (?, ?, ?, ?)"
                        ));

                        query.set_timestamp(Some(-property_value));

                        self.scylla_client
                            .execute(
                                query,
                                &(tenant_id, uid.as_i64(), property_name.value, property_value),
                            )
                            .timeout(std::time::Duration::from_secs(3))
                            .await
                            .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                                tenant_id,
                                insert_type: "MIN_U_64",
                            })??;
                        Ok(())
                    }
                    .instrument(tracing::info_span!("upsert_min_u64"))
                },
            )
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_immutable_u64(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: u64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_imm_u64(tenant_id, node_type.clone(), property_name.clone(), || {
                async move {
                    let property_value = property_value as i64;
                    let query = Query::new(format!(
                        r"
                        INSERT INTO tenant_graph_ks.{IMM_U_64_TABLE_NAME}
                        (tenant_id, uid, populated_field, value)
                        VALUES (?, ?, ?, ?)
                    "
                    ));

                    self.scylla_client
                        .execute(
                            query,
                            &(tenant_id, uid.as_i64(), property_name.value, property_value),
                        )
                        .timeout(std::time::Duration::from_secs(3))
                        .await
                        .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                            tenant_id,
                            insert_type: "MAX_U_64",
                        })??;
                    Ok(())
                }
                .instrument(tracing::info_span!("upsert_max_u64"))
            })
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_i64(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: i64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_max_i64(
                tenant_id,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || {
                    async move {
                        let mut query = Query::new(format!(
                            r"
                        INSERT INTO tenant_graph_ks.{MAX_I_64_TABLE_NAME}
                        (tenant_id, uid, populated_field, value)
                        VALUES (?, ?, ?, ?)
                    "
                        ));
                        query.set_timestamp(Some(property_value));

                        self.scylla_client
                            .execute(
                                query,
                                &(tenant_id, uid.as_i64(), property_name.value, property_value),
                            )
                            .timeout(std::time::Duration::from_secs(3))
                            .await
                            .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                                tenant_id,
                                insert_type: "MAX_I_64",
                            })??;
                        Ok(())
                    }
                    .instrument(tracing::info_span!("upsert_max_i64"))
                },
            )
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_min_i64(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: i64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_min_i64(
                tenant_id,
                node_type.clone(),
                property_name.clone(),
                property_value,
                || {
                    async move {
                        let mut query = Query::new(format!(
                            r"
                                INSERT INTO tenant_graph_ks.{MIN_I_64_TABLE_NAME}
                                (tenant_id, uid, populated_field, value)
                                VALUES (?, ?, ?, ?)
                            "
                        ));
                        query.set_timestamp(Some(-property_value));

                        self.scylla_client
                            .execute(
                                query,
                                &(
                                    &tenant_id,
                                    uid.as_i64(),
                                    property_name.value,
                                    property_value,
                                ),
                            )
                            .timeout(std::time::Duration::from_secs(3))
                            .await
                            .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                                tenant_id,
                                insert_type: "MIN_I_64",
                            })??;
                        Ok(())
                    }
                    .instrument(tracing::info_span!("upsert_min_i64"))
                },
            )
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_immutable_i64(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: i64,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_imm_i64(tenant_id, node_type.clone(), property_name.clone(), || {
                async move {
                    let query = Query::new(format!(
                        "INSERT INTO tenant_graph_ks.{IMM_I_64_TABLE_NAME} \
                        (tenant_id, uid, populated_field, value) \
                        VALUES (?, ?, ?, ?)\
                    "
                    ));

                    self.scylla_client
                        .execute(
                            query,
                            &(tenant_id, uid.as_i64(), property_name.value, property_value),
                        )
                        .timeout(std::time::Duration::from_secs(3))
                        .await
                        .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                            tenant_id,
                            insert_type: "IMM_I_64",
                        })??;
                    Ok(())
                }
                .instrument(tracing::info_span!("upsert_imm_i64"))
            })
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn set_node_type(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_node_type(tenant_id, uid, || {
                async move {
                    let query = Query::new(
                        "INSERT INTO tenant_graph_ks.node_type \
                        (tenant_id, uid, node_type) \
                        VALUES (?, ?, ?)",
                    );

                    self.scylla_client
                        .execute(query, &(tenant_id, uid.as_i64(), node_type.value))
                        .timeout(std::time::Duration::from_secs(3))
                        .await
                        .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                            tenant_id,
                            insert_type: "NODE_TYPE",
                        })??;
                    Ok(())
                }
                .instrument(tracing::info_span!("set_node_type"))
            })
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_immutable_string(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: NodeType,
        property_name: PropertyName,
        property_value: String,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_imm_string(tenant_id, node_type.clone(), property_name.clone(), || {
                async move {
                    let query = Query::new(format!(
                        "INSERT INTO tenant_graph_ks.{IMM_STRING_TABLE_NAME} \
                        (tenant_id, uid, populated_field, value) \
                        VALUES (?, ?, ?, ?)"
                    ));

                    self.scylla_client
                        .execute(
                            query,
                            &(tenant_id, uid.as_i64(), property_name.value, property_value),
                        )
                        .timeout(std::time::Duration::from_secs(3))
                        .await
                        .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                            tenant_id,
                            insert_type: "IMM_STRING",
                        })??;
                    Ok(())
                }
                .instrument(tracing::info_span!("upsert_imm_string"))
            })
            .await
            .map(|_| ())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_edges(
        &self,
        tenant_id: uuid::Uuid,
        from_uid: Uid,
        to_uid: Uid,
        f_edge_name: EdgeName,
        r_edge_name: EdgeName,
    ) -> Result<(), GraphMutationManagerError> {
        self.write_dropper
            .check_edges(
                tenant_id,
                from_uid,
                to_uid,
                f_edge_name,
                r_edge_name,
                |f_edge_name, r_edge_name| {
                    async move {
                        let f_statement = "INSERT INTO tenant_graph_ks.edges (\
                                tenant_id, \
                                source_uid, \
                                destination_uid, \
                                f_edge_name, \
                                r_edge_name\
                            ) \
                            VALUES (?, ?, ?, ?, ?)"
                            .to_string();
                        let r_statement = f_statement.clone();

                        let mut batch: scylla::batch::Batch = Default::default();
                        batch.statements.reserve(2);
                        batch.append_statement(Query::from(f_statement));
                        batch.append_statement(Query::from(r_statement));
                        batch.set_is_idempotent(true);

                        self.scylla_client
                            .batch(
                                &batch,
                                (
                                    (
                                        tenant_id,
                                        from_uid.as_i64(),
                                        to_uid.as_i64(),
                                        &f_edge_name.value,
                                        &r_edge_name.value,
                                    ),
                                    (
                                        tenant_id,
                                        to_uid.as_i64(),
                                        from_uid.as_i64(),
                                        &r_edge_name.value,
                                        &f_edge_name.value,
                                    ),
                                ),
                            )
                            .timeout(std::time::Duration::from_secs(3))
                            .await
                            .map_err(|_| GraphMutationManagerError::ScyllaInsertTimeout {
                                tenant_id,
                                insert_type: "EDGES",
                            })??;
                        Ok(())
                    }
                    .instrument(tracing::info_span!("upsert_edges"))
                },
            )
            .await
            .map(|_| ())
    }
}

#[async_trait::async_trait]
impl GraphMutationApi for GraphMutationManager {
    type Error = GraphMutationManagerError;

    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    #[tracing::instrument(skip(self), err)]
    async fn create_node(
        &self,
        request: CreateNodeRequest,
    ) -> Result<CreateNodeResponse, Self::Error> {
        tracing::debug!(message = "Creating node",);
        let uid = self
            .uid_allocator_client
            .allocate_id(request.tenant_id)
            .await?;
        let uid = Uid::from_u64(uid).ok_or_else(|| GraphMutationManagerError::ZeroUid)?;
        tracing::debug!(
            message="Allocated uid",
            uid=?uid,
        );
        self.set_node_type(request.tenant_id, uid, request.node_type)
            .await?;
        tracing::debug!(
            message="Set node type",
            uid=?uid,
        );

        Ok(CreateNodeResponse { uid })
    }

    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    #[tracing::instrument(
    skip(self),
    fields(
        tenant_id=?request.tenant_id,
        property_name=?request.property_name,
    ), err)]
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
        tracing::debug!(
            message="Setting node property",
            uid=?uid,
            property_name=?property_name,
        );
        match property.property {
            Property::IncrementOnlyUintProp(property) => {
                self.upsert_max_u64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?;
            }
            Property::DecrementOnlyUintProp(property) => {
                self.upsert_min_u64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?;
            }
            Property::ImmutableUintProp(property) => {
                self.upsert_immutable_u64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?;
            }
            Property::IncrementOnlyIntProp(property) => {
                self.upsert_max_i64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?;
            }
            Property::DecrementOnlyIntProp(property) => {
                self.upsert_min_i64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?;
            }
            Property::ImmutableIntProp(property) => {
                self.upsert_immutable_i64(tenant_id, uid, node_type, property_name, property.prop)
                    .await?;
            }
            Property::ImmutableStrProp(property) => {
                self.upsert_immutable_string(
                    tenant_id,
                    uid,
                    node_type,
                    property_name,
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
    #[tracing::instrument(skip(self), err)]
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
        } = request;

        let reverse_edge_name = self
            .reverse_edge_resolver
            .resolve_reverse_edge(tenant_id, source_node_type.clone(), edge_name.clone())
            .await?;

        self.upsert_edges(tenant_id, from_uid, to_uid, edge_name, reverse_edge_name)
            .await?;

        Ok(CreateEdgeResponse {
            // todo: At this point we don't track if the update was redundant
            //       but it is always safe (albeit suboptimal) to assume that
            //       it was not.
            mutation_redundancy: MutationRedundancy::Maybe,
        })
    }
}
