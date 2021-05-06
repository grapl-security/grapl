use graphql_parser::schema::{Field};

/// Given a field in a GraphQL schema, represents that field
/// being an predicate vs an edge
#[derive(Copy, Clone, Debug)]
pub enum FieldType {
    Predicate,
    Edge,
}

impl<'a> From<&Field<'a, &'a str>> for FieldType {
    fn from(field: &Field<'a, &'a str>) -> Self {
        field.directives
            .iter()
            .find_map(|d| {
                match d.name {
                    "edge" => Some(FieldType::Edge),
                    // todo: We should be more specific here
                    _ => Some(FieldType::Predicate)
                }
            })
            .unwrap()
    }
}
