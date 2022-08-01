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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyName {
    pub value: String,
}

impl std::fmt::Display for PropertyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PropertyName {
    pub fn new_unchecked(value: String) -> Self {
        // todo: debug assertion
        Self { value }
    }
}

impl TryFrom<&'static str> for PropertyName {
    type Error = SerDeError;
    fn try_from(raw: &'static str) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(SerDeError::InvalidField {
                field_name: "PropertyName",
                assertion: "can not be empty".to_owned(),
            });
        }
        if raw.len() > 32 {
            return Err(SerDeError::InvalidField {
                field_name: "PropertyName",
                assertion: "can not be more than 32 characters".to_owned(),
            });
        }

        Ok(Self {
            value: raw.to_owned(),
        })
    }
}

impl TryFrom<String> for PropertyName {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // todo: Validate this thing
        Ok(PropertyName { value })
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeName {
    pub value: String,
}

impl std::fmt::Display for EdgeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<String> for EdgeName {
    type Error = &'static str;
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err("EdgeName can not be empty");
        }
        if raw.len() > 32 {
            return Err("EdgeName can not be more than 32 characters");
        }

        Ok(Self { value: raw })
    }
}

impl TryFrom<&'static str> for EdgeName {
    type Error = &'static str;
    fn try_from(raw: &'static str) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err("EdgeName can not be empty");
        }
        if raw.len() > 32 {
            return Err("EdgeName can not be more than 32 characters");
        }

        Ok(Self {
            value: raw.to_owned(),
        })
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeType {
    pub value: String,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<&'static str> for NodeType {
    type Error = SerDeError;
    fn try_from(raw: &'static str) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(SerDeError::InvalidField {
                field_name: "EdgeName",
                assertion: "can not be empty".to_owned(),
            });
        }
        if raw.len() > 32 {
            return Err(SerDeError::InvalidField {
                field_name: "EdgeName",
                assertion: "can not be more than 32 characters".to_owned(),
            });
        }

        Ok(Self {
            value: raw.to_owned(),
        })
    }
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

#[derive(Debug, PartialOrd, Ord, Copy, Clone, PartialEq, Eq, Hash)]
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
