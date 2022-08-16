use graphql_parser::schema::Directive;

/// Represents which algorithm is used to identify a node
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum IdentificationAlgorithm {
    Session,
    Static,
}

use std::convert::TryFrom;

use crate::{
    constants::{
        SESSION_ALGORITHM,
        STATIC_ALGORITHM,
    },
    errors::CodeGenError,
};

impl IdentificationAlgorithm {
    pub fn from_directive(directive: &Directive<'static, String>) -> Option<Self> {
        if directive.name != "grapl" {
            return None;
        }
        directive
            .arguments
            .iter()
            .find_map(|(arg_name, arg)| match (arg_name.as_str(), arg) {
                ("identity_algorithm", graphql_parser::schema::Value::String(s))
                    if s == SESSION_ALGORITHM =>
                {
                    Some(Self::Session)
                }
                ("identity_algorithm", graphql_parser::schema::Value::String(s))
                    if s == STATIC_ALGORITHM =>
                {
                    Some(Self::Static)
                }
                (_, _) => None,
            })
    }
}

impl TryFrom<&[Directive<'static, String>]> for IdentificationAlgorithm {
    type Error = CodeGenError;

    fn try_from(directives: &[Directive<'static, String>]) -> Result<Self, Self::Error> {
        directives
            .iter()
            .find_map(IdentificationAlgorithm::from_directive)
            .ok_or_else(|| CodeGenError::MissingNodeIdentificationAlgorithm {
                directives: directives.to_vec(),
            })
    }
}
