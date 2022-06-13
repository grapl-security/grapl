use crate::{
    protobufs::graplinc::grapl::common::v1beta1::{
        EdgeName as EdgeNameProto,
        NodeType as NodeTypeProto,
        PropertyName as PropertyNameProto,
        Uid as UidProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyName {
    pub value: String,
}

impl TryFrom<PropertyNameProto> for PropertyName {
    type Error = SerDeError;
    fn try_from(proto: PropertyNameProto) -> Result<Self, Self::Error> {
        Ok(Self { value: proto.value })
    }
}

impl From<PropertyName> for PropertyNameProto {
    fn from(value: PropertyName) -> Self {
        Self { value: value.value }
    }
}

impl serde_impl::ProtobufSerializable for PropertyName {
    type ProtobufMessage = PropertyNameProto;
}

impl type_url::TypeUrl for PropertyName {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.common.v1beta1.PropertyName";
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeName {
    pub value: String,
}

impl TryFrom<EdgeNameProto> for EdgeName {
    type Error = SerDeError;
    fn try_from(proto: EdgeNameProto) -> Result<Self, Self::Error> {
        Ok(Self { value: proto.value })
    }
}

impl From<EdgeName> for EdgeNameProto {
    fn from(value: EdgeName) -> Self {
        Self { value: value.value }
    }
}

impl serde_impl::ProtobufSerializable for EdgeName {
    type ProtobufMessage = EdgeNameProto;
}

impl type_url::TypeUrl for EdgeName {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.common.v1beta1.EdgeName";
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeType {
    pub value: String,
}

impl TryFrom<NodeTypeProto> for NodeType {
    type Error = SerDeError;
    fn try_from(proto: NodeTypeProto) -> Result<Self, Self::Error> {
        Ok(Self { value: proto.value })
    }
}

impl From<NodeType> for NodeTypeProto {
    fn from(value: NodeType) -> Self {
        Self { value: value.value }
    }
}

impl serde_impl::ProtobufSerializable for NodeType {
    type ProtobufMessage = NodeTypeProto;
}

impl type_url::TypeUrl for NodeType {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.common.v1beta1.NodeType";
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Uid {
    value: u64,
}

impl Uid {
    pub fn from_i64(value: i64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Self::from_u64(value as u64)
        }
    }
    pub fn from_u64(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self { value })
        }
    }
    pub fn as_i64(&self) -> i64 {
        self.value as i64
    }
    pub fn as_u64(&self) -> u64 {
        self.value
    }
}

impl TryFrom<UidProto> for Uid {
    type Error = SerDeError;
    fn try_from(proto: UidProto) -> Result<Self, Self::Error> {
        Ok(Self { value: proto.value })
    }
}

impl From<Uid> for UidProto {
    fn from(value: Uid) -> Self {
        Self { value: value.value }
    }
}

impl serde_impl::ProtobufSerializable for Uid {
    type ProtobufMessage = UidProto;
}

impl type_url::TypeUrl for Uid {
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.common.v1beta1.Uid";
}
