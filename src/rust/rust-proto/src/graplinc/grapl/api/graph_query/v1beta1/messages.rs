use std::collections::hash_map::Entry;

use rustc_hash::{
    FxHashMap,
    FxHashSet,
};

use crate::{
    graplinc::grapl::common::v1beta1::types::{
        EdgeName,
        NodeType,
        PropertyName,
        Uid,
    },
    protobufs::graplinc::grapl::api::graph_query::v1beta1 as proto,
    SerDeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryId {
    pub value: u64,
}

impl Default for QueryId {
    fn default() -> Self {
        Self {
            value: rand::random::<u64>() | 1,
        }
    }
}

impl TryFrom<proto::QueryId> for QueryId {
    type Error = SerDeError;
    fn try_from(value: proto::QueryId) -> Result<Self, Self::Error> {
        Ok(Self { value: value.value })
    }
}

impl From<QueryId> for proto::QueryId {
    fn from(value: QueryId) -> Self {
        Self { value: value.value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntOperation {
    Has,
    Equal,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl TryFrom<proto::int_filter::Operation> for IntOperation {
    type Error = SerDeError;
    fn try_from(value_proto: proto::int_filter::Operation) -> Result<Self, Self::Error> {
        match value_proto {
            proto::int_filter::Operation::Unspecified => {
                Err(SerDeError::UnknownVariant("IntOperation"))
            }
            proto::int_filter::Operation::Has => Ok(Self::Has),
            proto::int_filter::Operation::Equal => Ok(Self::Equal),
            proto::int_filter::Operation::LessThan => Ok(Self::LessThan),
            proto::int_filter::Operation::LessThanOrEqual => Ok(Self::LessThanOrEqual),
            proto::int_filter::Operation::GreaterThan => Ok(Self::GreaterThan),
            proto::int_filter::Operation::GreaterThanOrEqual => Ok(Self::GreaterThanOrEqual),
        }
    }
}

impl From<IntOperation> for proto::int_filter::Operation {
    fn from(value: IntOperation) -> Self {
        match value {
            IntOperation::Has => proto::int_filter::Operation::Has,
            IntOperation::Equal => proto::int_filter::Operation::Equal,
            IntOperation::LessThan => proto::int_filter::Operation::LessThan,
            IntOperation::LessThanOrEqual => proto::int_filter::Operation::LessThanOrEqual,
            IntOperation::GreaterThan => proto::int_filter::Operation::GreaterThan,
            IntOperation::GreaterThanOrEqual => proto::int_filter::Operation::GreaterThanOrEqual,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntFilter {
    pub operation: IntOperation,
    pub value: i64,
    pub negated: bool,
}

impl TryFrom<proto::IntFilter> for IntFilter {
    type Error = SerDeError;

    fn try_from(value_proto: proto::IntFilter) -> Result<Self, Self::Error> {
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

impl From<IntFilter> for proto::IntFilter {
    fn from(value: IntFilter) -> proto::IntFilter {
        proto::IntFilter {
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

impl TryFrom<proto::AndIntFilters> for AndIntFilters {
    type Error = SerDeError;
    fn try_from(value: proto::AndIntFilters) -> Result<Self, Self::Error> {
        let int_filters = value
            .int_filters
            .into_iter()
            .map(IntFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { int_filters })
    }
}

impl From<AndIntFilters> for proto::AndIntFilters {
    fn from(value: AndIntFilters) -> Self {
        Self {
            int_filters: value
                .int_filters
                .into_iter()
                .map(proto::IntFilter::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrIntFilters {
    pub and_int_filters: Vec<AndIntFilters>,
}

impl OrIntFilters {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            and_int_filters: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, and_int_filters: AndIntFilters) {
        self.and_int_filters.push(and_int_filters);
    }
}

impl TryFrom<proto::OrIntFilters> for OrIntFilters {
    type Error = SerDeError;
    fn try_from(value: proto::OrIntFilters) -> Result<Self, Self::Error> {
        let and_int_filters = value
            .and_int_filters
            .into_iter()
            .map(AndIntFilters::try_from)
            .collect::<Result<_, SerDeError>>()?;
        Ok(Self { and_int_filters })
    }
}

impl From<OrIntFilters> for proto::OrIntFilters {
    fn from(value: OrIntFilters) -> Self {
        let and_int_filters = value
            .and_int_filters
            .into_iter()
            .map(proto::AndIntFilters::from)
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

impl TryFrom<proto::string_filter::Operation> for StringOperation {
    type Error = SerDeError;
    fn try_from(value_proto: proto::string_filter::Operation) -> Result<Self, Self::Error> {
        match value_proto {
            proto::string_filter::Operation::Unspecified => {
                Err(SerDeError::UnknownVariant("StringOperation"))
            }
            proto::string_filter::Operation::Has => Ok(Self::Has),
            proto::string_filter::Operation::Equal => Ok(Self::Equal),
            proto::string_filter::Operation::Contains => Ok(Self::Contains),
            proto::string_filter::Operation::Regex => Ok(Self::Regex),
        }
    }
}

impl From<StringOperation> for proto::string_filter::Operation {
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

impl TryFrom<proto::StringFilter> for StringFilter {
    type Error = SerDeError;

    fn try_from(value_proto: proto::StringFilter) -> Result<Self, Self::Error> {
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

impl From<StringFilter> for proto::StringFilter {
    fn from(value: StringFilter) -> proto::StringFilter {
        proto::StringFilter {
            operation: value.operation as i32,
            value: value.value,
            negated: value.negated,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct AndStringFilters {
    pub string_filters: Vec<StringFilter>,
}

impl TryFrom<proto::AndStringFilters> for AndStringFilters {
    type Error = SerDeError;
    fn try_from(value: proto::AndStringFilters) -> Result<Self, Self::Error> {
        let string_filters = value
            .string_filters
            .into_iter()
            .map(StringFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { string_filters })
    }
}

impl From<AndStringFilters> for proto::AndStringFilters {
    fn from(value: AndStringFilters) -> Self {
        Self {
            string_filters: value
                .string_filters
                .into_iter()
                .map(proto::StringFilter::from)
                .collect(),
        }
    }
}

impl From<Vec<StringCmp>> for AndStringFilters {
    fn from(cmps: Vec<StringCmp>) -> AndStringFilters {
        AndStringFilters {
            string_filters: cmps.into_iter().map(StringFilter::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
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

impl TryFrom<proto::OrStringFilters> for OrStringFilters {
    type Error = SerDeError;
    fn try_from(value: proto::OrStringFilters) -> Result<Self, Self::Error> {
        let and_string_filters = value
            .and_string_filters
            .into_iter()
            .map(AndStringFilters::try_from)
            .collect::<Result<_, SerDeError>>()?;
        Ok(Self { and_string_filters })
    }
}

impl From<OrStringFilters> for proto::OrStringFilters {
    fn from(value: OrStringFilters) -> Self {
        let and_string_filters = value
            .and_string_filters
            .into_iter()
            .map(proto::AndStringFilters::from)
            .collect();
        Self { and_string_filters }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UidOperation {
    Equal,
}

impl TryFrom<proto::uid_filter::Operation> for UidOperation {
    type Error = SerDeError;
    fn try_from(value_proto: proto::uid_filter::Operation) -> Result<Self, Self::Error> {
        match value_proto {
            proto::uid_filter::Operation::Unspecified => {
                Err(SerDeError::UnknownVariant("UidOperation"))
            }
            proto::uid_filter::Operation::Equal => Ok(Self::Equal),
        }
    }
}

impl From<UidOperation> for proto::uid_filter::Operation {
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

impl TryFrom<proto::UidFilter> for UidFilter {
    type Error = SerDeError;

    fn try_from(value_proto: proto::UidFilter) -> Result<Self, Self::Error> {
        let operation = value_proto.operation().try_into()?;
        let value = value_proto
            .value
            .ok_or(SerDeError::MissingField("uid"))?
            .try_into()?;
        Ok(Self { operation, value })
    }
}

impl From<UidFilter> for proto::UidFilter {
    fn from(value: UidFilter) -> proto::UidFilter {
        proto::UidFilter {
            operation: value.operation as i32,
            value: Some(value.value.into()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UidFilters {
    pub uid_filters: Vec<UidFilter>,
}

impl TryFrom<proto::UidFilters> for UidFilters {
    type Error = SerDeError;
    fn try_from(value: proto::UidFilters) -> Result<Self, Self::Error> {
        let uid_filters = value
            .uid_filters
            .into_iter()
            .map(UidFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { uid_filters })
    }
}

impl From<UidFilters> for proto::UidFilters {
    fn from(value: UidFilters) -> Self {
        Self {
            uid_filters: value
                .uid_filters
                .into_iter()
                .map(proto::UidFilter::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertyQuery {
    pub query_id: QueryId,
    pub node_type: NodeType,
    pub immutable_int_filters: FxHashMap<PropertyName, OrIntFilters>,
    pub max_int_filters: FxHashMap<PropertyName, OrIntFilters>,
    pub min_int_filters: FxHashMap<PropertyName, OrIntFilters>,
    pub string_filters: FxHashMap<PropertyName, OrStringFilters>,
    pub uid_filters: UidFilters,
}

impl NodePropertyQuery {
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            query_id: QueryId::default(),
            immutable_int_filters: Default::default(),
            max_int_filters: Default::default(),
            min_int_filters: Default::default(),
            string_filters: Default::default(),
            uid_filters: Default::default(),
        }
    }

    pub fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.query_id, other.query_id);
        debug_assert_eq!(self.node_type, other.node_type);
        self.string_filters.extend(other.string_filters);
        self.immutable_int_filters.extend(other.immutable_int_filters);
        self.max_int_filters.extend(other.max_int_filters);
        self.min_int_filters.extend(other.min_int_filters);
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

    pub fn with_immutable_int_filters(
        &mut self,
        property_name: PropertyName,
        filters: impl Into<AndIntFilters>,
    ) -> &mut Self {
        let filters = filters.into();
        self.immutable_int_filters
            .entry(property_name)
            .or_insert_with(|| OrIntFilters::with_capacity(1))
            .push(filters);
        self
    }

    pub fn with_max_int_filters(
        &mut self,
        property_name: PropertyName,
        filters: impl Into<AndIntFilters>,
    ) -> &mut Self {
        let filters = filters.into();
        self.max_int_filters
            .entry(property_name)
            .or_insert_with(|| OrIntFilters::with_capacity(1))
            .push(filters);
        self
    }

    pub fn with_min_int_filters(
        &mut self,
        property_name: PropertyName,
        filters: impl Into<AndIntFilters>,
    ) -> &mut Self {
        let filters = filters.into();
        self.min_int_filters
            .entry(property_name)
            .or_insert_with(|| OrIntFilters::with_capacity(1))
            .push(filters);
        self
    }
}

impl TryFrom<proto::NodePropertyQuery> for NodePropertyQuery {
    type Error = SerDeError;
    fn try_from(value: proto::NodePropertyQuery) -> Result<Self, Self::Error> {
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
                        assertion: e.to_string(),
                    })?,
                    v.try_into()?,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let immutable_int_filters = value
            .immutable_int_filters
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "immutable_int_filters",
                        assertion: e.to_string(),
                    })?,
                    v.try_into()?,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let max_int_filters = value
            .max_int_filters
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "max_int_filters",
                        assertion: e.to_string(),
                    })?,
                    v.try_into()?,
                ))
            })
            .collect::<Result<_, SerDeError>>()?;

        let min_int_filters = value
            .min_int_filters
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    PropertyName::try_from(k).map_err(|e| SerDeError::InvalidField {
                        field_name: "min_int_filters",
                        assertion: e.to_string(),
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
            immutable_int_filters,
            max_int_filters,
            min_int_filters,
            string_filters,
            uid_filters,
        })
    }
}

impl From<NodePropertyQuery> for proto::NodePropertyQuery {
    fn from(value: NodePropertyQuery) -> Self {
        let query_id = Some(value.query_id.into());
        let node_type = Some(value.node_type.into());
        let string_filters = value
            .string_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let immutable_int_filters = value
            .immutable_int_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let max_int_filters = value
            .max_int_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let min_int_filters = value
            .min_int_filters
            .into_iter()
            .map(|(k, v)| (k.value, v.into()))
            .collect();

        let uid_filters = value.uid_filters.into();

        Self {
            query_id,
            node_type,
            immutable_int_filters,
            max_int_filters,
            min_int_filters,
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
                immutable_int_filters: Default::default(),
                max_int_filters: Default::default(),
                min_int_filters: Default::default(),
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

impl TryFrom<proto::GraphQuery> for GraphQuery {
    type Error = SerDeError;
    fn try_from(value: proto::GraphQuery) -> Result<Self, Self::Error> {
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

impl From<GraphQuery> for proto::GraphQuery {
    fn from(value: GraphQuery) -> Self {
        let root_query_id = Some(value.root_query_id.into());
        let node_property_queries = Some(proto::NodePropertyQueryMap {
            entries: value
                .node_property_queries
                .into_iter()
                .map(|(k, v)| proto::NodePropertyQueryEntry {
                    query_id: Some(k.into()),
                    node_property_query: Some(v.into()),
                })
                .collect(),
        });
        let edge_filters = Some(proto::EdgeQueryMap {
            entries: value
                .edge_filters
                .into_iter()
                .map(|((k0, k1), v)| proto::EdgeQueryEntry {
                    query_id: Some(k0.into()),
                    edge_name: Some(k1.into()),
                    neighbor_query_ids: v.into_iter().map(proto::QueryId::from).collect(),
                })
                .collect(),
        });

        let edge_map = Some(proto::EdgeNameMap {
            entries: value
                .edge_map
                .into_iter()
                .map(|(k, v)| proto::EdgeNameEntry {
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

#[derive(Debug, Clone, Default)]
pub struct I64Properties {
    pub prop_map: FxHashMap<PropertyName, i64>,
}

impl I64Properties {
    pub fn merge(&mut self, other: Self) {
        self.prop_map.extend(other.prop_map);
    }

    pub fn add_i64_property(&mut self, property_name: PropertyName, value: i64) {
        self.prop_map.insert(property_name, value);
    }
}

impl TryFrom<proto::I64Properties> for I64Properties {
    type Error = SerDeError;
    fn try_from(value: proto::I64Properties) -> Result<Self, Self::Error> {
        let mut prop_map = FxHashMap::default();
        prop_map.reserve(value.properties.len());

        for property in value.properties {
            let property_name = property
                .property_name
                .ok_or_else(|| SerDeError::MissingField("property_name"))?;
            prop_map.insert(property_name.try_into()?, property.property_value);
        }

        Ok(Self { prop_map })
    }
}

impl From<I64Properties> for proto::I64Properties {
    fn from(value: I64Properties) -> Self {
        let props_as_vec: Vec<proto::I64Property> = value
            .prop_map
            .into_iter()
            .map(|(k, v)| proto::I64Property {
                property_name: Some(k.into()),
                property_value: v,
            })
            .collect();
        proto::I64Properties {
            properties: props_as_vec,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StringProperties {
    pub prop_map: FxHashMap<PropertyName, String>,
}

impl StringProperties {
    pub fn merge(&mut self, other: Self) {
        self.prop_map.extend(other.prop_map);
    }

    pub fn add_string_property(&mut self, property_name: PropertyName, value: String) {
        self.prop_map.insert(property_name, value);
    }
}

impl TryFrom<proto::StringProperties> for StringProperties {
    type Error = SerDeError;
    fn try_from(value: proto::StringProperties) -> Result<Self, Self::Error> {
        let mut prop_map = FxHashMap::default();
        prop_map.reserve(value.properties.len());

        for string_property in value.properties {
            let property_name = string_property
                .property_name
                .ok_or_else(|| SerDeError::MissingField("property_name"))?;
            prop_map.insert(property_name.try_into()?, string_property.property_value);
        }

        Ok(Self { prop_map })
    }
}

impl From<StringProperties> for proto::StringProperties {
    fn from(value: StringProperties) -> Self {
        let props_as_vec: Vec<proto::StringProperty> = value
            .prop_map
            .into_iter()
            .map(|(k, v)| proto::StringProperty {
                property_name: Some(k.into()),
                property_value: v,
            })
            .collect();
        proto::StringProperties {
            properties: props_as_vec,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertiesView {
    pub uid: Uid,
    pub node_type: NodeType,
    pub string_properties: StringProperties,
    pub immutable_i64_properties: I64Properties,
    pub max_i64_properties: I64Properties,
    pub min_i64_properties: I64Properties,
}

impl NodePropertiesView {
    pub fn new(
        uid: Uid,
        node_type: NodeType,
        string_properties: StringProperties,
        immutable_i64_properties: I64Properties,
        max_i64_properties: I64Properties,
        min_i64_properties: I64Properties,
    ) -> Self {
        Self {
            uid,
            node_type,
            string_properties,
            immutable_i64_properties,
            max_i64_properties,
            min_i64_properties,
        }
    }

    pub fn merge(&mut self, other: Self) {
        debug_assert_eq!(self.uid, other.uid);
        debug_assert_eq!(self.node_type, other.node_type);
        self.string_properties.merge(other.string_properties);
        self.immutable_i64_properties.merge(other.immutable_i64_properties);
        self.max_i64_properties.merge(other.max_i64_properties);
        self.min_i64_properties.merge(other.min_i64_properties);
    }

    pub fn add_string_property(&mut self, property_name: PropertyName, value: String) {
        self.string_properties
            .add_string_property(property_name, value);
    }

    pub fn add_immutable_i64_property(&mut self, property_name: PropertyName, value: i64) {
        self.immutable_i64_properties
            .add_i64_property(property_name, value);
    }


    pub fn add_max_i64_property(&mut self, property_name: PropertyName, value: i64) {
        self.immutable_i64_properties
            .add_i64_property(property_name, value);
    }


    pub fn add_min_i64_property(&mut self, property_name: PropertyName, value: i64) {
        self.immutable_i64_properties
            .add_i64_property(property_name, value);
    }


}

impl TryFrom<proto::NodePropertiesView> for NodePropertiesView {
    type Error = SerDeError;
    fn try_from(value: proto::NodePropertiesView) -> Result<Self, Self::Error> {
        let proto_string_properties = value
            .string_properties
            .ok_or_else(|| SerDeError::MissingField("string_properties"))?;

        let string_properties = StringProperties::try_from(proto_string_properties)?;

        let proto_immutable_i64_properties = value
            .immutable_i64_properties
            .ok_or_else(|| SerDeError::MissingField("immutable_i64_properties"))?;


        let proto_max_i64_properties = value
            .max_i64_properties
            .ok_or_else(|| SerDeError::MissingField("max_i64_properties"))?;

        let proto_min_i64_properties = value
            .min_i64_properties
            .ok_or_else(|| SerDeError::MissingField("min_i64_properties"))?;

        let immutable_i64_properties = I64Properties::try_from(proto_immutable_i64_properties)?;
        let max_i64_properties = I64Properties::try_from(proto_max_i64_properties)?;
        let min_i64_properties = I64Properties::try_from(proto_min_i64_properties)?;

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
            immutable_i64_properties,
            max_i64_properties,
            min_i64_properties,
        })
    }
}

impl From<NodePropertiesView> for proto::NodePropertiesView {
    fn from(value: NodePropertiesView) -> Self {
        let string_properties = proto::StringProperties::from(value.string_properties);
        let immutable_i64_properties = proto::I64Properties::from(value.immutable_i64_properties);
        let max_i64_properties = proto::I64Properties::from(value.max_i64_properties);
        let min_i64_properties = proto::I64Properties::from(value.min_i64_properties);

        Self {
            uid: Some(value.uid.into()),
            node_type: Some(value.node_type.into()),
            string_properties: Some(string_properties),
            immutable_i64_properties: Some(immutable_i64_properties),
            max_i64_properties: Some(max_i64_properties),
            min_i64_properties: Some(min_i64_properties),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertiesViewEntry {
    pub uid: Uid,
    pub node_view: NodePropertiesView,
}

impl TryFrom<proto::NodePropertiesViewEntry> for NodePropertiesViewEntry {
    type Error = SerDeError;
    fn try_from(value: proto::NodePropertiesViewEntry) -> Result<Self, Self::Error> {
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

impl From<NodePropertiesViewEntry> for proto::NodePropertiesViewEntry {
    fn from(value: NodePropertiesViewEntry) -> Self {
        proto::NodePropertiesViewEntry {
            uid: Some(value.uid.into()),
            node_view: Some(value.node_view.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodePropertiesViewMap {
    pub entries: FxHashMap<Uid, NodePropertiesView>,
}

impl TryFrom<proto::NodePropertiesViewMap> for NodePropertiesViewMap {
    type Error = SerDeError;
    fn try_from(value: proto::NodePropertiesViewMap) -> Result<Self, Self::Error> {
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

impl From<NodePropertiesViewMap> for proto::NodePropertiesViewMap {
    fn from(value: NodePropertiesViewMap) -> Self {
        let mut entries = Vec::with_capacity(value.entries.len());

        for (uid, node_view) in value.entries.into_iter() {
            entries.push(proto::NodePropertiesViewEntry {
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

impl TryFrom<proto::EdgeViewEntry> for EdgeViewEntry {
    type Error = SerDeError;
    fn try_from(value: proto::EdgeViewEntry) -> Result<Self, Self::Error> {
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

impl From<EdgeViewEntry> for proto::EdgeViewEntry {
    fn from(value: EdgeViewEntry) -> Self {
        let mut neighbors = Vec::with_capacity(value.neighbors.len());
        for neighbor in value.neighbors.into_iter() {
            neighbors.push(neighbor.into());
        }
        proto::EdgeViewEntry {
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

impl TryFrom<proto::EdgeViewMap> for EdgeViewMap {
    type Error = SerDeError;
    fn try_from(value: proto::EdgeViewMap) -> Result<Self, Self::Error> {
        let mut entries = FxHashMap::default();
        entries.reserve(value.entries.len());

        for entry in value.entries.into_iter() {
            let entry: EdgeViewEntry = entry.try_into()?;
            entries.insert((entry.uid, entry.edge_name), entry.neighbors);
        }

        Ok(Self { entries })
    }
}

impl From<EdgeViewMap> for proto::EdgeViewMap {
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
    pub fn new_node(&mut self, uid: Uid, node_type: NodeType) -> &mut NodePropertiesView {
        self.nodes
            .entry(uid)
            .or_insert_with(|| NodePropertiesView::new(
                uid,
                node_type,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ))
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
            self.add_edges(src_uid, edge_name, dst_uids);
        }
    }

    pub fn get_nodes(&self) -> &FxHashMap<Uid, NodePropertiesView> {
        &self.nodes
    }
}

impl TryFrom<proto::GraphView> for GraphView {
    type Error = SerDeError;
    fn try_from(value: proto::GraphView) -> Result<Self, Self::Error> {
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

impl From<GraphView> for proto::GraphView {
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
pub struct QueryGraphWithUidRequest {
    pub tenant_id: uuid::Uuid,
    pub node_uid: Uid,
    pub graph_query: GraphQuery,
}

impl TryFrom<proto::QueryGraphWithUidRequest> for QueryGraphWithUidRequest {
    type Error = SerDeError;

    fn try_from(value: proto::QueryGraphWithUidRequest) -> Result<Self, Self::Error> {
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

impl From<QueryGraphWithUidRequest> for proto::QueryGraphWithUidRequest {
    fn from(value: QueryGraphWithUidRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_uid: Some(value.node_uid.into()),
            graph_query: Some(value.graph_query.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchedGraphWithUid {
    pub matched_graph: GraphView,
    pub root_uid: Uid,
}

impl TryFrom<proto::MatchedGraphWithUid> for MatchedGraphWithUid {
    type Error = SerDeError;
    fn try_from(value: proto::MatchedGraphWithUid) -> Result<Self, Self::Error> {
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

impl From<MatchedGraphWithUid> for proto::MatchedGraphWithUid {
    fn from(value: MatchedGraphWithUid) -> Self {
        Self {
            matched_graph: Some(value.matched_graph.into()),
            root_uid: Some(value.root_uid.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoMatchWithUid {}

impl From<proto::NoMatchWithUid> for NoMatchWithUid {
    fn from(value: proto::NoMatchWithUid) -> Self {
        let proto::NoMatchWithUid {} = value;
        Self {}
    }
}

impl From<NoMatchWithUid> for proto::NoMatchWithUid {
    fn from(value: NoMatchWithUid) -> Self {
        let NoMatchWithUid {} = value;
        Self {}
    }
}

#[derive(Debug, Clone)]
pub enum MaybeMatchWithUid {
    Matched(MatchedGraphWithUid),
    Missed(NoMatchWithUid),
}

impl TryFrom<proto::MaybeMatchWithUid> for MaybeMatchWithUid {
    type Error = SerDeError;
    fn try_from(value_proto: proto::MaybeMatchWithUid) -> Result<Self, Self::Error> {
        match value_proto.inner {
            Some(proto::maybe_match_with_uid::Inner::Matched(matched)) => {
                Ok(MaybeMatchWithUid::Matched(matched.try_into()?))
            }
            Some(proto::maybe_match_with_uid::Inner::Missed(missed)) => {
                Ok(MaybeMatchWithUid::Missed(missed.into()))
            }
            None => Err(SerDeError::UnknownVariant("MaybeMatchWithUid")),
        }
    }
}

impl From<MaybeMatchWithUid> for proto::MaybeMatchWithUid {
    fn from(value: MaybeMatchWithUid) -> Self {
        match value {
            MaybeMatchWithUid::Matched(matched) => proto::MaybeMatchWithUid {
                inner: Some(proto::maybe_match_with_uid::Inner::Matched(matched.into())),
            },
            MaybeMatchWithUid::Missed(p) => proto::MaybeMatchWithUid {
                inner: Some(proto::maybe_match_with_uid::Inner::Missed(p.into())),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphWithUidResponse {
    pub maybe_match: MaybeMatchWithUid,
}

impl TryFrom<proto::QueryGraphWithUidResponse> for QueryGraphWithUidResponse {
    type Error = SerDeError;
    fn try_from(value: proto::QueryGraphWithUidResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            maybe_match: value
                .maybe_match
                .ok_or(SerDeError::MissingField("maybe_match"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphWithUidResponse> for proto::QueryGraphWithUidResponse {
    fn from(value: QueryGraphWithUidResponse) -> Self {
        Self {
            maybe_match: Some(value.maybe_match.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromUidRequest {
    pub tenant_id: uuid::Uuid,
    pub node_uid: Uid,
    pub graph_query: GraphQuery,
}

impl TryFrom<proto::QueryGraphFromUidRequest> for QueryGraphFromUidRequest {
    type Error = SerDeError;

    fn try_from(value: proto::QueryGraphFromUidRequest) -> Result<Self, Self::Error> {
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

impl From<QueryGraphFromUidRequest> for proto::QueryGraphFromUidRequest {
    fn from(value: QueryGraphFromUidRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            node_uid: Some(value.node_uid.into()),
            graph_query: Some(value.graph_query.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromUidResponse {
    pub matched_graph: Option<GraphView>,
}

impl TryFrom<proto::QueryGraphFromUidResponse> for QueryGraphFromUidResponse {
    type Error = SerDeError;
    fn try_from(value: proto::QueryGraphFromUidResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            matched_graph: value.matched_graph.map(|g| g.try_into()).transpose()?,
        })
    }
}

impl From<QueryGraphFromUidResponse> for proto::QueryGraphFromUidResponse {
    fn from(value: QueryGraphFromUidResponse) -> Self {
        Self {
            matched_graph: value.matched_graph.map(Into::into),
        }
    }
}
