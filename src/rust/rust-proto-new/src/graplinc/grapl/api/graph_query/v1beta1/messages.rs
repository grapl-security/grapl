use std::collections::HashMap;

use crate::{
    graplinc::grapl::{
        api::graph::v1beta1::{
            DecrementOnlyIntProp,
            ImmutableIntProp,
            IncrementOnlyIntProp,
        },
        common::v1beta1::types::{
            EdgeName,
            NodeType,
            PropertyName,
            Uid,
        },
    },
    protobufs::graplinc::grapl::api::graph_query::v1beta1::{
        int_filter::Operation as IntOperationProto,
        integer_property as integer_property_proto,
        string_filter::Operation as StringOperationProto,
        uid_filter::Operation as UidOperationProto,
        AndIntFilters as AndIntFiltersProto,
        AndStringFilters as AndStringFiltersProto,
        EdgeFilters as EdgeFiltersProto,
        EdgeView as EdgeViewProto,
        EdgeViews as EdgeViewsProto,
        GraphView as GraphViewProto,
        IntFilter as IntFilterProto,
        IntegerProperty as IntegerPropertyProto,
        NodeQuery as NodeQueryProto,
        NodeView as NodeViewProto,
        OrIntFilters as OrIntFiltersProto,
        OrStringFilters as OrStringFiltersProto,
        QueryGraphFromNodeRequest as QueryGraphFromNodeRequestProto,
        QueryGraphFromNodeResponse as QueryGraphFromNodeResponseProto,
        QueryGraphWithNodeRequest as QueryGraphWithNodeRequestProto,
        QueryGraphWithNodeResponse as QueryGraphWithNodeResponseProto,
        StringFilter as StringFilterProto,
        UidFilter as UidFilterProto,
        UidFilters as UidFiltersProto,
    },
    SerDeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntegerProperty {
    IncrementOnlyInt(IncrementOnlyIntProp),
    DecrementOnlyInt(DecrementOnlyIntProp),
    ImmutableInt(ImmutableIntProp),
}

impl TryFrom<IntegerPropertyProto> for IntegerProperty {
    type Error = SerDeError;
    fn try_from(value_proto: IntegerPropertyProto) -> Result<Self, Self::Error> {
        match value_proto.property {
            Some(integer_property_proto::Property::IncrementOnlyInt(p)) => {
                Ok(IntegerProperty::IncrementOnlyInt(p.try_into()?))
            }
            Some(integer_property_proto::Property::DecrementOnlyInt(p)) => {
                Ok(IntegerProperty::DecrementOnlyInt(p.try_into()?))
            }
            Some(integer_property_proto::Property::ImmutableInt(p)) => {
                Ok(IntegerProperty::ImmutableInt(p.try_into()?))
            }
            None => Err(SerDeError::UnknownVariant("IntegerProperty")),
        }
    }
}

impl From<IntegerProperty> for IntegerPropertyProto {
    fn from(value: IntegerProperty) -> Self {
        match value {
            IntegerProperty::IncrementOnlyInt(p) => IntegerPropertyProto {
                property: Some(integer_property_proto::Property::IncrementOnlyInt(p.into())),
            },
            IntegerProperty::DecrementOnlyInt(p) => IntegerPropertyProto {
                property: Some(integer_property_proto::Property::DecrementOnlyInt(p.into())),
            },
            IntegerProperty::ImmutableInt(p) => IntegerPropertyProto {
                property: Some(integer_property_proto::Property::ImmutableInt(p.into())),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IntOperation {
    Has,
    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl TryFrom<IntOperationProto> for IntOperation {
    type Error = SerDeError;
    fn try_from(value_proto: IntOperationProto) -> Result<Self, Self::Error> {
        match value_proto {
            IntOperationProto::UnknownOperation => Err(SerDeError::UnknownVariant("IntOperation")),
            IntOperationProto::Has => Ok(Self::Has),
            IntOperationProto::Equal => Ok(Self::Equal),
            IntOperationProto::LessThan => Ok(Self::LessThan),
            IntOperationProto::LessThanOrEqual => Ok(Self::LessThanOrEqual),
            IntOperationProto::GreaterThan => Ok(Self::GreaterThan),
            IntOperationProto::GreaterThanOrEqual => Ok(Self::GreaterThanOrEqual),
        }
    }
}

impl From<IntOperation> for IntOperationProto {
    fn from(value: IntOperation) -> Self {
        match value {
            IntOperation::Has => IntOperationProto::Has,
            IntOperation::Equal => IntOperationProto::Equal,
            IntOperation::LessThan => IntOperationProto::LessThan,
            IntOperation::LessThanOrEqual => IntOperationProto::LessThanOrEqual,
            IntOperation::GreaterThan => IntOperationProto::GreaterThan,
            IntOperation::GreaterThanOrEqual => IntOperationProto::GreaterThanOrEqual,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntFilter {
    pub operation: IntOperation,
    pub value: i64,
    pub negated: bool,
}

impl TryFrom<IntFilterProto> for IntFilter {
    type Error = SerDeError;

    fn try_from(value_proto: IntFilterProto) -> Result<Self, Self::Error> {
        let operation = value_proto.operation().try_into()?;
        let value = value_proto.value; // todo: use Uid
        let negated = value_proto.negated;
        Ok(Self {
            operation,
            value,
            negated,
        })
    }
}

impl From<IntFilter> for IntFilterProto {
    fn from(value: IntFilter) -> IntFilterProto {
        IntFilterProto {
            operation: value.operation as i32,
            value: value.value,
            negated: value.negated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AndIntFilters {
    pub int_filters: Vec<IntFilter>,
}

impl TryFrom<AndIntFiltersProto> for AndIntFilters {
    type Error = SerDeError;
    fn try_from(value: AndIntFiltersProto) -> Result<Self, Self::Error> {
        let int_filters = value
            .int_filters
            .into_iter()
            .map(IntFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { int_filters })
    }
}

impl From<AndIntFilters> for AndIntFiltersProto {
    fn from(value: AndIntFilters) -> Self {
        Self {
            int_filters: value
                .int_filters
                .into_iter()
                .map(IntFilterProto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrIntFilters {
    pub and_int_filters: Vec<AndIntFilters>,
}

impl TryFrom<OrIntFiltersProto> for OrIntFilters {
    type Error = SerDeError;
    fn try_from(value: OrIntFiltersProto) -> Result<Self, Self::Error> {
        let and_int_filters = value
            .and_int_filters
            .into_iter()
            .map(AndIntFilters::try_from)
            .collect::<Result<_, SerDeError>>()?;
        Ok(Self { and_int_filters })
    }
}

impl From<OrIntFilters> for OrIntFiltersProto {
    fn from(value: OrIntFilters) -> Self {
        let and_int_filters = value
            .and_int_filters
            .into_iter()
            .map(AndIntFiltersProto::from)
            .collect();
        Self { and_int_filters }
    }
}

// Higher level helper
#[derive(Clone, Debug)]
pub enum StringCmp {
    Eq(String, bool),
    Contains(String, bool),
    Has,
}

impl StringCmp {
    pub fn eq(value: impl Into<String>, negated: bool) -> Self {
        StringCmp::Eq(value.into(), negated)
    }
}

impl From<StringFilter> for StringCmp {
    fn from(string_filter: StringFilter) -> StringCmp {
        match string_filter.operation {
            StringOperation::Has => StringCmp::Has,
            StringOperation::Equal => StringCmp::Eq(string_filter.value, string_filter.negated),
            StringOperation::Contains => {
                StringCmp::Contains(string_filter.value, string_filter.negated)
            }
            StringOperation::Regex => {
                unimplemented!()
            }
        }
    }
}

impl From<StringCmp> for StringFilter {
    fn from(string_cmp: StringCmp) -> StringFilter {
        match string_cmp {
            StringCmp::Has => StringFilter {
                operation: StringOperation::Has,
                value: "".to_string(),
                negated: false,
            },
            StringCmp::Eq(value, negated) => StringFilter {
                operation: StringOperation::Equal,
                value,
                negated,
            },
            StringCmp::Contains(value, negated) => StringFilter {
                operation: StringOperation::Contains,
                value,
                negated,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringOperation {
    Has,
    Equal,
    Contains,
    Regex,
}

impl TryFrom<StringOperationProto> for StringOperation {
    type Error = SerDeError;
    fn try_from(value_proto: StringOperationProto) -> Result<Self, Self::Error> {
        match value_proto {
            StringOperationProto::UnknownOperation => {
                Err(SerDeError::UnknownVariant("StringOperation"))
            }
            StringOperationProto::Has => Ok(Self::Has),
            StringOperationProto::Equal => Ok(Self::Equal),
            StringOperationProto::Contains => Ok(Self::Contains),
            StringOperationProto::Regex => Ok(Self::Regex),
        }
    }
}

impl From<StringOperation> for StringOperationProto {
    fn from(value: StringOperation) -> Self {
        match value {
            StringOperation::Has => Self::Has,
            StringOperation::Equal => Self::Equal,
            StringOperation::Contains => Self::Contains,
            StringOperation::Regex => Self::Regex,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringFilter {
    pub operation: StringOperation,
    pub value: String,
    pub negated: bool,
}

impl TryFrom<StringFilterProto> for StringFilter {
    type Error = SerDeError;

    fn try_from(value_proto: StringFilterProto) -> Result<Self, Self::Error> {
        let operation = value_proto.operation().try_into()?;
        let value = value_proto.value; // todo: use Uid
        let negated = value_proto.negated;
        Ok(Self {
            operation,
            value,
            negated,
        })
    }
}

impl From<StringFilter> for StringFilterProto {
    fn from(value: StringFilter) -> StringFilterProto {
        StringFilterProto {
            operation: value.operation as i32,
            value: value.value,
            negated: value.negated,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AndStringFilters {
    pub string_filters: Vec<StringFilter>,
}

impl TryFrom<AndStringFiltersProto> for AndStringFilters {
    type Error = SerDeError;
    fn try_from(value: AndStringFiltersProto) -> Result<Self, Self::Error> {
        let string_filters = value
            .string_filters
            .into_iter()
            .map(StringFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { string_filters })
    }
}

impl From<AndStringFilters> for AndStringFiltersProto {
    fn from(value: AndStringFilters) -> Self {
        Self {
            string_filters: value
                .string_filters
                .into_iter()
                .map(StringFilterProto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrStringFilters {
    pub and_string_filters: Vec<AndStringFilters>,
}

impl From<OrStringFilters> for Vec<Vec<StringCmp>> {
    fn from(or_string_filters: OrStringFilters) -> Vec<Vec<StringCmp>> {
        // "or filters" are collections of "and" filters
        let mut new_or_filters = Vec::with_capacity(or_string_filters.and_string_filters.len());
        for and_filters in or_string_filters.and_string_filters {
            let and_filters = and_filters.string_filters;
            let mut new_and_filters = Vec::with_capacity(and_filters.len());
            for string_cmp in and_filters {
                new_and_filters.push(string_cmp.into());
            }
            new_or_filters.push(new_and_filters);
        }

        new_or_filters
    }
}

impl TryFrom<OrStringFiltersProto> for OrStringFilters {
    type Error = SerDeError;
    fn try_from(value: OrStringFiltersProto) -> Result<Self, Self::Error> {
        let and_string_filters = value
            .and_string_filters
            .into_iter()
            .map(AndStringFilters::try_from)
            .collect::<Result<_, SerDeError>>()?;
        Ok(Self { and_string_filters })
    }
}

impl From<OrStringFilters> for OrStringFiltersProto {
    fn from(value: OrStringFilters) -> Self {
        let and_string_filters = value
            .and_string_filters
            .into_iter()
            .map(AndStringFiltersProto::from)
            .collect();
        Self { and_string_filters }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UidOperation {
    Equal,
}

impl TryFrom<UidOperationProto> for UidOperation {
    type Error = SerDeError;
    fn try_from(value_proto: UidOperationProto) -> Result<Self, Self::Error> {
        match value_proto {
            UidOperationProto::UnknownOperation => Err(SerDeError::UnknownVariant("UidOperation")),
            UidOperationProto::Equal => Ok(Self::Equal),
        }
    }
}

impl From<UidOperation> for UidOperationProto {
    fn from(value: UidOperation) -> Self {
        match value {
            UidOperation::Equal => Self::Equal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UidFilter {
    pub operation: UidOperation,
    pub value: i64,
}

impl TryFrom<UidFilterProto> for UidFilter {
    type Error = SerDeError;

    fn try_from(value_proto: UidFilterProto) -> Result<Self, Self::Error> {
        let operation = value_proto.operation().try_into()?;
        let value = value_proto.value; // todo: use Uid
        Ok(Self { operation, value })
    }
}

impl From<UidFilter> for UidFilterProto {
    fn from(value: UidFilter) -> UidFilterProto {
        UidFilterProto {
            operation: value.operation as i32,
            value: value.value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UidFilters {
    pub uid_filters: Vec<UidFilter>,
}

impl TryFrom<UidFiltersProto> for UidFilters {
    type Error = SerDeError;
    fn try_from(value: UidFiltersProto) -> Result<Self, Self::Error> {
        let uid_filters = value
            .uid_filters
            .into_iter()
            .map(UidFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { uid_filters })
    }
}

impl From<UidFilters> for UidFiltersProto {
    fn from(value: UidFilters) -> Self {
        Self {
            uid_filters: value
                .uid_filters
                .into_iter()
                .map(UidFilterProto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeQuery {
    pub node_type: NodeType,
    pub int_filters: HashMap<PropertyName, OrIntFilters>,
    pub string_filters: HashMap<PropertyName, OrStringFilters>,
    pub edge_filters: HashMap<EdgeName, EdgeFilters>,
    pub uid_filters: UidFilters,
}

impl TryFrom<NodeQueryProto> for NodeQuery {
    type Error = SerDeError;
    fn try_from(value: NodeQueryProto) -> Result<Self, Self::Error> {
        let node_type = value
            .node_type
            .ok_or(SerDeError::MissingField("node_type"))?
            .try_into()?;
        let int_filters = value
            .int_filters
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "int_filters",
                        assertion: e.to_owned(),
                    })?,
                    v.try_into()?,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let string_filters = value
            .string_filters
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "string_filters",
                        assertion: e.to_owned(),
                    })?,
                    v.try_into()?,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let edge_filters = value
            .edge_filters
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    EdgeName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "edge_filters",
                        assertion: e.to_owned(),
                    })?,
                    v.try_into()?,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let uid_filters = value
            .uid_filters
            .ok_or(SerDeError::MissingField("uid_filters"))?
            .try_into()?;

        Ok(Self {
            node_type,
            int_filters,
            string_filters,
            edge_filters,
            uid_filters,
        })
    }
}

impl From<NodeQuery> for NodeQueryProto {
    fn from(value: NodeQuery) -> Self {
        let node_type = Some(value.node_type.into());
        let int_filters = value
            .int_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let string_filters = value
            .string_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let uid_filters = value.uid_filters.into();

        let edge_filters = value
            .edge_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        Self {
            node_type,
            int_filters,
            string_filters,
            uid_filters: Some(uid_filters),
            edge_filters,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeFilters {
    pub node_queries: Vec<NodeQuery>,
}

impl TryFrom<EdgeFiltersProto> for EdgeFilters {
    type Error = SerDeError;

    fn try_from(value: EdgeFiltersProto) -> Result<Self, Self::Error> {
        Ok(Self {
            node_queries: value
                .node_queries
                .into_iter()
                .map(NodeQuery::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<EdgeFilters> for EdgeFiltersProto {
    fn from(value: EdgeFilters) -> Self {
        Self {
            node_queries: value
                .node_queries
                .into_iter()
                .map(NodeQueryProto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeView {
    pub uid: Uid,
    pub node_type: NodeType,
    pub string_properties: HashMap<PropertyName, String>,
    pub int_properties: HashMap<PropertyName, i64>,
}

impl TryFrom<NodeViewProto> for NodeView {
    type Error = SerDeError;
    fn try_from(value: NodeViewProto) -> Result<Self, Self::Error> {
        let string_properties = value
            .string_properties
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "string_properties",
                        assertion: e.to_owned(),
                    })?,
                    v,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let int_properties = value
            .int_properties
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "string_properties",
                        assertion: e.to_owned(),
                    })?,
                    v,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        Ok(Self {
            uid: value
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?,
            node_type: value
                .node_type
                .ok_or(SerDeError::MissingField("node_type"))?
                .try_into()?,
            string_properties,
            int_properties,
        })
    }
}

impl From<NodeView> for NodeViewProto {
    fn from(value: NodeView) -> Self {
        let string_properties = value
            .string_properties
            .into_iter()
            .map(|(k, v)| (k.value, v))
            .collect();

        let int_properties = value
            .int_properties
            .into_iter()
            .map(|(k, v)| (k.value, v))
            .collect();

        Self {
            uid: Some(value.uid.into()),
            node_type: Some(value.node_type.into()),
            string_properties,
            int_properties,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeView {
    pub edge_name: EdgeName,
    pub source_uid: Uid,
    pub destination_uid: Uid,
}

impl TryFrom<EdgeViewProto> for EdgeView {
    type Error = SerDeError;
    fn try_from(value: EdgeViewProto) -> Result<Self, Self::Error> {
        Ok(Self {
            edge_name: value
                .edge_name
                .ok_or(SerDeError::MissingField("edge_name"))?
                .try_into()?,
            source_uid: value
                .source_uid
                .ok_or(SerDeError::MissingField("source_uid"))?
                .try_into()?,
            destination_uid: value
                .destination_uid
                .ok_or(SerDeError::MissingField("destination_uid"))?
                .try_into()?,
        })
    }
}

impl From<EdgeView> for EdgeViewProto {
    fn from(value: EdgeView) -> Self {
        Self {
            edge_name: Some(value.edge_name.into()),
            source_uid: Some(value.source_uid.into()),
            destination_uid: Some(value.destination_uid.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeViews {
    pub edge_view_list: Vec<EdgeView>,
}

impl TryFrom<EdgeViewsProto> for EdgeViews {
    type Error = SerDeError;
    fn try_from(value: EdgeViewsProto) -> Result<Self, Self::Error> {
        Ok(Self {
            edge_view_list: value
                .edge_view_list
                .into_iter()
                .map(EdgeView::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<EdgeViews> for EdgeViewsProto {
    fn from(value: EdgeViews) -> Self {
        Self {
            edge_view_list: value
                .edge_view_list
                .into_iter()
                .map(EdgeViewProto::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphView {
    pub nodes: Vec<NodeView>,
    pub edges: HashMap<String, EdgeViews>,
}

impl TryFrom<GraphViewProto> for GraphView {
    type Error = SerDeError;
    fn try_from(value: GraphViewProto) -> Result<Self, Self::Error> {
        let nodes = value
            .nodes
            .into_iter()
            .map(NodeView::try_from)
            .collect::<Result<_, _>>()?;

        let edges = value
            .edges
            .into_iter()
            .map(|(k, v)| v.try_into().map(|v| (k, v)))
            .collect::<Result<_, _>>()?;

        Ok(Self { nodes, edges })
    }
}

impl From<GraphView> for GraphViewProto {
    fn from(value: GraphView) -> Self {
        let nodes = value.nodes.into_iter().map(NodeViewProto::from).collect();

        let edges = value
            .edges
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();

        Self { nodes, edges }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphWithNodeRequest {
    pub tenant_id: uuid::Uuid,
    pub node_uid: Uid,
    pub node_query: NodeQuery,
    pub edge_mapping: HashMap<EdgeName, EdgeName>,
}

impl TryFrom<QueryGraphWithNodeRequestProto> for QueryGraphWithNodeRequest {
    type Error = SerDeError;

    fn try_from(value: QueryGraphWithNodeRequestProto) -> Result<Self, Self::Error> {
        let edge_mapping = value
            .edge_mapping
            .into_iter()
            .map(|(k, v)| Ok((k.try_into()?, v.try_into()?)))
            .collect::<Result<_, &str>>()
            .map_err(|e| SerDeError::InvalidField {
                field_name: "edge_mapping",
                assertion: e.to_string(),
            })?;

        Ok(Self {
            tenant_id: value
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            node_uid: value
                .node_uid
                .ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            node_query: value
                .node_query
                .ok_or(SerDeError::MissingField("node_query"))?
                .try_into()?,
            edge_mapping,
        })
    }
}

impl From<QueryGraphWithNodeRequest> for QueryGraphWithNodeRequestProto {
    fn from(value: QueryGraphWithNodeRequest) -> Self {
        let edge_mapping = value
            .edge_mapping
            .into_iter()
            .map(|(k, v)| (k.value, v.value))
            .collect();

        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_uid: Some(value.node_uid.into()),
            node_query: Some(value.node_query.into()),
            edge_mapping,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphWithNodeResponse {
    pub matched_graph: GraphView,
    pub root_uid: Uid,
}

impl TryFrom<QueryGraphWithNodeResponseProto> for QueryGraphWithNodeResponse {
    type Error = SerDeError;
    fn try_from(value: QueryGraphWithNodeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            matched_graph: value
                .matched_graph
                .ok_or(SerDeError::MissingField("matched_graph"))?
                .try_into()?,
            root_uid: value
                .root_uid
                .ok_or(SerDeError::MissingField("root_uid"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphWithNodeResponse> for QueryGraphWithNodeResponseProto {
    fn from(value: QueryGraphWithNodeResponse) -> Self {
        Self {
            matched_graph: Some(value.matched_graph.into()),
            root_uid: Some(value.root_uid.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromNodeRequest {
    pub tenant_id: uuid::Uuid,
    pub node_uid: Uid,
    pub node_query: NodeQuery,
    pub edge_mapping: HashMap<EdgeName, EdgeName>,
}

impl TryFrom<QueryGraphFromNodeRequestProto> for QueryGraphFromNodeRequest {
    type Error = SerDeError;

    fn try_from(value: QueryGraphFromNodeRequestProto) -> Result<Self, Self::Error> {
        let edge_mapping = value
            .edge_mapping
            .into_iter()
            .map(|(k, v)| Ok((k.try_into()?, v.try_into()?)))
            .collect::<Result<_, &str>>()
            .map_err(|e| SerDeError::InvalidField {
                field_name: "edge_mapping",
                assertion: e.to_string(),
            })?;

        Ok(Self {
            tenant_id: value
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            node_uid: value
                .node_uid
                .ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            node_query: value
                .node_query
                .ok_or(SerDeError::MissingField("node_query"))?
                .try_into()?,
            edge_mapping,
        })
    }
}

impl From<QueryGraphFromNodeRequest> for QueryGraphFromNodeRequestProto {
    fn from(value: QueryGraphFromNodeRequest) -> Self {
        let edge_mapping = value
            .edge_mapping
            .into_iter()
            .map(|(k, v)| (k.value, v.value))
            .collect();

        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_uid: Some(value.node_uid.into()),
            node_query: Some(value.node_query.into()),
            edge_mapping,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromNodeResponse {
    pub matched_graph: GraphView,
    pub root_uid: Uid,
}

impl TryFrom<QueryGraphFromNodeResponseProto> for QueryGraphFromNodeResponse {
    type Error = SerDeError;
    fn try_from(value: QueryGraphFromNodeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            matched_graph: value
                .matched_graph
                .ok_or(SerDeError::MissingField("matched_graph"))?
                .try_into()?,
            root_uid: value
                .root_uid
                .ok_or(SerDeError::MissingField("root_uid"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphFromNodeResponse> for QueryGraphFromNodeResponseProto {
    fn from(value: QueryGraphFromNodeResponse) -> Self {
        Self {
            matched_graph: Some(value.matched_graph.into()),
            root_uid: Some(value.root_uid.into()),
        }
    }
}
