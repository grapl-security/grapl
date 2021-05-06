use std::convert::{TryFrom, TryInto};
use graphql_parser::schema::Type;
use crate::errors::CodeGenError;
use crate::constants::{
    STRING,
    UINT,
    INT,
};

/// PredicateType represents one of the supported types in Grapl
#[derive(Copy, Clone, Debug)]
pub enum PredicateType {
    String,
    I64,
    U64,
}

// Python code generation
impl PredicateType {
    pub fn into_python_prop_primitive(self) -> String {
        match self {
            PredicateType::String => "grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Str, False)",
            PredicateType::I64 => "grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False)",
            PredicateType::U64 => "grapl_analyzerlib.node_types.PropType(grapl_analyzerlib.node_types.PropPrimitive.Int, False)",
        }.to_string()
    }

    pub fn into_python_primitive_type(self) -> String {
        match self {
            PredicateType::String => "str",
            PredicateType::I64 => "int",
            PredicateType::U64 => "int",
        }.to_string()
    }

    pub fn into_python_primitive_type_or_not(self) -> String {
        match self {
            PredicateType::String => "StrOrNot",
            PredicateType::I64 => "IntOrNot",
            PredicateType::U64 => "IntOrNot",
        }.to_string()
    }
}

impl<'a> TryFrom<&Type<'a, &'a str>> for PredicateType {
    type Error = CodeGenError<'a>;

    #[tracing::instrument]
    fn try_from(value: &Type<'a, &'a str>) -> Result<Self, Self::Error> {
        match value {
            Type::NamedType(value) => {
                match *value {
                    STRING => Ok(PredicateType::String),
                    INT => Ok(PredicateType::I64),
                    UINT => Ok(PredicateType::U64),
                    // todo: error
                    unsupported => panic!("Unsupported type: {}", unsupported),
                }
            }
            Type::NonNullType(ref value) => value.as_ref().try_into(),
            Type::ListType(_) => todo!("We don't currently support sets of values in Grapl")
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predicate_type() {
        assert_eq!(PredicateType::String.into_python_primitive_type_or_not(), "StrOrNot");
        assert_eq!(PredicateType::I64.into_python_primitive_type_or_not(), "IntOrNot");
        assert_eq!(PredicateType::U64.into_python_primitive_type_or_not(), "IntOrNot");
    }
}
