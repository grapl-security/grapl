use graphql_parser::schema::Field;

/// Given a field in a GraphQL schema, represents that field
/// being an predicate vs an edge
#[derive(Copy, Clone, Debug)]
pub enum FieldType {
    Predicate,
    Edge,
}

impl From<&Field<'static, String>> for FieldType {
    fn from(field: &Field<'static, String>) -> Self {
        field
            .directives
            .iter()
            .map(|d| {
                match d.name.as_str() {
                    "edge" => FieldType::Edge,
                    // todo: We should be more specific here
                    _ => FieldType::Predicate,
                }
            })
            .next()
            .unwrap()
    }
}
