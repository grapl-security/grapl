use std::convert::TryFrom;

use graphql_parser::schema::Type;

use crate::errors::CodeGenError;

/// EdgeRel describes the bi-directional relationship of an edge in terms of
/// whether the edge points to one or many nodes
#[derive(Copy, Debug, Clone)]
pub enum EdgeRel {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

// Python code generation for EdgeRel
impl EdgeRel {
    pub fn to_one(&self) -> bool {
        match self {
            Self::OneToOne => true,
            Self::ManyToOne => true,
            Self::OneToMany => false,
            Self::ManyToMany => false,
        }
    }

    pub fn to_many(&self) -> bool {
        match self {
            Self::OneToOne => false,
            Self::ManyToOne => false,
            Self::OneToMany => true,
            Self::ManyToMany => true,
        }
    }

    pub fn reverse(self) -> Self {
        match self {
            Self::OneToOne => Self::OneToOne,
            Self::OneToMany => Self::ManyToOne,
            Self::ManyToOne => Self::OneToMany,
            Self::ManyToMany => Self::ManyToMany,
        }
    }

    pub fn to_edge_rel_py(&self) -> String {
        match self {
            Self::OneToOne => "grapl_analyzerlib.node_types.EdgeRelationship.OneToOne".to_owned(),
            Self::OneToMany => "grapl_analyzerlib.node_types.EdgeRelationship.OneToMany".to_owned(),
            Self::ManyToOne => "grapl_analyzerlib.node_types.EdgeRelationship.ManyToOne".to_owned(),
            Self::ManyToMany => {
                "grapl_analyzerlib.node_types.EdgeRelationship.ManyToMany".to_owned()
            }
        }
    }
}
use graphql_parser::schema::Field;

impl<'a> TryFrom<&Field<'a, &'a str>> for EdgeRel {
    type Error = CodeGenError<'a>;

    fn try_from(field: &Field<'a, &'a str>) -> Result<Self, Self::Error> {
        let edge_directive = field
            .directives
            .iter()
            .find(|directive| directive.name == "edge");
        let edge_directive = match edge_directive {
            Some(d) => d,
            None => {
                return Err(CodeGenError::MissingGraplDirective {
                    directives: field.directives.to_vec(),
                })
            }
        };
        let reverse_rel =
            edge_directive
                .arguments
                .iter()
                .find_map(|(arg_name, arg)| match (*arg_name, arg) {
                    ("reverse_relationship", graphql_parser::schema::Value::String(s)) => Some(s),
                    (_, _) => None,
                });
        let reverse_rel = match reverse_rel {
            Some(reverse_rel) => reverse_rel.as_str(),
            None => return Err(CodeGenError::NodeTypeParseError),
        };

        let forward_rel = match field.field_type {
            Type::NamedType(_) => "ToOne",
            Type::ListType(_) => "ToMany",
            Type::NonNullType(ref t) => match t.as_ref() {
                Type::NamedType(_) => "ToOne",
                Type::ListType(_) => "ToMany",
                _ => return Err(CodeGenError::NodeTypeParseError),
            },
        };

        // match (forward_rel, reverse_rel) {
        //     ("ToOne", "ToOne") => Ok(EdgeRel::OneToOne),
        //     ("ToOne", "ToMany") => Ok(EdgeRel::OneToMany),
        //     ("ToMany", "ToOne") => Ok(EdgeRel::ManyToOne),
        //     ("ToMany", "ToMany") => Ok(EdgeRel::ManyToMany),
        //     (_, _) => unreachable!(),
        // }

        match (forward_rel, reverse_rel) {
            ("ToOne", "ToOne") => Ok(EdgeRel::OneToOne),
            ("ToOne", "ToMany") => Ok(EdgeRel::ManyToOne),
            ("ToMany", "ToOne") => Ok(EdgeRel::OneToMany),
            ("ToMany", "ToMany") => Ok(EdgeRel::ManyToMany),
            (_, _) => unreachable!(),
        }
    }
}
