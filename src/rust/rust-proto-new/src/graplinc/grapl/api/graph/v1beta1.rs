use std::collections::HashMap;

use bytes::{
    Buf,
    Bytes,
    BytesMut,
};
use prost::Message;

use crate::{
    protobufs::graplinc::grapl::api::graph::v1beta1::{
        DecrementOnlyIntProp as DecrementOnlyIntPropProto,
        DecrementOnlyUintProp as DecrementOnlyUintPropProto,
        Edge as EdgeProto,
        EdgeList as EdgeListProto,
        GraphDescription as GraphDescriptionProto,
        IdStrategy as IdStrategyProto,
        IdentifiedGraph as IdentifiedGraphProto,
        IdentifiedNode as IdentifiedNodeProto,
        ImmutableIntProp as ImmutableIntPropProto,
        ImmutableStrProp as ImmutableStrPropProto,
        ImmutableUintProp as ImmutableUintPropProto,
        IncrementOnlyIntProp as IncrementOnlyIntPropProto,
        IncrementOnlyUintProp as IncrementOnlyUintPropProto,
        MergedEdge as MergedEdgeProto,
        MergedEdgeList as MergedEdgeListProto,
        MergedGraph as MergedGraphProto,
        MergedNode as MergedNodeProto,
        NodeDescription as NodeDescriptionProto,
        NodeProperty as NodePropertyProto,
        Session as SessionProto,
        Static as StaticProto,
    },
    type_url,
    SerDe,
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

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for Session {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let session_proto = SessionProto::from(self);
        let mut buf = BytesMut::with_capacity(session_proto.encoded_len());
        session_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let session_proto: SessionProto = Message::decode(buf)?;
        Ok(session_proto.into())
    }
}

//
// Static
//

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for Static {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let static_proto = StaticProto::from(self);
        let mut buf = BytesMut::with_capacity(static_proto.encoded_len());
        static_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let static_proto: StaticProto = Message::decode(buf)?;
        Ok(static_proto.into())
    }
}

//
// IdStrategy
//

#[derive(Debug, PartialEq, Clone)]
pub enum Strategy {
    Session(Session),
    Static(Static),
}

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for IdStrategy {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let id_strategy_proto = IdStrategyProto::from(self);
        let mut buf = BytesMut::with_capacity(id_strategy_proto.encoded_len());
        id_strategy_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let id_strategy_proto: IdStrategyProto = Message::decode(buf)?;
        id_strategy_proto.try_into()
    }
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
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

impl SerDe for IncrementOnlyUintProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let increment_only_uint_prop_proto = IncrementOnlyUintPropProto::from(self);
        let mut buf = BytesMut::with_capacity(increment_only_uint_prop_proto.encoded_len());
        increment_only_uint_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let increment_only_uint_prop_proto: IncrementOnlyUintPropProto = Message::decode(buf)?;
        Ok(increment_only_uint_prop_proto.into())
    }
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
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

impl SerDe for ImmutableUintProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let immutable_uint_prop_proto = ImmutableUintPropProto::from(self);
        let mut buf = BytesMut::with_capacity(immutable_uint_prop_proto.encoded_len());
        immutable_uint_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let immutable_uint_prop_proto: ImmutableUintPropProto = Message::decode(buf)?;
        Ok(immutable_uint_prop_proto.into())
    }
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
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

impl SerDe for DecrementOnlyUintProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let decrement_only_uint_prop_proto = DecrementOnlyUintPropProto::from(self);
        let mut buf = BytesMut::with_capacity(decrement_only_uint_prop_proto.encoded_len());
        decrement_only_uint_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let decrement_only_uint_prop_proto: DecrementOnlyUintPropProto = Message::decode(buf)?;
        Ok(decrement_only_uint_prop_proto.into())
    }
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
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

impl SerDe for IncrementOnlyIntProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let increment_only_int_prop_proto = IncrementOnlyIntPropProto::from(self);
        let mut buf = BytesMut::with_capacity(increment_only_int_prop_proto.encoded_len());
        increment_only_int_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let increment_only_int_prop_proto: IncrementOnlyIntPropProto = Message::decode(buf)?;
        Ok(increment_only_int_prop_proto.into())
    }
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
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

impl SerDe for DecrementOnlyIntProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let decrement_only_int_prop_proto = DecrementOnlyIntPropProto::from(self);
        let mut buf = BytesMut::with_capacity(decrement_only_int_prop_proto.encoded_len());
        decrement_only_int_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let decrement_only_int_prop_proto: DecrementOnlyIntPropProto = Message::decode(buf)?;
        Ok(decrement_only_int_prop_proto.into())
    }
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
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

impl SerDe for ImmutableIntProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let immutable_int_prop_proto = ImmutableIntPropProto::from(self);
        let mut buf = BytesMut::with_capacity(immutable_int_prop_proto.encoded_len());
        immutable_int_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let immutable_int_prop_proto: ImmutableIntPropProto = Message::decode(buf)?;
        Ok(immutable_int_prop_proto.into())
    }
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

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for ImmutableStrProp {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let immutable_str_prop_proto = ImmutableStrPropProto::from(self);
        let mut buf = BytesMut::with_capacity(immutable_str_prop_proto.encoded_len());
        immutable_str_prop_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let immutable_str_prop_proto: ImmutableStrPropProto = Message::decode(buf)?;
        Ok(immutable_str_prop_proto.into())
    }
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

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for NodeProperty {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let node_property_proto = NodePropertyProto::from(self);
        let mut buf = BytesMut::with_capacity(node_property_proto.encoded_len());
        node_property_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let node_property_proto: NodePropertyProto = Message::decode(buf)?;
        node_property_proto.try_into()
    }
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

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for NodeDescription {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let node_description_proto = NodeDescriptionProto::from(self);
        let mut buf = BytesMut::with_capacity(node_description_proto.encoded_len());
        node_description_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let node_description_proto: NodeDescriptionProto = Message::decode(buf)?;
        node_description_proto.try_into()
    }
}

//
// IdentifiedNode
//

#[derive(Debug, PartialEq, Clone)]
pub struct IdentifiedNode {
    pub properties: HashMap<String, NodeProperty>,
    pub node_key: String,
    pub node_type: String,
}

impl IdentifiedNode {
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

    pub fn get_cache_identities_for_predicates(&self) -> Vec<Vec<u8>> {
        let mut predicate_cache_identities = Vec::with_capacity(self.properties.len());

        for (key, prop) in &self.properties {
            let prop_value = prop.property.to_string();

            predicate_cache_identities.push(format!("{}:{}:{}", &self.node_key, key, prop_value));
        }

        predicate_cache_identities
            .into_iter()
            .map(|item| item.into_bytes())
            .collect()
    }

    pub fn into(self, uid: u64) -> MergedNode {
        MergedNode {
            uid,
            properties: self.properties,
            node_key: self.node_key,
            node_type: self.node_type,
        }
    }
    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

impl TryFrom<IdentifiedNodeProto> for IdentifiedNode {
    type Error = SerDeError;

    fn try_from(identified_node_proto: IdentifiedNodeProto) -> Result<Self, Self::Error> {
        let mut properties = HashMap::with_capacity(identified_node_proto.properties.len());
        for (key, property) in identified_node_proto.properties {
            properties.insert(key, NodeProperty::try_from(property)?);
        }

        Ok(IdentifiedNode {
            properties,
            node_key: identified_node_proto.node_key,
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
            node_key: identified_node.node_key,
            node_type: identified_node.node_type,
        }
    }
}

impl type_url::TypeUrl for IdentifiedNode {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdentifiedNode";
}

impl SerDe for IdentifiedNode {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let identified_node_proto = IdentifiedNodeProto::from(self);
        let mut buf = BytesMut::with_capacity(identified_node_proto.encoded_len());
        identified_node_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let identified_node_proto: IdentifiedNodeProto = Message::decode(buf)?;
        identified_node_proto.try_into()
    }
}

impl From<NodeDescription> for IdentifiedNode {
    fn from(n: NodeDescription) -> Self {
        IdentifiedNode {
            properties: n.properties,
            node_key: n.node_key,
            node_type: n.node_type,
        }
    }
}

//
// MergedNode
//

#[derive(Debug, PartialEq, Clone)]
pub struct MergedNode {
    pub properties: HashMap<String, NodeProperty>,
    pub uid: u64,
    pub node_key: String,
    pub node_type: String,
}

impl MergedNode {
    pub fn from(n: IdentifiedNode, uid: u64) -> Self {
        Self {
            uid,
            properties: n.properties,
            node_key: n.node_key,
            node_type: n.node_type,
        }
    }

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

impl TryFrom<MergedNodeProto> for MergedNode {
    type Error = SerDeError;

    fn try_from(merged_node_proto: MergedNodeProto) -> Result<Self, Self::Error> {
        let mut properties = HashMap::with_capacity(merged_node_proto.properties.len());
        for (key, property) in merged_node_proto.properties {
            properties.insert(key, NodeProperty::try_from(property)?);
        }

        Ok(MergedNode {
            properties,
            uid: merged_node_proto.uid,
            node_key: merged_node_proto.node_key,
            node_type: merged_node_proto.node_type,
        })
    }
}

impl From<MergedNode> for MergedNodeProto {
    fn from(merged_node: MergedNode) -> Self {
        let mut properties = HashMap::with_capacity(merged_node.properties.len());
        for (key, property) in merged_node.properties {
            properties.insert(key, NodePropertyProto::from(property));
        }

        MergedNodeProto {
            properties,
            uid: merged_node.uid,
            node_key: merged_node.node_key,
            node_type: merged_node.node_type,
        }
    }
}

impl type_url::TypeUrl for MergedNode {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.MergedNode";
}

impl SerDe for MergedNode {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let merged_node_proto = MergedNodeProto::from(self);
        let mut buf = BytesMut::with_capacity(merged_node_proto.encoded_len());
        merged_node_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let merged_node_proto: MergedNodeProto = Message::decode(buf)?;
        merged_node_proto.try_into()
    }
}

//
// Edge
//

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for Edge {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let edge_proto = EdgeProto::from(self);
        let mut buf = BytesMut::with_capacity(edge_proto.encoded_len());
        edge_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let edge_proto: EdgeProto = Message::decode(buf)?;
        Ok(edge_proto.into())
    }
}

//
// MergedEdge
//

#[derive(Debug, PartialEq, Clone)]
pub struct MergedEdge {
    pub from_uid: String,
    pub from_node_key: String,
    pub to_uid: String,
    pub to_node_key: String,
    pub edge_name: String,
}

impl From<MergedEdgeProto> for MergedEdge {
    fn from(merged_edge_proto: MergedEdgeProto) -> Self {
        MergedEdge {
            from_uid: merged_edge_proto.from_uid,
            from_node_key: merged_edge_proto.from_node_key,
            to_uid: merged_edge_proto.to_uid,
            to_node_key: merged_edge_proto.to_node_key,
            edge_name: merged_edge_proto.edge_name,
        }
    }
}

impl From<MergedEdge> for MergedEdgeProto {
    fn from(merged_edge: MergedEdge) -> Self {
        MergedEdgeProto {
            from_uid: merged_edge.from_uid,
            from_node_key: merged_edge.from_node_key,
            to_uid: merged_edge.to_uid,
            to_node_key: merged_edge.to_node_key,
            edge_name: merged_edge.edge_name,
        }
    }
}

impl type_url::TypeUrl for MergedEdge {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.MergedEdge";
}

impl SerDe for MergedEdge {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let merged_edge_proto = MergedEdgeProto::from(self);
        let mut buf = BytesMut::with_capacity(merged_edge_proto.encoded_len());
        merged_edge_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let merged_edge_proto: MergedEdgeProto = Message::decode(buf)?;
        Ok(merged_edge_proto.into())
    }
}

//
// EdgeList
//

#[derive(Debug, PartialEq, Clone)]
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

impl SerDe for EdgeList {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let edge_list_proto = EdgeListProto::from(self);
        let mut buf = BytesMut::with_capacity(edge_list_proto.encoded_len());
        edge_list_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let edge_list_proto: EdgeListProto = Message::decode(buf)?;
        Ok(edge_list_proto.into())
    }
}

//
// MergedEdgeList
//

#[derive(Debug, PartialEq, Clone)]
pub struct MergedEdgeList {
    pub edges: Vec<MergedEdge>,
}

impl From<MergedEdgeListProto> for MergedEdgeList {
    fn from(merged_edge_list_proto: MergedEdgeListProto) -> Self {
        let mut edges = Vec::with_capacity(merged_edge_list_proto.edges.len());
        for edge in merged_edge_list_proto.edges {
            edges.push(MergedEdge::from(edge));
        }

        MergedEdgeList { edges }
    }
}

impl From<MergedEdgeList> for MergedEdgeListProto {
    fn from(merged_edge_list: MergedEdgeList) -> Self {
        let mut edges = Vec::with_capacity(merged_edge_list.edges.len());
        for edge in merged_edge_list.edges {
            edges.push(MergedEdgeProto::from(edge));
        }

        MergedEdgeListProto { edges }
    }
}

impl type_url::TypeUrl for MergedEdgeList {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.MergedEdgeList";
}

impl SerDe for MergedEdgeList {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let merged_edge_list_proto = MergedEdgeListProto::from(self);
        let mut buf = BytesMut::with_capacity(merged_edge_list_proto.encoded_len());
        merged_edge_list_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let merged_edge_list_proto: MergedEdgeListProto = Message::decode(buf)?;
        Ok(merged_edge_list_proto.into())
    }
}

//
// GraphDescription
//

#[derive(Debug, Default, PartialEq, Clone)]
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

impl SerDe for GraphDescription {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let graph_description_proto = GraphDescriptionProto::from(self);
        let mut buf = BytesMut::with_capacity(graph_description_proto.encoded_len());
        graph_description_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let graph_description_proto: GraphDescriptionProto = Message::decode(buf)?;
        graph_description_proto.try_into()
    }
}

//
// IdentifiedGraph
//

#[derive(Debug, Default, PartialEq, Clone)]
pub struct IdentifiedGraph {
    pub nodes: HashMap<String, IdentifiedNode>,
    pub edges: HashMap<String, EdgeList>,
}

impl IdentifiedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: IdentifiedNode) {
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
        assert_ne!(from_node_key, to_node_key);

        let edge_name = edge_name.into();
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

impl TryFrom<IdentifiedGraphProto> for IdentifiedGraph {
    type Error = SerDeError;

    fn try_from(identified_graph_proto: IdentifiedGraphProto) -> Result<Self, Self::Error> {
        let mut nodes = HashMap::with_capacity(identified_graph_proto.nodes.len());
        for (key, identified_node) in identified_graph_proto.nodes {
            nodes.insert(key, IdentifiedNode::try_from(identified_node)?);
        }

        let mut edges = HashMap::with_capacity(identified_graph_proto.edges.len());
        for (key, edge_list) in identified_graph_proto.edges {
            edges.insert(key, EdgeList::from(edge_list));
        }

        Ok(IdentifiedGraph { nodes, edges })
    }
}

impl From<IdentifiedGraph> for IdentifiedGraphProto {
    fn from(identified_graph: IdentifiedGraph) -> Self {
        let mut nodes = HashMap::with_capacity(identified_graph.nodes.len());
        for (key, identified_node) in identified_graph.nodes {
            nodes.insert(key, IdentifiedNodeProto::from(identified_node));
        }

        let mut edges = HashMap::with_capacity(identified_graph.edges.len());
        for (key, edge_list) in identified_graph.edges {
            edges.insert(key, EdgeListProto::from(edge_list));
        }

        IdentifiedGraphProto { nodes, edges }
    }
}

impl type_url::TypeUrl for IdentifiedGraph {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.IdentifiedGraph";
}

impl SerDe for IdentifiedGraph {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let identified_graph_proto = IdentifiedGraphProto::from(self);
        let mut buf = BytesMut::with_capacity(identified_graph_proto.encoded_len());
        identified_graph_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let identified_graph_proto: IdentifiedGraphProto = Message::decode(buf)?;
        identified_graph_proto.try_into()
    }
}

//
// MergedGraph
//

#[derive(Debug, Default, PartialEq, Clone)]
pub struct MergedGraph {
    pub nodes: HashMap<String, MergedNode>,
    pub edges: HashMap<String, MergedEdgeList>,
}

impl MergedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: MergedNode) {
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_merged_edge(&mut self, edge: MergedEdge) {
        let from_node_key = edge.from_node_key.clone();
        let edge_list: &mut Vec<MergedEdge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| MergedEdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        from_uid: impl Into<String>,
        to_node_key: impl Into<String>,
        to_uid: impl Into<String>,
    ) {
        let edge_name = edge_name.into();
        let from_node_key = from_node_key.into();
        let from_uid = from_uid.into();
        let to_node_key = to_node_key.into();
        let to_uid = to_uid.into();
        assert_ne!(from_node_key, to_node_key);
        assert_ne!(from_uid, to_uid);
        let edge = MergedEdge {
            from_node_key: from_node_key.clone(),
            from_uid,
            to_node_key,
            to_uid,
            edge_name,
        };

        let edge_list: &mut Vec<MergedEdge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| MergedEdgeList {
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
                    edge.from_uid.clone(),
                    edge.to_node_key.clone(),
                    edge.to_uid.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl TryFrom<MergedGraphProto> for MergedGraph {
    type Error = SerDeError;

    fn try_from(merged_graph_proto: MergedGraphProto) -> Result<Self, Self::Error> {
        let mut nodes = HashMap::with_capacity(merged_graph_proto.nodes.len());
        for (key, merged_node) in merged_graph_proto.nodes {
            nodes.insert(key, MergedNode::try_from(merged_node)?);
        }

        let mut edges = HashMap::with_capacity(merged_graph_proto.edges.len());
        for (key, merged_edge_list) in merged_graph_proto.edges {
            edges.insert(key, MergedEdgeList::from(merged_edge_list));
        }

        Ok(MergedGraph { nodes, edges })
    }
}

impl From<MergedGraph> for MergedGraphProto {
    fn from(merged_graph: MergedGraph) -> Self {
        let mut nodes = HashMap::with_capacity(merged_graph.nodes.len());
        for (key, merged_node) in merged_graph.nodes {
            nodes.insert(key, MergedNodeProto::from(merged_node));
        }

        let mut edges = HashMap::with_capacity(merged_graph.edges.len());
        for (key, merged_edge_list) in merged_graph.edges {
            edges.insert(key, MergedEdgeListProto::from(merged_edge_list));
        }

        MergedGraphProto { nodes, edges }
    }
}

impl type_url::TypeUrl for MergedGraph {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.graph.v1beta1.MergedGraph";
}

impl SerDe for MergedGraph {
    fn serialize(self) -> Result<Bytes, SerDeError> {
        let merged_graph_proto = MergedGraphProto::from(self);
        let mut buf = BytesMut::with_capacity(merged_graph_proto.encoded_len());
        merged_graph_proto.encode(&mut buf)?;
        Ok(buf.freeze())
    }

    fn deserialize<B>(buf: B) -> Result<Self, SerDeError>
    where
        B: Buf,
        Self: Sized,
    {
        let merged_graph_proto: MergedGraphProto = Message::decode(buf)?;
        merged_graph_proto.try_into()
    }
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

    fn choose_property(node_key: &str, property_name: &str, g: &mut Gen) -> NodeProperty {
        let s = format!("{}{}", node_key, property_name);

        let props = &[
            Property::IncrementOnlyIntProp(IncrementOnlyIntProp::arbitrary(g)),
            Property::DecrementOnlyIntProp(DecrementOnlyIntProp::arbitrary(g)),
            Property::IncrementOnlyUintProp(IncrementOnlyUintProp::arbitrary(g)),
            Property::DecrementOnlyUintProp(DecrementOnlyUintProp::arbitrary(g)),
            Property::ImmutableIntProp(ImmutableIntProp::from(
                hash(&[node_key, property_name]) as i64
            )),
            Property::ImmutableUintProp(ImmutableUintProp::from(hash(&[node_key, property_name]))),
            Property::ImmutableStrProp(ImmutableStrProp::from(s)),
        ];
        let p: Property = choice(node_key, props);
        p.into()
    }

    impl Arbitrary for IdentifiedNode {
        fn arbitrary(g: &mut Gen) -> Self {
            let node_keys = &[
                "c413e25e-9c50-4faf-8e61-f8bfb0e0d18e".to_string(),
                "0d5c9261-2b6e-4094-8de3-b349cb0aa310".to_string(),
                "ed1f73df-f38d-43c0-87b0-5aff06e1f68b".to_string(),
                "6328e956-117e-4f7f-8a5b-c56be1111f43".to_string(),
            ];
            let node_key = g.choose(node_keys).unwrap().clone();

            let node_types = &["Process", "File", "IpAddress"];
            let node_type = choice(&node_key, node_types);
            let mut properties = HashMap::new();
            let property_names: Vec<String> = Vec::arbitrary(g);

            for property_name in property_names {
                let property = choose_property(&node_key, &property_name, g);
                properties.insert(property_name.to_owned(), property);
            }

            IdentifiedNode {
                node_key: node_key.to_owned(),
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
        if node_0.node_key != node_1.node_key {
            return;
        }
        // let original = node_0.clone();
        node_0.merge(&node_1);

        // for (_o_pred_name, o_pred_val) in original.iter() {
        //     let mut copy = o_pred_val.clone();
        // }
    }
}
