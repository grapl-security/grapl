use std::collections::hash_map::Entry;

use rustc_hash::{
    FxHashMap,
    FxHashSet,
};

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
    protobufs::graplinc::grapl::api::graph_query_service::v1beta1::{
        int_filter::Operation as IntOperationProto,
        integer_property as integer_property_proto,
        string_filter::Operation as StringOperationProto,
        uid_filter::Operation as UidOperationProto,
        AndIntFilters as AndIntFiltersProto,
        AndStringFilters as AndStringFiltersProto,
        EdgeEntry as EdgeEntryProto,
        EdgeMap as EdgeMapProto,
        EdgeQueryEntry as EdgeQueryEntryProto,
        EdgeQueryMap as EdgeQueryMapProto,
        EdgeViewEntry as EdgeViewEntryProto,
        EdgeViewMap as EdgeViewMapProto,
        GraphQuery as GraphQueryProto,
        GraphView as GraphViewProto,
        IntFilter as IntFilterProto,
        IntegerProperty as IntegerPropertyProto,
        NodePropertiesView as NodePropertiesViewProto,
        NodePropertiesViewEntry as NodePropertiesViewEntryProto,
        NodePropertiesViewMap as NodePropertiesViewMapProto,
        NodePropertyQuery as NodePropertyQueryProto,
        NodePropertyQueryEntry as NodePropertyQueryEntryProto,
        NodePropertyQueryMap as NodePropertyQueryMapProto,
        OrIntFilters as OrIntFiltersProto,
        OrStringFilters as OrStringFiltersProto,
        QueryGraphFromNodeRequest as QueryGraphFromNodeRequestProto,
        QueryGraphFromNodeResponse as QueryGraphFromNodeResponseProto,
        QueryGraphWithNodeRequest as QueryGraphWithNodeRequestProto,
        QueryGraphWithNodeResponse as QueryGraphWithNodeResponseProto,
        QueryId as QueryIdProto,
        StringFilter as StringFilterProto,
        StringProperties as StringPropertiesProto,
        StringProperty as StringPropertyProto,
        UidFilter as UidFilterProto,
        UidFilters as UidFiltersProto,
    },
    SerDeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryId {
    pub value: u64,
}

impl QueryId {
    pub fn new() -> Self {
        QueryId {
            value: rand::random::<u64>() | 1,
        }
    }
}

impl TryFrom<QueryIdProto> for QueryId {
    type Error = SerDeError;
    fn try_from(value: QueryIdProto) -> Result<Self, Self::Error> {
        Ok(Self { value: value.value })
    }
}

impl From<QueryId> for QueryIdProto {
    fn from(value: QueryId) -> Self {
        Self { value: value.value }
    }
}

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
        let value = value_proto.value;
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
pub enum StrCmp<'a> {
    Eq(&'a str, bool),
    Contains(&'a str, bool),
    Has,
}

impl<'a> StrCmp<'a> {
    pub fn eq(value: &'a str, negated: bool) -> Self {
        StrCmp::Eq(value, negated)
    }
}

impl<'a> From<&'a StringFilter> for StrCmp<'a> {
    fn from(string_filter: &'a StringFilter) -> StrCmp<'a> {
        match string_filter.operation {
            StringOperation::Has => StrCmp::Has,
            StringOperation::Equal => {
                StrCmp::Eq(string_filter.value.as_str(), string_filter.negated)
            }
            StringOperation::Contains => {
                StrCmp::Contains(string_filter.value.as_str(), string_filter.negated)
            }
            StringOperation::Regex => {
                // todo: We don't currently support Regex, but we should
                unimplemented!()
            }
        }
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
                // todo: We don't currently support Regex, but we should
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
        let value = value_proto.value;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl OrStringFilters {
    pub fn new() -> Self {
        Self {
            and_string_filters: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            and_string_filters: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, filters: AndStringFilters) {
        self.and_string_filters.push(filters);
    }
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
    pub value: Uid,
}

impl TryFrom<UidFilterProto> for UidFilter {
    type Error = SerDeError;

    fn try_from(value_proto: UidFilterProto) -> Result<Self, Self::Error> {
        let operation = value_proto.operation().try_into()?;
        let value = value_proto
            .value
            .ok_or(SerDeError::MissingField("uid"))?
            .try_into()?;
        Ok(Self { operation, value })
    }
}

impl From<UidFilter> for UidFilterProto {
    fn from(value: UidFilter) -> UidFilterProto {
        UidFilterProto {
            operation: value.operation as i32,
            value: Some(value.value.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
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
pub struct NodePropertyQuery {
    pub query_id: QueryId,
    pub node_type: NodeType,
    pub int_filters: FxHashMap<PropertyName, OrIntFilters>,
    pub string_filters: FxHashMap<PropertyName, OrStringFilters>,
    pub uid_filters: UidFilters,
}

impl NodePropertyQuery {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            query_id: QueryId::new(),
            int_filters: Default::default(),
            string_filters: Default::default(),
            uid_filters: Default::default(),
        }
    }

    pub fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.query_id, other.query_id);
        debug_assert_eq!(self.node_type, other.node_type);
        self.string_filters.extend(other.string_filters);
    }

    pub fn with_string_filters(
        &mut self,
        property_name: PropertyName,
        filters: impl Into<AndStringFilters>,
    ) -> &mut Self {
        let filters = filters.into();
        self.string_filters
            .entry(property_name)
            .or_insert_with(|| OrStringFilters::with_capacity(1))
            .push(filters);
        self
    }
}

impl TryFrom<NodePropertyQueryProto> for NodePropertyQuery {
    type Error = SerDeError;
    fn try_from(value: NodePropertyQueryProto) -> Result<Self, Self::Error> {
        let node_type = value
            .node_type
            .ok_or(SerDeError::MissingField("node_type"))?
            .try_into()?;

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

        let uid_filters = value
            .uid_filters
            .ok_or(SerDeError::MissingField("uid_filters"))?
            .try_into()?;

        let query_id = value
            .query_id
            .ok_or(SerDeError::MissingField("query_id"))?
            .try_into()?;
        Ok(Self {
            query_id,
            node_type,
            int_filters,
            string_filters,
            uid_filters,
        })
    }
}

impl From<NodePropertyQuery> for NodePropertyQueryProto {
    fn from(value: NodePropertyQuery) -> Self {
        let query_id = Some(value.query_id.into());
        let node_type = Some(value.node_type.into());
        let string_filters = value
            .string_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let int_filters = value
            .int_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let uid_filters = value.uid_filters.into();

        Self {
            query_id,
            node_type,
            int_filters,
            string_filters,
            uid_filters: Some(uid_filters),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphQuery {
    pub root_query_id: QueryId,
    pub node_property_queries: FxHashMap<QueryId, NodePropertyQuery>,
    pub edge_filters: FxHashMap<(QueryId, EdgeName), FxHashSet<QueryId>>,
    pub edge_map: FxHashMap<EdgeName, EdgeName>,
}

impl GraphQuery {
    pub fn add_node(&mut self, query_id: QueryId, node_type: NodeType) {
        self.node_property_queries.insert(
            query_id,
            NodePropertyQuery {
                query_id,
                node_type,
                int_filters: Default::default(),
                string_filters: Default::default(),
                uid_filters: Default::default(),
            },
        );
    }

    pub fn merge_node(&mut self, node: NodePropertyQuery) {
        match self.node_property_queries.entry(node.query_id) {
            Entry::Occupied(n) => {
                n.into_mut().merge(node);
            }
            Entry::Vacant(n) => {
                n.insert(node);
            }
        }
    }
}

impl TryFrom<GraphQueryProto> for GraphQuery {
    type Error = SerDeError;
    fn try_from(value: GraphQueryProto) -> Result<Self, Self::Error> {
        let root_query_id: QueryId = value
            .root_query_id
            .ok_or_else(|| SerDeError::MissingField("root_query_id"))?
            .try_into()?;

        let node_property_queries_proto = value
            .node_property_queries
            .ok_or_else(|| SerDeError::MissingField("node_property_queries"))?;

        let mut node_property_queries: FxHashMap<QueryId, NodePropertyQuery> = FxHashMap::default();
        node_property_queries.reserve(node_property_queries_proto.entries.len());

        for node_property_query in node_property_queries_proto.entries {
            let query_id = node_property_query
                .query_id
                .ok_or_else(|| SerDeError::MissingField("query_id"))?
                .try_into()?;
            let node_property_query = node_property_query
                .node_property_query
                .ok_or_else(|| SerDeError::MissingField("node_property_query"))?
                .try_into()?;
            node_property_queries.insert(query_id, node_property_query);
        }

        let edge_filters_proto = value
            .edge_filters
            .ok_or_else(|| SerDeError::MissingField("edge_filters"))?;

        let mut edge_filters: FxHashMap<(QueryId, EdgeName), FxHashSet<QueryId>> =
            FxHashMap::default();
        edge_filters.reserve(edge_filters_proto.entries.len());

        for edge_filter in edge_filters_proto.entries {
            let query_id = edge_filter
                .query_id
                .ok_or_else(|| SerDeError::MissingField("query_id"))?
                .try_into()?;
            let edge_name = edge_filter
                .edge_name
                .ok_or_else(|| SerDeError::MissingField("edge_name"))?
                .try_into()?;
            let neighbor_query_ids = edge_filter
                .neighbor_query_ids
                .into_iter()
                .map(|query_id| query_id.try_into())
                .collect::<Result<FxHashSet<_>, _>>()?;
            edge_filters.insert((query_id, edge_name), neighbor_query_ids);
        }

        let edge_map_proto = value
            .edge_map
            .ok_or_else(|| SerDeError::MissingField("edge_map"))?;
        let mut edge_map: FxHashMap<EdgeName, EdgeName> = FxHashMap::default();
        edge_map.reserve(edge_map_proto.entries.len());

        for edge_entry in edge_map_proto.entries {
            let forward_edge_name = edge_entry
                .forward_edge_name
                .ok_or_else(|| SerDeError::MissingField("forward_edge_name"))?
                .try_into()?;
            let reverse_edge_name = edge_entry
                .reverse_edge_name
                .ok_or_else(|| SerDeError::MissingField("reverse_edge_name"))?
                .try_into()?;
            edge_map.insert(forward_edge_name, reverse_edge_name);
        }

        Ok(Self {
            root_query_id,
            node_property_queries,
            edge_filters,
            edge_map,
        })
    }
}

impl From<GraphQuery> for GraphQueryProto {
    fn from(value: GraphQuery) -> Self {
        let root_query_id = Some(value.root_query_id.into());
        let node_property_queries = Some(NodePropertyQueryMapProto {
            entries: value
                .node_property_queries
                .into_iter()
                .map(|(k, v)| NodePropertyQueryEntryProto {
                    query_id: Some(k.into()),
                    node_property_query: Some(v.into()),
                })
                .collect(),
        });
        let edge_filters = Some(EdgeQueryMapProto {
            entries: value
                .edge_filters
                .into_iter()
                .map(|((k0, k1), v)| EdgeQueryEntryProto {
                    query_id: Some(k0.into()),
                    edge_name: Some(k1.into()),
                    neighbor_query_ids: v.into_iter().map(QueryIdProto::from).collect(),
                })
                .collect(),
        });

        let edge_map = Some(EdgeMapProto {
            entries: value
                .edge_map
                .into_iter()
                .map(|(k, v)| EdgeEntryProto {
                    forward_edge_name: Some(k.into()),
                    reverse_edge_name: Some(v.into()),
                })
                .collect(),
        });
        Self {
            root_query_id,
            node_property_queries,
            edge_filters,
            edge_map,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertiesView {
    pub uid: Uid,
    pub node_type: NodeType,
    pub string_properties: FxHashMap<PropertyName, String>,
}

impl NodePropertiesView {
    pub fn new(
        uid: Uid,
        node_type: NodeType,
        string_properties: FxHashMap<PropertyName, String>,
    ) -> Self {
        Self {
            uid,
            node_type,
            string_properties,
        }
    }

    pub fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.uid, other.uid);
        debug_assert_eq!(self.node_type, other.node_type);
        self.string_properties.extend(other.string_properties);
    }

    pub fn add_string_property(&mut self, property_name: PropertyName, value: String) {
        self.string_properties.insert(property_name, value);
    }
}

impl TryFrom<NodePropertiesViewProto> for NodePropertiesView {
    type Error = SerDeError;
    fn try_from(value: NodePropertiesViewProto) -> Result<Self, Self::Error> {
        let proto_string_properties = value
            .string_properties
            .ok_or_else(|| SerDeError::MissingField("string_properties"))?;

        let mut string_properties = FxHashMap::default();
        string_properties.reserve(proto_string_properties.properties.len());

        for string_property in proto_string_properties.properties {
            let property_name = string_property
                .property_name
                .ok_or_else(|| SerDeError::MissingField("property_name"))?;
            string_properties.insert(property_name.try_into()?, string_property.property_value);
        }

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
        })
    }
}

impl From<NodePropertiesView> for NodePropertiesViewProto {
    fn from(value: NodePropertiesView) -> Self {
        let string_properties: Vec<StringPropertyProto> = value
            .string_properties
            .into_iter()
            .map(|(k, v)| StringPropertyProto {
                property_name: Some(k.into()),
                property_value: v,
            })
            .collect();
        let string_properties = Some(StringPropertiesProto {
            properties: string_properties,
        });

        Self {
            uid: Some(value.uid.into()),
            node_type: Some(value.node_type.into()),
            string_properties,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertiesViewEntry {
    pub uid: Uid,
    pub node_view: NodePropertiesView,
}

impl TryFrom<NodePropertiesViewEntryProto> for NodePropertiesViewEntry {
    type Error = SerDeError;
    fn try_from(value: NodePropertiesViewEntryProto) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: value
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?,
            node_view: value
                .node_view
                .ok_or(SerDeError::MissingField("node_view"))?
                .try_into()?,
        })
    }
}

impl From<NodePropertiesViewEntry> for NodePropertiesViewEntryProto {
    fn from(value: NodePropertiesViewEntry) -> Self {
        NodePropertiesViewEntryProto {
            uid: Some(value.uid.into()),
            node_view: Some(value.node_view.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertiesViewMap {
    pub entries: FxHashMap<Uid, NodePropertiesView>,
}

impl TryFrom<NodePropertiesViewMapProto> for NodePropertiesViewMap {
    type Error = SerDeError;
    fn try_from(value: NodePropertiesViewMapProto) -> Result<Self, Self::Error> {
        let mut entries = FxHashMap::default();
        entries.reserve(value.entries.len());

        for entry in value.entries.into_iter() {
            let uid = entry
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?;
            let node_view = entry
                .node_view
                .ok_or(SerDeError::MissingField("node_view"))?
                .try_into()?;
            entries.insert(uid, node_view);
        }

        Ok(Self { entries })
    }
}

impl From<NodePropertiesViewMap> for NodePropertiesViewMapProto {
    fn from(value: NodePropertiesViewMap) -> Self {
        let mut entries = Vec::with_capacity(value.entries.len());

        for (uid, node_view) in value.entries.into_iter() {
            entries.push(NodePropertiesViewEntryProto {
                uid: Some(uid.into()),
                node_view: Some(node_view.into()),
            });
        }

        Self { entries }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeViewEntry {
    pub uid: Uid,
    pub edge_name: EdgeName,
    pub neighbors: FxHashSet<Uid>,
}

impl TryFrom<EdgeViewEntryProto> for EdgeViewEntry {
    type Error = SerDeError;
    fn try_from(value: EdgeViewEntryProto) -> Result<Self, Self::Error> {
        let mut neighbors = FxHashSet::default();
        neighbors.reserve(value.neighbors.len());
        for neighbor in value.neighbors.into_iter() {
            neighbors.insert(neighbor.try_into()?);
        }
        Ok(Self {
            uid: value
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?,
            edge_name: value
                .edge_name
                .ok_or(SerDeError::MissingField("edge_name"))?
                .try_into()?,
            neighbors,
        })
    }
}

impl From<EdgeViewEntry> for EdgeViewEntryProto {
    fn from(value: EdgeViewEntry) -> Self {
        let mut neighbors = Vec::with_capacity(value.neighbors.len());
        for neighbor in value.neighbors.into_iter() {
            neighbors.push(neighbor.into());
        }
        EdgeViewEntryProto {
            uid: Some(value.uid.into()),
            edge_name: Some(value.edge_name.into()),
            neighbors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeViewMap {
    pub entries: FxHashMap<(Uid, EdgeName), FxHashSet<Uid>>,
}

impl TryFrom<EdgeViewMapProto> for EdgeViewMap {
    type Error = SerDeError;
    fn try_from(value: EdgeViewMapProto) -> Result<Self, Self::Error> {
        let mut entries = FxHashMap::default();
        entries.reserve(value.entries.len());

        for entry in value.entries.into_iter() {
            let entry: EdgeViewEntry = entry.try_into()?;
            entries.insert((entry.uid, entry.edge_name), entry.neighbors);
        }

        Ok(Self { entries })
    }
}

impl From<EdgeViewMap> for EdgeViewMapProto {
    fn from(value: EdgeViewMap) -> Self {
        let mut entries = Vec::with_capacity(value.entries.len());

        for ((uid, edge_name), neighbors) in value.entries.into_iter() {
            entries.push(
                EdgeViewEntry {
                    uid,
                    edge_name,
                    neighbors,
                }
                .into(),
            );
        }

        Self { entries }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GraphView {
    pub nodes: FxHashMap<Uid, NodePropertiesView>,
    pub edges: FxHashMap<(Uid, EdgeName), FxHashSet<Uid>>,
}

impl GraphView {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn new_node(&mut self, uid: Uid, node_type: NodeType) -> &mut NodePropertiesView {
        self.nodes
            .entry(uid)
            .or_insert_with(|| NodePropertiesView::new(uid, node_type, Default::default()))
    }

    pub fn add_node(&mut self, node: NodePropertiesView) {
        match self.nodes.entry(node.uid) {
            Entry::Occupied(n) => {
                n.into_mut().merge(node);
            }
            Entry::Vacant(n) => {
                n.insert(node);
            }
        }
    }

    pub fn add_edge(&mut self, from: Uid, edge_name: EdgeName, to: Uid) {
        self.edges
            .entry((from, edge_name))
            .or_insert_with(FxHashSet::default)
            .insert(to);
    }

    pub fn add_edges(&mut self, src_uid: Uid, edge_name: EdgeName, dst_uids: FxHashSet<Uid>) {
        self.edges
            .entry((src_uid, edge_name))
            .or_insert_with(FxHashSet::default)
            .extend(dst_uids);
    }

    pub fn get_node(&self, uid: Uid) -> Option<&NodePropertiesView> {
        self.nodes.get(&uid)
    }

    pub fn get_edges(&self, from: Uid) -> impl Iterator<Item = (&EdgeName, &FxHashSet<Uid>)> {
        self.edges
            .iter()
            .filter(move |(key, _)| key.0 == from)
            .map(|(key, value)| (&key.1, value))
    }

    pub fn merge(&mut self, other: Self) {
        for (_, node) in other.nodes.into_iter() {
            self.add_node(node);
        }

        for ((src_uid, edge_name), dst_uids) in other.edges.into_iter() {
            self.add_edges(src_uid.clone(), edge_name.clone(), dst_uids.clone());
        }
    }

    pub fn get_nodes(&self) -> &FxHashMap<Uid, NodePropertiesView> {
        &self.nodes
    }
}

impl TryFrom<GraphViewProto> for GraphView {
    type Error = SerDeError;
    fn try_from(value: GraphViewProto) -> Result<Self, Self::Error> {
        let nodes: NodePropertiesViewMap = value
            .nodes
            .ok_or(SerDeError::MissingField("nodes"))?
            .try_into()?;
        let edges: EdgeViewMap = value
            .edges
            .ok_or(SerDeError::MissingField("edges"))?
            .try_into()?;

        Ok(Self {
            nodes: nodes.entries,
            edges: edges.entries,
        })
    }
}

impl From<GraphView> for GraphViewProto {
    fn from(value: GraphView) -> Self {
        let nodes = NodePropertiesViewMap {
            entries: value.nodes,
        }
        .into();
        let edges = EdgeViewMap {
            entries: value.edges,
        }
        .into();
        Self {
            nodes: Some(nodes),
            edges: Some(edges),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphWithNodeRequest {
    pub tenant_id: uuid::Uuid,
    pub node_uid: Uid,
    pub graph_query: GraphQuery,
}

impl TryFrom<QueryGraphWithNodeRequestProto> for QueryGraphWithNodeRequest {
    type Error = SerDeError;

    fn try_from(value: QueryGraphWithNodeRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            node_uid: value
                .node_uid
                .ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            graph_query: value
                .graph_query
                .ok_or(SerDeError::MissingField("node_query"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphWithNodeRequest> for QueryGraphWithNodeRequestProto {
    fn from(value: QueryGraphWithNodeRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_uid: Some(value.node_uid.into()),
            graph_query: Some(value.graph_query.into()),
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
    pub graph_query: GraphQuery,
}

impl TryFrom<QueryGraphFromNodeRequestProto> for QueryGraphFromNodeRequest {
    type Error = SerDeError;

    fn try_from(value: QueryGraphFromNodeRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            node_uid: value
                .node_uid
                .ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            graph_query: value
                .graph_query
                .ok_or(SerDeError::MissingField("graph_query"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphFromNodeRequest> for QueryGraphFromNodeRequestProto {
    fn from(value: QueryGraphFromNodeRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_uid: Some(value.node_uid.into()),
            graph_query: Some(value.graph_query.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromNodeResponse {
    pub matched_graph: GraphView,
}

impl TryFrom<QueryGraphFromNodeResponseProto> for QueryGraphFromNodeResponse {
    type Error = SerDeError;
    fn try_from(value: QueryGraphFromNodeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            matched_graph: value
                .matched_graph
                .ok_or(SerDeError::MissingField("matched_graph"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphFromNodeResponse> for QueryGraphFromNodeResponseProto {
    fn from(value: QueryGraphFromNodeResponse) -> Self {
        Self {
            matched_graph: Some(value.matched_graph.into()),
        }
    }
}
