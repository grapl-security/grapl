use std::collections::HashMap;

use crate::{
    graplinc::grapl::common::v1beta1::types::Uid,
    protobufs::graplinc::grapl::api::graph::v1beta1::{
        DecrementOnlyIntProp as DecrementOnlyIntPropProto,
        DecrementOnlyUintProp as DecrementOnlyUintPropProto,
        Edge as EdgeProto,
        EdgeList as EdgeListProto,
        GraphDescription as GraphDescriptionProto,
        IdStrategy as IdStrategyProto,
        IdentifiedEdge as IdentifiedEdgeProto,
        IdentifiedEdgeList as IdentifiedEdgeListProto,
        IdentifiedGraph as IdentifiedGraphProto,
        IdentifiedNode as IdentifiedNodeProto,
        ImmutableIntProp as ImmutableIntPropProto,
        ImmutableStrProp as ImmutableStrPropProto,
        ImmutableUintProp as ImmutableUintPropProto,
        IncrementOnlyIntProp as IncrementOnlyIntPropProto,
        IncrementOnlyUintProp as IncrementOnlyUintPropProto,
        Lens as LensProto,
        NodeDescription as NodeDescriptionProto,
        NodeProperty as NodePropertyProto,
        Session as SessionProto,
        Static as StaticProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

// A helper macro to generate `From` impl boilerplate.
macro_rules ! impl_from_for_unit {
    ($into_t:ty, $field:tt, $from_t:ty) => {
        impl From<$from_t> for $into_t
        {
            fn from(p: $from_t) -> Self {
                let p = p.to_owned().into();
                Self {$field: p}
            }
        }
    };
    ($into_t:ty, $field:tt, $head:ty, $($tail:ty),*) => {
        impl_from_for_unit!($into_t, $field, $head);
        impl_from_for_unit!($into_t, $field, $($tail),*);
    };
}

//
// Session
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Session {
    pub primary_key_properties: Vec<String>,
    pub primary_key_requires_asset_id: bool,
    pub create_time: u64,    // TODO: use Timestamp
    pub last_seen_time: u64, // TODO: use Timestamp
    pub terminate_time: u64, // TODO: use Timestamp
}

impl From<SessionProto> for Session {
    fn from(session_proto: SessionProto) -> Self {
        Session {
            primary_key_properties: session_proto.primary_key_properties,
            primary_key_requires_asset_id: session_proto.primary_key_requires_asset_id,
            create_time: session_proto.create_time,
            last_seen_time: session_proto.last_seen_time,
            terminate_time: session_proto.terminate_time,
        }
    }
}

impl From<Session> for SessionProto {
    fn from(session: Session) -> Self {
        SessionProto {
            primary_key_properties: session.primary_key_properties,
            primary_key_requires_asset_id: session.primary_key_requires_asset_id,
            create_time: session.create_time,
            last_seen_time: session.last_seen_time,
            terminate_time: session.terminate_time,
        }
    }
}

impl type_url::TypeUrl for Session {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.Session";
}

impl serde_impl::ProtobufSerializable for Session {
    type ProtobufMessage = SessionProto;
}

//
// Static
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Static {
    pub primary_key_properties: Vec<String>,
    pub primary_key_requires_asset_id: bool,
}

impl From<StaticProto> for Static {
    fn from(static_proto: StaticProto) -> Self {
        Static {
            primary_key_properties: static_proto.primary_key_properties,
            primary_key_requires_asset_id: static_proto.primary_key_requires_asset_id,
        }
    }
}

impl From<Static> for StaticProto {
    fn from(static_: Static) -> Self {
        StaticProto {
            primary_key_properties: static_.primary_key_properties,
            primary_key_requires_asset_id: static_.primary_key_requires_asset_id,
        }
    }
}

impl type_url::TypeUrl for Static {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.Static";
}

impl serde_impl::ProtobufSerializable for Static {
    type ProtobufMessage = StaticProto;
}

//
// IdStrategy
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Strategy {
    Session(Session),
    Static(Static),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdStrategy {
    pub strategy: Strategy,
}

impl TryFrom<IdStrategyProto> for IdStrategy {
    type Error = SerDeError;

    fn try_from(id_strategy_proto: IdStrategyProto) -> Result<Self, Self::Error> {
        match id_strategy_proto.strategy {
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::id_strategy::Strategy::Session(
                    session_proto
                )
            ) => {
                let session: Session = session_proto.into();
                Ok(IdStrategy {
                    strategy: Strategy::Session(session)
                })
            },
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::id_strategy::Strategy::Static(
                    static_proto
                )
            ) => {
                let static_: Static = static_proto.into();
                Ok(IdStrategy {
                    strategy: Strategy::Static(static_)
                })
            },
            None => Err(SerDeError::MissingField("strategy")),
        }
    }
}

impl From<IdStrategy> for IdStrategyProto {
    fn from(id_strategy: IdStrategy) -> Self {
        match id_strategy.strategy {
            Strategy::Session(session) => IdStrategyProto {
                strategy: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::id_strategy::Strategy::Session(
                        session.into()
                    )
                ),
            },
            Strategy::Static(static_) => IdStrategyProto {
                strategy: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::id_strategy::Strategy::Static(
                        static_.into()
                    )
                ),
            },
        }
    }
}

impl type_url::TypeUrl for IdStrategy {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdStrategy";
}

impl serde_impl::ProtobufSerializable for IdStrategy {
    type ProtobufMessage = IdStrategyProto;
}

impl From<Static> for IdStrategy {
    fn from(strategy: Static) -> IdStrategy {
        IdStrategy {
            strategy: Strategy::Static(strategy),
        }
    }
}

impl From<Session> for IdStrategy {
    fn from(strategy: Session) -> IdStrategy {
        IdStrategy {
            strategy: Strategy::Session(strategy),
        }
    }
}

//
// IncrementOnlyUintProp
//

#[derive(Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub struct IncrementOnlyUintProp {
    pub prop: u64,
}

impl IncrementOnlyUintProp {
    pub fn as_inner(&self) -> u64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="IncrementOnlyUintProp merge", self_prop=?self, other_prop=?other_prop);
        self.prop = std::cmp::max(self.prop, other_prop.prop);
    }
}

impl From<IncrementOnlyUintPropProto> for IncrementOnlyUintProp {
    fn from(increment_only_uint_prop_proto: IncrementOnlyUintPropProto) -> Self {
        IncrementOnlyUintProp {
            prop: increment_only_uint_prop_proto.prop,
        }
    }
}

impl From<IncrementOnlyUintProp> for IncrementOnlyUintPropProto {
    fn from(increment_only_uint_prop: IncrementOnlyUintProp) -> Self {
        IncrementOnlyUintPropProto {
            prop: increment_only_uint_prop.prop,
        }
    }
}

impl type_url::TypeUrl for IncrementOnlyUintProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IncrementOnlyUintProp";
}

impl serde_impl::ProtobufSerializable for IncrementOnlyUintProp {
    type ProtobufMessage = IncrementOnlyUintPropProto;
}

impl std::string::ToString for IncrementOnlyUintProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    IncrementOnlyUintProp,
    prop,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);

//
// ImmutableUintProp
//

#[derive(Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub struct ImmutableUintProp {
    pub prop: u64,
}

impl ImmutableUintProp {
    pub fn as_inner(&self) -> u64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="ImmutableUintProp merge", self_prop=?self, other_prop=?other_prop);
    }
}

impl From<ImmutableUintPropProto> for ImmutableUintProp {
    fn from(immutable_uint_prop_proto: ImmutableUintPropProto) -> Self {
        ImmutableUintProp {
            prop: immutable_uint_prop_proto.prop,
        }
    }
}

impl From<ImmutableUintProp> for ImmutableUintPropProto {
    fn from(immutable_uint_prop: ImmutableUintProp) -> Self {
        ImmutableUintPropProto {
            prop: immutable_uint_prop.prop,
        }
    }
}

impl type_url::TypeUrl for ImmutableUintProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.ImmutableUintProp";
}

impl serde_impl::ProtobufSerializable for ImmutableUintProp {
    type ProtobufMessage = ImmutableUintPropProto;
}

impl std::string::ToString for ImmutableUintProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    ImmutableUintProp,
    prop,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);

//
// DecrementOnlyUintProp
//

#[derive(Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub struct DecrementOnlyUintProp {
    pub prop: u64,
}

impl DecrementOnlyUintProp {
    pub fn as_inner(&self) -> u64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        self.prop = std::cmp::min(self.prop, other_prop.prop);
    }
}

impl From<DecrementOnlyUintPropProto> for DecrementOnlyUintProp {
    fn from(decrement_only_uint_prop_proto: DecrementOnlyUintPropProto) -> Self {
        DecrementOnlyUintProp {
            prop: decrement_only_uint_prop_proto.prop,
        }
    }
}

impl From<DecrementOnlyUintProp> for DecrementOnlyUintPropProto {
    fn from(decrement_only_uint_prop: DecrementOnlyUintProp) -> Self {
        DecrementOnlyUintPropProto {
            prop: decrement_only_uint_prop.prop,
        }
    }
}

impl type_url::TypeUrl for DecrementOnlyUintProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.DecrementOnlyUintProp";
}

impl serde_impl::ProtobufSerializable for DecrementOnlyUintProp {
    type ProtobufMessage = DecrementOnlyUintPropProto;
}

impl std::string::ToString for DecrementOnlyUintProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    DecrementOnlyUintProp,
    prop,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);

//
// IncrementOnlyIntProp
//

#[derive(Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub struct IncrementOnlyIntProp {
    pub prop: i64,
}

impl IncrementOnlyIntProp {
    pub fn as_inner(&self) -> i64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="IncrementOnlyIntProp merge", self_prop=?self, other_prop=?other_prop);
        self.prop = std::cmp::max(self.prop, other_prop.prop);
    }
}

impl From<IncrementOnlyIntPropProto> for IncrementOnlyIntProp {
    fn from(increment_only_int_prop_proto: IncrementOnlyIntPropProto) -> Self {
        IncrementOnlyIntProp {
            prop: increment_only_int_prop_proto.prop,
        }
    }
}

impl From<IncrementOnlyIntProp> for IncrementOnlyIntPropProto {
    fn from(increment_only_int_prop: IncrementOnlyIntProp) -> Self {
        IncrementOnlyIntPropProto {
            prop: increment_only_int_prop.prop,
        }
    }
}

impl type_url::TypeUrl for IncrementOnlyIntProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IncrementOnlyIntProp";
}

impl serde_impl::ProtobufSerializable for IncrementOnlyIntProp {
    type ProtobufMessage = IncrementOnlyIntPropProto;
}

impl std::string::ToString for IncrementOnlyIntProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    IncrementOnlyIntProp,
    prop,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);

//
// DecrementOnlyIntProp
//

#[derive(Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub struct DecrementOnlyIntProp {
    pub prop: i64,
}

impl DecrementOnlyIntProp {
    pub fn as_inner(&self) -> i64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        self.prop = std::cmp::min(self.prop, other_prop.prop);
    }
}

impl From<DecrementOnlyIntPropProto> for DecrementOnlyIntProp {
    fn from(decrement_only_int_prop_proto: DecrementOnlyIntPropProto) -> Self {
        DecrementOnlyIntProp {
            prop: decrement_only_int_prop_proto.prop,
        }
    }
}

impl From<DecrementOnlyIntProp> for DecrementOnlyIntPropProto {
    fn from(decrement_only_int_prop: DecrementOnlyIntProp) -> Self {
        DecrementOnlyIntPropProto {
            prop: decrement_only_int_prop.prop,
        }
    }
}

impl type_url::TypeUrl for DecrementOnlyIntProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.DecrementOnlyIntProp";
}

impl serde_impl::ProtobufSerializable for DecrementOnlyIntProp {
    type ProtobufMessage = DecrementOnlyIntPropProto;
}

impl std::string::ToString for DecrementOnlyIntProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    DecrementOnlyIntProp,
    prop,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);

//
// ImmutableIntProp
//

#[derive(Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Hash)]
pub struct ImmutableIntProp {
    pub prop: i64,
}

impl ImmutableIntProp {
    pub fn as_inner(&self) -> i64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="ImmutableIntProp merge", self_prop=?self, other_prop=?other_prop);
    }
}

impl From<ImmutableIntPropProto> for ImmutableIntProp {
    fn from(immutable_int_prop_proto: ImmutableIntPropProto) -> Self {
        ImmutableIntProp {
            prop: immutable_int_prop_proto.prop,
        }
    }
}

impl From<ImmutableIntProp> for ImmutableIntPropProto {
    fn from(immutable_int_prop: ImmutableIntProp) -> Self {
        ImmutableIntPropProto {
            prop: immutable_int_prop.prop,
        }
    }
}

impl type_url::TypeUrl for ImmutableIntProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.ImmutableIntProp";
}

impl serde_impl::ProtobufSerializable for ImmutableIntProp {
    type ProtobufMessage = ImmutableIntPropProto;
}

impl std::string::ToString for ImmutableIntProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    ImmutableIntProp,
    prop,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);

//
// ImmutableStrProp
//

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ImmutableStrProp {
    pub prop: String,
}

impl ImmutableStrProp {
    pub fn as_inner(&self) -> &str {
        self.prop.as_str()
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="ImmutableStrProp merge", self_prop=?self, other_prop=?other_prop);
    }
}

impl From<ImmutableStrPropProto> for ImmutableStrProp {
    fn from(immutable_str_prop_proto: ImmutableStrPropProto) -> Self {
        ImmutableStrProp {
            prop: immutable_str_prop_proto.prop,
        }
    }
}

impl From<ImmutableStrProp> for ImmutableStrPropProto {
    fn from(immutable_str_prop: ImmutableStrProp) -> Self {
        ImmutableStrPropProto {
            prop: immutable_str_prop.prop,
        }
    }
}

impl type_url::TypeUrl for ImmutableStrProp {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.ImmutableStrProp";
}

impl serde_impl::ProtobufSerializable for ImmutableStrProp {
    type ProtobufMessage = ImmutableStrPropProto;
}

impl std::string::ToString for ImmutableStrProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl_from_for_unit!(
    ImmutableStrProp,
    prop,
    String,
    &String,
    &str,
    &std::borrow::Cow<'_, str>
);

//
// NodeProperty
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Property {
    IncrementOnlyUintProp(IncrementOnlyUintProp),
    DecrementOnlyUintProp(DecrementOnlyUintProp),
    ImmutableUintProp(ImmutableUintProp),
    IncrementOnlyIntProp(IncrementOnlyIntProp),
    DecrementOnlyIntProp(DecrementOnlyIntProp),
    ImmutableIntProp(ImmutableIntProp),
    ImmutableStrProp(ImmutableStrProp),
}

impl Property {
    pub fn merge_property(&mut self, other: &Self) {
        match (self, other) {
            (
                Property::IncrementOnlyUintProp(ref mut self_prop),
                Property::IncrementOnlyUintProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                Property::ImmutableUintProp(ref mut self_prop),
                Property::ImmutableUintProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                Property::DecrementOnlyUintProp(ref mut self_prop),
                Property::DecrementOnlyUintProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                Property::DecrementOnlyIntProp(ref mut self_prop),
                Property::DecrementOnlyIntProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                Property::IncrementOnlyIntProp(ref mut self_prop),
                Property::IncrementOnlyIntProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                Property::ImmutableIntProp(ref mut self_prop),
                Property::ImmutableIntProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                Property::ImmutableStrProp(ref mut self_prop),
                Property::ImmutableStrProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            // technically we could improve type safety here by exhausting the combinations,
            // but I'm not going to type that all out right now
            // TODO: figure out what this comment means ^^
            (p, op) => {
                // Currently we don't guarantee that randomly generated nodes will have consistent
                // property types when they share a property name
                // TODO: figure out what this comment means ^^
                debug_assert!(false, "Invalid property merge: {:?} {:?}", p, op);
                tracing::warn!("Invalid property merge: {:?} {:?}", p, op);
            }
        }
    }
}

impl From<ImmutableUintProp> for Property {
    fn from(p: ImmutableUintProp) -> Self {
        Self::ImmutableUintProp(p)
    }
}

impl From<IncrementOnlyUintProp> for Property {
    fn from(p: IncrementOnlyUintProp) -> Self {
        Self::IncrementOnlyUintProp(p)
    }
}

impl From<DecrementOnlyUintProp> for Property {
    fn from(p: DecrementOnlyUintProp) -> Self {
        Self::DecrementOnlyUintProp(p)
    }
}

impl From<ImmutableIntProp> for Property {
    fn from(p: ImmutableIntProp) -> Self {
        Self::ImmutableIntProp(p)
    }
}

impl From<IncrementOnlyIntProp> for Property {
    fn from(p: IncrementOnlyIntProp) -> Self {
        Self::IncrementOnlyIntProp(p)
    }
}

impl From<DecrementOnlyIntProp> for Property {
    fn from(p: DecrementOnlyIntProp) -> Self {
        Self::DecrementOnlyIntProp(p)
    }
}

impl From<ImmutableStrProp> for Property {
    fn from(p: ImmutableStrProp) -> Self {
        Self::ImmutableStrProp(p)
    }
}

impl std::string::ToString for Property {
    fn to_string(&self) -> String {
        match self {
            Property::IncrementOnlyUintProp(increment_only_uint_prop) => {
                increment_only_uint_prop.to_string()
            }
            Property::ImmutableUintProp(immutable_uint_prop) => immutable_uint_prop.to_string(),
            Property::DecrementOnlyUintProp(decrement_only_uint_prop) => {
                decrement_only_uint_prop.to_string()
            }
            Property::DecrementOnlyIntProp(decrement_only_int_prop) => {
                decrement_only_int_prop.to_string()
            }
            Property::IncrementOnlyIntProp(increment_only_int_prop) => {
                increment_only_int_prop.to_string()
            }
            Property::ImmutableIntProp(immutable_int_prop) => immutable_int_prop.to_string(),
            Property::ImmutableStrProp(immutable_str_prop) => immutable_str_prop.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeProperty {
    pub property: Property,
}

impl NodeProperty {
    pub fn merge(&mut self, other: &Self) {
        self.property.merge_property(&other.property)
    }

    pub fn as_increment_only_uint(&self) -> Option<IncrementOnlyUintProp> {
        match self.property {
            Property::IncrementOnlyUintProp(ref prop) => Some(prop.clone()),
            _ => None,
        }
    }

    pub fn as_immutable_uint(&self) -> Option<ImmutableUintProp> {
        match self.property {
            Property::ImmutableUintProp(ref prop) => Some(prop.clone()),
            _ => None,
        }
    }

    pub fn as_decrement_only_uint(&self) -> Option<DecrementOnlyUintProp> {
        match self.property {
            Property::DecrementOnlyUintProp(ref prop) => Some(prop.clone()),
            _ => None,
        }
    }

    pub fn as_decrement_only_int(&self) -> Option<DecrementOnlyIntProp> {
        match self.property {
            Property::DecrementOnlyIntProp(ref prop) => Some(prop.clone()),
            _ => None,
        }
    }

    pub fn as_increment_only_int(&self) -> Option<IncrementOnlyIntProp> {
        match self.property {
            Property::IncrementOnlyIntProp(ref prop) => Some(prop.clone()),
            _ => None,
        }
    }

    pub fn as_immutable_int(&self) -> Option<ImmutableIntProp> {
        match self.property {
            Property::ImmutableIntProp(ref prop) => Some(prop.clone()),
            _ => None,
        }
    }

    pub fn as_immutable_str(&self) -> Option<&ImmutableStrProp> {
        match self.property {
            Property::ImmutableStrProp(ref prop) => Some(prop),
            _ => None,
        }
    }
}

impl TryFrom<NodePropertyProto> for NodeProperty {
    type Error = SerDeError;

    fn try_from(node_property_proto: NodePropertyProto) -> Result<Self, Self::Error> {
        match node_property_proto.property {
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::IncrementOnlyUint(
                    increment_only_uint_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::IncrementOnlyUintProp(
                    increment_only_uint_prop_proto.into()
                )
            }),
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::DecrementOnlyUint(
                    decrement_only_uint_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::DecrementOnlyUintProp(
                    decrement_only_uint_prop_proto.into()
                )
            }),
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::ImmutableUint(
                    immutable_uint_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::ImmutableUintProp(
                    immutable_uint_prop_proto.into()
                )
            }),
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::IncrementOnlyInt(
                    increment_only_int_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::IncrementOnlyIntProp(
                    increment_only_int_prop_proto.into()
                )
            }),
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::DecrementOnlyInt(
                    decrement_only_int_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::DecrementOnlyIntProp(
                    decrement_only_int_prop_proto.into()
                )
            }),
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::ImmutableInt(
                    immutable_int_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::ImmutableIntProp(
                    immutable_int_prop_proto.into()
                )
            }),
            Some(
                crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::ImmutableStr(
                    immutable_str_prop_proto
                )
            ) => Ok(NodeProperty {
                property: Property::ImmutableStrProp(
                    immutable_str_prop_proto.into()
                )
            }),
            None => Err(SerDeError::MissingField("property")),
        }
    }
}

impl From<NodeProperty> for NodePropertyProto {
    fn from(node_property: NodeProperty) -> Self {
        match node_property.property {
            Property::IncrementOnlyUintProp(increment_only_uint_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::IncrementOnlyUint(
                        increment_only_uint_prop.into()
                    )
                )
            },
            Property::DecrementOnlyUintProp(decrement_only_uint_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::DecrementOnlyUint(
                        decrement_only_uint_prop.into()
                    )
                )
            },
            Property::ImmutableUintProp(immutable_uint_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::ImmutableUint(
                        immutable_uint_prop.into()
                    )
                )
            },
            Property::IncrementOnlyIntProp(increment_only_int_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::IncrementOnlyInt(
                        increment_only_int_prop.into()
                    )
                )
            },
            Property::DecrementOnlyIntProp(decrement_only_int_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::DecrementOnlyInt(
                        decrement_only_int_prop.into()
                    )
                )
            },
            Property::ImmutableIntProp(immutable_int_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::ImmutableInt(
                        immutable_int_prop.into()
                    )
                )
            },
            Property::ImmutableStrProp(immutable_str_prop) => NodePropertyProto {
                property: Some(
                    crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property::ImmutableStr(
                        immutable_str_prop.into()
                    )
                )
            },
        }
    }
}

impl type_url::TypeUrl for NodeProperty {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.NodeProperty";
}

impl serde_impl::ProtobufSerializable for NodeProperty {
    type ProtobufMessage = NodePropertyProto;
}

impl std::string::ToString for NodeProperty {
    fn to_string(&self) -> String {
        match &self.property {
            Property::IncrementOnlyUintProp(increment_only_uint_prop) => {
                increment_only_uint_prop.to_string()
            }
            Property::ImmutableUintProp(immutable_uint_prop) => immutable_uint_prop.to_string(),
            Property::DecrementOnlyUintProp(decrement_only_uint_prop) => {
                decrement_only_uint_prop.to_string()
            }
            Property::DecrementOnlyIntProp(decrement_only_int_prop) => {
                decrement_only_int_prop.to_string()
            }
            Property::IncrementOnlyIntProp(increment_only_int_prop) => {
                increment_only_int_prop.to_string()
            }
            Property::ImmutableIntProp(immutable_int_prop) => immutable_int_prop.to_string(),
            Property::ImmutableStrProp(immutable_str_prop) => immutable_str_prop.to_string(),
        }
    }
}

impl<T> From<T> for NodeProperty
where
    T: Into<Property>,
{
    fn from(t: T) -> Self {
        NodeProperty { property: t.into() }
    }
}

//
// NodeDescription
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NodeDescription {
    pub properties: HashMap<String, NodeProperty>,
    pub node_key: String,
    pub node_type: String,
    pub id_strategy: Vec<IdStrategy>,
}

impl NodeDescription {
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<&NodeProperty> {
        self.properties.get(name.as_ref())
    }

    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<NodeProperty>) {
        self.properties.insert(name.into(), value.into());
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }
}

impl NodeDescription {
    pub fn merge(&mut self, other: &Self) {
        debug_assert_eq!(self.node_type, other.node_type);
        debug_assert_eq!(self.node_key, other.node_key);
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }
    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

impl TryFrom<NodeDescriptionProto> for NodeDescription {
    type Error = SerDeError;

    fn try_from(node_description_proto: NodeDescriptionProto) -> Result<Self, Self::Error> {
        let mut properties = HashMap::with_capacity(node_description_proto.properties.len());
        for (key, property) in node_description_proto.properties {
            properties.insert(key, NodeProperty::try_from(property)?);
        }

        let mut id_strategy = Vec::with_capacity(node_description_proto.id_strategy.len());
        for strategy in node_description_proto.id_strategy {
            id_strategy.push(IdStrategy::try_from(strategy)?);
        }

        Ok(NodeDescription {
            properties,
            node_key: node_description_proto.node_key,
            node_type: node_description_proto.node_type,
            id_strategy,
        })
    }
}

impl From<NodeDescription> for NodeDescriptionProto {
    fn from(node_description: NodeDescription) -> Self {
        let mut properties = HashMap::with_capacity(node_description.properties.len());
        for (key, property) in node_description.properties {
            properties.insert(key, NodePropertyProto::from(property));
        }

        let mut id_strategy = Vec::with_capacity(node_description.id_strategy.len());
        for strategy in node_description.id_strategy {
            id_strategy.push(IdStrategyProto::from(strategy));
        }

        NodeDescriptionProto {
            properties,
            node_key: node_description.node_key,
            node_type: node_description.node_type,
            id_strategy,
        }
    }
}

impl type_url::TypeUrl for NodeDescription {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.NodeDescription";
}

impl serde_impl::ProtobufSerializable for NodeDescription {
    type ProtobufMessage = NodeDescriptionProto;
}

//
// IdentifiedNode
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdentifiedNode {
    pub properties: HashMap<String, NodeProperty>,
    pub uid: Uid,
    pub node_type: String,
}

impl IdentifiedNode {
    pub fn from(n: IdentifiedNode, uid: Uid) -> Self {
        Self {
            uid,
            properties: n.properties,
            node_type: n.node_type,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        debug_assert_eq!(self.node_type, other.node_type);
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }
}

impl TryFrom<IdentifiedNodeProto> for IdentifiedNode {
    type Error = SerDeError;

    fn try_from(identified_node_proto: IdentifiedNodeProto) -> Result<Self, Self::Error> {
        let uid = identified_node_proto
            .uid
            .ok_or(SerDeError::MissingField("IdentifiedNode.uid"))?
            .try_into()?;

        let mut properties = HashMap::with_capacity(identified_node_proto.properties.len());
        for (key, property) in identified_node_proto.properties {
            properties.insert(key, NodeProperty::try_from(property)?);
        }

        Ok(IdentifiedNode {
            properties,
            uid,
            node_type: identified_node_proto.node_type,
        })
    }
}

impl From<IdentifiedNode> for IdentifiedNodeProto {
    fn from(identified_node: IdentifiedNode) -> Self {
        let mut properties = HashMap::with_capacity(identified_node.properties.len());
        for (key, property) in identified_node.properties {
            properties.insert(key, NodePropertyProto::from(property));
        }

        IdentifiedNodeProto {
            properties,
            uid: Some(identified_node.uid.into()),
            node_type: identified_node.node_type,
        }
    }
}

impl type_url::TypeUrl for IdentifiedNode {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdentifiedNode";
}

impl serde_impl::ProtobufSerializable for IdentifiedNode {
    type ProtobufMessage = IdentifiedNodeProto;
}

//
// Edge
//

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct Edge {
    pub to_node_key: String,
    pub from_node_key: String,
    pub edge_name: String,
}

impl From<EdgeProto> for Edge {
    fn from(edge_proto: EdgeProto) -> Self {
        Edge {
            to_node_key: edge_proto.to_node_key,
            from_node_key: edge_proto.from_node_key,
            edge_name: edge_proto.edge_name,
        }
    }
}

impl From<Edge> for EdgeProto {
    fn from(edge: Edge) -> Self {
        EdgeProto {
            from_node_key: edge.from_node_key,
            to_node_key: edge.to_node_key,
            edge_name: edge.edge_name,
        }
    }
}

impl type_url::TypeUrl for Edge {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.Edge";
}

impl serde_impl::ProtobufSerializable for Edge {
    type ProtobufMessage = EdgeProto;
}

//
// IdentifiedEdge
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdentifiedEdge {
    pub from_uid: Uid,
    pub to_uid: Uid,
    pub edge_name: String,
}

impl TryFrom<IdentifiedEdgeProto> for IdentifiedEdge {
    type Error = SerDeError;
    fn try_from(identified_edge_proto: IdentifiedEdgeProto) -> Result<Self, Self::Error> {
        let from_uid = identified_edge_proto
            .from_uid
            .ok_or(SerDeError::MissingField("IdentifiedEdge.from_uid"))?
            .try_into()?;
        let to_uid = identified_edge_proto
            .to_uid
            .ok_or(SerDeError::MissingField("IdentifiedEdge.to_uid"))?
            .try_into()?;

        Ok(IdentifiedEdge {
            from_uid,
            to_uid,
            edge_name: identified_edge_proto.edge_name,
        })
    }
}

impl From<IdentifiedEdge> for IdentifiedEdgeProto {
    fn from(identified_edge: IdentifiedEdge) -> Self {
        IdentifiedEdgeProto {
            from_uid: Some(identified_edge.from_uid.into()),
            to_uid: Some(identified_edge.to_uid.into()),
            edge_name: identified_edge.edge_name,
        }
    }
}

impl type_url::TypeUrl for IdentifiedEdge {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdentifiedEdge";
}

impl serde_impl::ProtobufSerializable for IdentifiedEdge {
    type ProtobufMessage = IdentifiedEdgeProto;
}

//
// Lens
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Lens {
    pub lens_type: String,
    pub lens_name: String,
    pub uid: Option<u64>,
    pub score: Option<u64>,
}

impl From<LensProto> for Lens {
    fn from(lens_proto: LensProto) -> Self {
        Lens {
            lens_type: lens_proto.lens_type,
            lens_name: lens_proto.lens_name,
            uid: lens_proto.uid,
            score: lens_proto.score,
        }
    }
}

impl From<Lens> for LensProto {
    fn from(lens: Lens) -> Self {
        LensProto {
            lens_type: lens.lens_type,
            lens_name: lens.lens_name,
            uid: lens.uid,
            score: lens.score,
        }
    }
}

impl type_url::TypeUrl for Lens {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.Lens";
}

impl serde_impl::ProtobufSerializable for Lens {
    type ProtobufMessage = LensProto;
}

//
// EdgeList
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EdgeList {
    pub edges: Vec<Edge>,
}

impl From<EdgeListProto> for EdgeList {
    fn from(edge_list_proto: EdgeListProto) -> Self {
        let mut edges = Vec::with_capacity(edge_list_proto.edges.len());
        for edge in edge_list_proto.edges {
            edges.push(Edge::from(edge));
        }

        EdgeList { edges }
    }
}

impl From<EdgeList> for EdgeListProto {
    fn from(edge_list: EdgeList) -> Self {
        let mut edges = Vec::with_capacity(edge_list.edges.len());
        for edge in edge_list.edges {
            edges.push(EdgeProto::from(edge));
        }

        EdgeListProto { edges }
    }
}

impl type_url::TypeUrl for EdgeList {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.EdgeList";
}

impl serde_impl::ProtobufSerializable for EdgeList {
    type ProtobufMessage = EdgeListProto;
}

//
// IdentifiedEdgeList
//

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IdentifiedEdgeList {
    pub edges: Vec<IdentifiedEdge>,
}

impl TryFrom<IdentifiedEdgeListProto> for IdentifiedEdgeList {
    type Error = SerDeError;
    fn try_from(identified_edge_list_proto: IdentifiedEdgeListProto) -> Result<Self, Self::Error> {
        let mut edges = Vec::with_capacity(identified_edge_list_proto.edges.len());
        for edge in identified_edge_list_proto.edges {
            edges.push(IdentifiedEdge::try_from(edge)?);
        }

        Ok(IdentifiedEdgeList { edges })
    }
}

impl From<IdentifiedEdgeList> for IdentifiedEdgeListProto {
    fn from(identified_edge_list: IdentifiedEdgeList) -> Self {
        let mut edges = Vec::with_capacity(identified_edge_list.edges.len());
        for edge in identified_edge_list.edges {
            edges.push(IdentifiedEdgeProto::from(edge));
        }

        IdentifiedEdgeListProto { edges }
    }
}

impl type_url::TypeUrl for IdentifiedEdgeList {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdentifiedEdgeList";
}

impl serde_impl::ProtobufSerializable for IdentifiedEdgeList {
    type ProtobufMessage = IdentifiedEdgeListProto;
}

//
// GraphDescription
//

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct GraphDescription {
    pub nodes: HashMap<String, NodeDescription>,
    pub edges: HashMap<String, EdgeList>,
}

impl GraphDescription {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: impl Into<NodeDescription>) {
        let node = node.into();
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        to_node_key: impl Into<String>,
    ) {
        let from_node_key = from_node_key.into();
        let to_node_key = to_node_key.into();
        let edge_name = edge_name.into();

        assert_ne!(from_node_key, to_node_key);

        let edge = Edge {
            from_node_key: from_node_key.clone(),
            to_node_key,
            edge_name,
        };

        let edge_list: &mut Vec<Edge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| EdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.to_node_key.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl TryFrom<GraphDescriptionProto> for GraphDescription {
    type Error = SerDeError;

    fn try_from(graph_description_proto: GraphDescriptionProto) -> Result<Self, Self::Error> {
        let mut nodes = HashMap::with_capacity(graph_description_proto.nodes.len());
        for (key, node_description) in graph_description_proto.nodes {
            nodes.insert(key, NodeDescription::try_from(node_description)?);
        }

        let mut edges = HashMap::with_capacity(graph_description_proto.edges.len());
        for (key, edge_list) in graph_description_proto.edges {
            edges.insert(key, EdgeList::from(edge_list));
        }

        Ok(GraphDescription { nodes, edges })
    }
}

impl From<GraphDescription> for GraphDescriptionProto {
    fn from(graph_description: GraphDescription) -> Self {
        let mut nodes = HashMap::with_capacity(graph_description.nodes.len());
        for (key, node_description) in graph_description.nodes {
            nodes.insert(key, NodeDescriptionProto::from(node_description));
        }

        let mut edges = HashMap::with_capacity(graph_description.edges.len());
        for (key, edge_list) in graph_description.edges {
            edges.insert(key, EdgeListProto::from(edge_list));
        }

        GraphDescriptionProto { nodes, edges }
    }
}

impl type_url::TypeUrl for GraphDescription {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.GraphDescription";
}

impl serde_impl::ProtobufSerializable for GraphDescription {
    type ProtobufMessage = GraphDescriptionProto;
}

//
// IdentifiedGraph
//

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct IdentifiedGraph {
    pub nodes: HashMap<Uid, IdentifiedNode>,
    pub edges: HashMap<Uid, IdentifiedEdgeList>,
}

impl IdentifiedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: IdentifiedNode) {
        match self.nodes.get_mut(&node.uid) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.uid, node);
            }
        };
    }

    pub fn add_edge(&mut self, edge_name: impl Into<String>, from_uid: Uid, to_uid: Uid) {
        assert_ne!(from_uid, to_uid);

        let edge_name = edge_name.into();
        let edge = IdentifiedEdge {
            from_uid,
            to_uid,
            edge_name,
        };

        let edge_list: &mut Vec<_> = &mut self
            .edges
            .entry(from_uid)
            .or_insert_with(|| IdentifiedEdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(edge.edge_name.clone(), edge.from_uid, edge.to_uid);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl TryFrom<IdentifiedGraphProto> for IdentifiedGraph {
    type Error = SerDeError;

    fn try_from(identified_graph_proto: IdentifiedGraphProto) -> Result<Self, Self::Error> {
        let mut nodes = HashMap::with_capacity(identified_graph_proto.nodes.len());
        for (key, identified_node) in identified_graph_proto.nodes {
            nodes.insert(
                Uid::from_u64(key).unwrap(),
                IdentifiedNode::try_from(identified_node)?,
            );
        }

        let mut edges = HashMap::with_capacity(identified_graph_proto.edges.len());
        for (key, edge_list) in identified_graph_proto.edges {
            edges.insert(
                Uid::from_u64(key).unwrap(),
                IdentifiedEdgeList::try_from(edge_list)?,
            );
        }

        Ok(IdentifiedGraph { nodes, edges })
    }
}

impl From<IdentifiedGraph> for IdentifiedGraphProto {
    fn from(identified_graph: IdentifiedGraph) -> Self {
        let mut nodes = HashMap::with_capacity(identified_graph.nodes.len());
        for (key, identified_node) in identified_graph.nodes {
            nodes.insert(key.as_u64(), IdentifiedNodeProto::from(identified_node));
        }

        let mut edges = HashMap::with_capacity(identified_graph.edges.len());
        for (key, edge_list) in identified_graph.edges {
            edges.insert(key.as_u64(), IdentifiedEdgeListProto::from(edge_list));
        }

        IdentifiedGraphProto { nodes, edges }
    }
}

impl type_url::TypeUrl for IdentifiedGraph {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdentifiedGraph";
}

impl serde_impl::ProtobufSerializable for IdentifiedGraph {
    type ProtobufMessage = IdentifiedGraphProto;
}

#[cfg(test)]
pub mod test {
    // TODO: refactor these tests to use proptest instead of quickcheck

    use std::{
        collections::HashMap,
        hash::Hasher,
    };

    use quickcheck::{
        Arbitrary,
        Gen,
    };
    use quickcheck_macros::quickcheck;
    use tracing_subscriber::{
        EnvFilter,
        FmtSubscriber,
    };

    use super::*;

    impl Arbitrary for IncrementOnlyIntProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: i64::arbitrary(g),
            }
        }
    }

    impl Arbitrary for DecrementOnlyIntProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: i64::arbitrary(g),
            }
        }
    }

    impl Arbitrary for ImmutableIntProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: i64::arbitrary(g),
            }
        }
    }

    impl Arbitrary for IncrementOnlyUintProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: u64::arbitrary(g),
            }
        }
    }

    impl Arbitrary for DecrementOnlyUintProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: u64::arbitrary(g),
            }
        }
    }

    impl Arbitrary for ImmutableUintProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: u64::arbitrary(g),
            }
        }
    }

    impl Arbitrary for ImmutableStrProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: String::arbitrary(g),
            }
        }
    }

    impl Arbitrary for Property {
        fn arbitrary(g: &mut Gen) -> Self {
            let props = &[
                Property::IncrementOnlyIntProp(IncrementOnlyIntProp::arbitrary(g)),
                Property::DecrementOnlyIntProp(DecrementOnlyIntProp::arbitrary(g)),
                Property::ImmutableIntProp(ImmutableIntProp::arbitrary(g)),
                Property::IncrementOnlyUintProp(IncrementOnlyUintProp::arbitrary(g)),
                Property::DecrementOnlyUintProp(DecrementOnlyUintProp::arbitrary(g)),
                Property::ImmutableUintProp(ImmutableUintProp::arbitrary(g)),
                Property::ImmutableStrProp(ImmutableStrProp::arbitrary(g)),
            ];
            g.choose(props).unwrap().clone()
        }
    }

    impl Arbitrary for NodeProperty {
        fn arbitrary(g: &mut Gen) -> Self {
            NodeProperty {
                property: Property::arbitrary(g),
            }
        }
    }

    fn hash(bytes: &[impl AsRef<[u8]>]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for _bytes in bytes {
            hasher.write(_bytes.as_ref());
        }
        hasher.finish() as u64
    }

    fn choice<T: Clone>(bytes: impl AsRef<[u8]>, choices: &[T]) -> T {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(bytes.as_ref());
        let choice_index = (hasher.finish() as usize) % choices.len();
        choices[choice_index].clone()
    }

    fn choose_property(uid: Uid, property_name: &str, g: &mut Gen) -> NodeProperty {
        let s = format!("{}{}", uid.as_u64(), property_name);

        let props = &[
            Property::IncrementOnlyIntProp(IncrementOnlyIntProp::arbitrary(g)),
            Property::DecrementOnlyIntProp(DecrementOnlyIntProp::arbitrary(g)),
            Property::IncrementOnlyUintProp(IncrementOnlyUintProp::arbitrary(g)),
            Property::DecrementOnlyUintProp(DecrementOnlyUintProp::arbitrary(g)),
            Property::ImmutableIntProp(ImmutableIntProp::from(hash(
                &[&uid.as_u64().to_string(), property_name][..],
            ) as i64)),
            Property::ImmutableUintProp(ImmutableUintProp::from(hash(
                &[&uid.as_u64().to_string(), property_name][..],
            ))),
            Property::ImmutableStrProp(ImmutableStrProp::from(s)),
        ];
        let p: Property = choice(&uid.as_u64().to_string(), props);
        p.into()
    }

    impl Arbitrary for IdentifiedNode {
        fn arbitrary(g: &mut Gen) -> Self {
            let uids = &[
                Uid::from_u64(123).unwrap(),
                Uid::from_u64(124).unwrap(),
                Uid::from_u64(125).unwrap(),
                Uid::from_u64(126).unwrap(),
            ];
            let uid = g.choose(uids).unwrap().clone();

            let node_types = &["Process", "File", "IpAddress"];
            let node_type = choice(&uid.as_u64().to_string(), node_types);
            let mut properties = HashMap::new();
            let property_names: Vec<String> = Vec::arbitrary(g);

            for property_name in property_names {
                let property = choose_property(uid, &property_name, g);
                properties.insert(property_name.to_owned(), property);
            }

            IdentifiedNode {
                uid,
                node_type: node_type.to_owned(),
                properties,
            }
        }
    }

    fn init_test_env() {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    }

    #[quickcheck]
    fn test_merge_str(x: ImmutableStrProp, y: ImmutableStrProp) {
        init_test_env();
        let original = x;
        let mut x = original.clone();
        x.merge_property(&y);
        assert_eq!(original, x);
    }

    #[quickcheck]
    fn test_merge_immutable_int(mut x: ImmutableIntProp, y: ImmutableIntProp) {
        init_test_env();
        let original = x.clone();
        x.merge_property(&y);
        assert_eq!(x, original);
    }

    #[quickcheck]
    fn test_merge_immutable_uint(mut x: ImmutableUintProp, y: ImmutableUintProp) {
        init_test_env();
        let original = x.clone();
        x.merge_property(&y);
        assert_eq!(x, original);
    }

    #[quickcheck]
    fn test_merge_uint_max(mut x: IncrementOnlyUintProp, y: IncrementOnlyUintProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x.clone(), std::cmp::max(x, y));
    }

    #[quickcheck]
    fn test_merge_int_max(mut x: IncrementOnlyIntProp, y: IncrementOnlyIntProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x.clone(), std::cmp::max(x, y));
    }

    #[quickcheck]
    fn test_merge_uint_min(mut x: DecrementOnlyUintProp, y: DecrementOnlyUintProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x.clone(), std::cmp::min(x, y));
    }

    #[quickcheck]
    fn test_merge_int_min(mut x: DecrementOnlyIntProp, y: DecrementOnlyIntProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x.clone(), std::cmp::min(x, y));
    }

    #[quickcheck]
    fn test_merge_incr_uint_commutative(mut properties: Vec<IncrementOnlyUintProp>) {
        init_test_env();
        if properties.is_empty() {
            return;
        }
        properties.sort_unstable();
        let max_value = properties.iter().max().unwrap().to_owned();
        let mut first_x = properties[0].clone();
        for property in properties.iter() {
            first_x.merge_property(property)
        }

        let properties: Vec<_> = properties.into_iter().rev().collect();
        let mut first_y = properties[0].clone();
        for property in properties.iter() {
            first_y.merge_property(property)
        }
        assert_eq!(first_x, first_y);
        assert_eq!(first_x, max_value);
    }

    #[quickcheck]
    fn test_merge_identified_node(mut node_0: IdentifiedNode, node_1: IdentifiedNode) {
        if node_0.uid != node_1.uid {
            return;
        }
        // let original = node_0.clone();
        node_0.merge(&node_1);

        // for (_o_pred_name, o_pred_val) in original.iter() {
        //     let mut copy = o_pred_val.clone();
        // }
    }
}
