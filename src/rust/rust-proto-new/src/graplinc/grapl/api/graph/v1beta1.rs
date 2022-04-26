#![allow(warnings)]


use crate::protobufs::graplinc::grapl::api::graph::v1beta1::{
    IncrementOnlyUintProp as IncrementOnlyUintPropProto,
    DecrementOnlyUintProp as DecrementOnlyUintPropProto,
    ImmutableUintProp as ImmutableUintPropProto,
    IncrementOnlyIntProp as IncrementOnlyIntPropProto,
    DecrementOnlyIntProp as DecrementOnlyIntPropProto,
    ImmutableIntProp as ImmutableIntPropProto,
    ImmutableStrProp as ImmutableStrPropProto,
    NodeProperty as NodePropertyProto
};
use crate::protobufs::graplinc::grapl::api::graph::v1beta1::node_property::Property as PropertyProto;
use crate::SerDeError;


pub struct IncrementOnlyUintProp {
    pub prop: u64,
}

impl TryFrom<IncrementOnlyUintPropProto> for IncrementOnlyUintProp {
    type Error = SerDeError;
    fn try_from(proto: IncrementOnlyUintPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<IncrementOnlyUintProp> for IncrementOnlyUintPropProto {
    fn from(value: IncrementOnlyUintProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub struct DecrementOnlyUintProp {
    pub prop: u64,
}

impl TryFrom<DecrementOnlyUintPropProto> for DecrementOnlyUintProp {
    type Error = SerDeError;
    fn try_from(proto: DecrementOnlyUintPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<DecrementOnlyUintProp> for DecrementOnlyUintPropProto {
    fn from(value: DecrementOnlyUintProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub struct ImmutableUintProp {
    pub prop: u64,
}

impl TryFrom<ImmutableUintPropProto> for ImmutableUintProp {
    type Error = SerDeError;
    fn try_from(proto: ImmutableUintPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<ImmutableUintProp> for ImmutableUintPropProto {
    fn from(value: ImmutableUintProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub struct IncrementOnlyIntProp {
    pub prop: i64,
}

impl TryFrom<IncrementOnlyIntPropProto> for IncrementOnlyIntProp {
    type Error = SerDeError;
    fn try_from(proto: IncrementOnlyIntPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<IncrementOnlyIntProp> for IncrementOnlyIntPropProto {
    fn from(value: IncrementOnlyIntProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub struct DecrementOnlyIntProp {
    pub prop: i64,
}

impl TryFrom<DecrementOnlyIntPropProto> for DecrementOnlyIntProp {
    type Error = SerDeError;
    fn try_from(proto: DecrementOnlyIntPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<DecrementOnlyIntProp> for DecrementOnlyIntPropProto {
    fn from(value: DecrementOnlyIntProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub struct ImmutableIntProp {
    pub prop: i64,
}

impl TryFrom<ImmutableIntPropProto> for ImmutableIntProp {
    type Error = SerDeError;
    fn try_from(proto: ImmutableIntPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<ImmutableIntProp> for ImmutableIntPropProto {
    fn from(value: ImmutableIntProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub struct ImmutableStrProp {
    pub prop: String,
}

impl TryFrom<ImmutableStrPropProto> for ImmutableStrProp {
    type Error = SerDeError;
    fn try_from(proto: ImmutableStrPropProto) -> Result<Self, Self::Error> {
        Ok(Self {
            prop: proto.prop,
        })
    }
}

impl From<ImmutableStrProp> for ImmutableStrPropProto {
    fn from(value: ImmutableStrProp) -> Self {
        Self {
            prop: value.prop,
        }
    }
}

pub enum Property {
    IncrementOnlyUint(IncrementOnlyUintProp),
    DecrementOnlyUint(DecrementOnlyUintProp),
    ImmutableUint(ImmutableUintProp),
    IncrementOnlyInt(IncrementOnlyIntProp),
    DecrementOnlyInt(DecrementOnlyIntProp),
    ImmutableInt(ImmutableIntProp),
    ImmutableStr(ImmutableStrProp),
}

impl TryFrom<PropertyProto> for Property {
    type Error = SerDeError;
    fn try_from(proto: PropertyProto) -> Result<Self, Self::Error> {
        match proto {
            PropertyProto::IncrementOnlyUint(proto) => Ok(Property::IncrementOnlyUint(
                IncrementOnlyUintProp::try_from(proto)?,
            )),
            PropertyProto::DecrementOnlyUint(proto) => Ok(Property::DecrementOnlyUint(
                DecrementOnlyUintProp::try_from(proto)?,
            )),
            PropertyProto::ImmutableUint(proto) => Ok(Property::ImmutableUint(
                ImmutableUintProp::try_from(proto)?,
            )),
            PropertyProto::IncrementOnlyInt(proto) => Ok(Property::IncrementOnlyInt(
                IncrementOnlyIntProp::try_from(proto)?,
            )),
            PropertyProto::DecrementOnlyInt(proto) => Ok(Property::DecrementOnlyInt(
                DecrementOnlyIntProp::try_from(proto)?,
            )),
            PropertyProto::ImmutableInt(proto) => Ok(Property::ImmutableInt(
                ImmutableIntProp::try_from(proto)?,
            )),
            PropertyProto::ImmutableStr(proto) => Ok(Property::ImmutableStr(
                ImmutableStrProp::try_from(proto)?,
            )),
        }
    }
}

impl From<Property> for PropertyProto {
    fn from(property: Property) -> PropertyProto {
        match property {
            Property::IncrementOnlyUint(value) => PropertyProto::IncrementOnlyUint(
                IncrementOnlyUintPropProto::from(value),
            ),
            Property::DecrementOnlyUint(value) => PropertyProto::DecrementOnlyUint(
                DecrementOnlyUintPropProto::from(value),
            ),
            Property::ImmutableUint(value) => PropertyProto::ImmutableUint(
                ImmutableUintPropProto::from(value),
            ),
            Property::IncrementOnlyInt(value) => PropertyProto::IncrementOnlyInt(
                IncrementOnlyIntPropProto::from(value),
            ),
            Property::DecrementOnlyInt(value) => PropertyProto::DecrementOnlyInt(
                DecrementOnlyIntPropProto::from(value),
            ),
            Property::ImmutableInt(value) => PropertyProto::ImmutableInt(
                ImmutableIntPropProto::from(value),
            ),
            Property::ImmutableStr(value) => PropertyProto::ImmutableStr(
                ImmutableStrPropProto::from(value),
            ),
        }
    }
}

pub struct NodeProperty {
    pub property: Property,
}

impl TryFrom<NodePropertyProto> for NodeProperty {
    type Error = SerDeError;
    fn try_from(proto: NodePropertyProto) -> Result<Self, Self::Error> {
        let property = proto.property.ok_or(SerDeError::MissingField("property"))?
            .try_into()?;
        Ok(Self {
            property,
        })
    }
}

impl From<NodeProperty> for NodePropertyProto {
    fn from(property: NodeProperty) -> NodePropertyProto {
        NodePropertyProto {
            property: Some(property.property.into()),
        }
    }
}