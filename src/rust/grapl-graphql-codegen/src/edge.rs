use std::convert::{
    TryFrom,
    TryInto,
};

use graphql_parser::schema::{
    Field,
    Type,
};

use crate::{
    edge_rel::EdgeRel,
    errors::CodeGenError,
};

/// The Edge structure represents a bi-directional relationship between
/// two nodes
#[derive(Debug, Clone)]
pub struct Edge {
    /// `edge_name` is the string name for the "forward" edge
    pub edge_name: String,
    /// `reverse_edge_name` is the string name for the "reverse" edge
    pub reverse_edge_name: String,
    /// The name of the Node that is "pointing"
    pub source_type_name: String,
    /// The name of the Node that is "pointed to"
    pub target_type_name: String,
    /// Relationships denote whether the forward/reverse edges are to one or many nodes
    pub relationship: EdgeRel,
}

// Python generation code for Edge
impl Edge {
    pub fn reverse(self) -> Self {
        let Edge {
            edge_name,
            reverse_edge_name,
            source_type_name,
            target_type_name,
            relationship,
        } = self;

        Edge {
            edge_name: reverse_edge_name,
            reverse_edge_name: edge_name,
            source_type_name: target_type_name,
            target_type_name: source_type_name,
            relationship: relationship.reverse(),
        }
    }

    pub fn generate_python_query_def(&self) -> String {
        let mut query_def = String::with_capacity(256);
        let src_edge_name = &self.edge_name;
        let rev_edge_name = &self.reverse_edge_name;
        query_def = query_def
            + &format!(
                r#"    def with_{src_edge_name}(self: {source_type_name}Query, *{src_edge_name}: {target_type_name}Query) -> {source_type_name}Query:"#,
                src_edge_name = src_edge_name,
                source_type_name = self.source_type_name,
                target_type_name = self.target_type_name,
            )
            + "\n";
        query_def = query_def
            + &format!(
                r#"        return self.with_to_neighbor({source_type_name}Query, "{src_edge_name}", "{rev_edge_name}", {src_edge_name})"#,
                source_type_name = self.source_type_name,
                src_edge_name = src_edge_name,
                rev_edge_name = rev_edge_name,
            )
            + "\n";
        query_def
    }

    pub fn generate_edge_relationship(&self) -> String {
        let mut edge_relationship = String::with_capacity(256);
        let src_schema = format!("{}Schema", self.source_type_name);
        let dst_schema = format!("{}Schema", self.target_type_name);

        edge_relationship =
            edge_relationship + &format!(r#"        "{}": ("#, self.edge_name) + "\n";
        edge_relationship = edge_relationship
            + &format!(
                "            grapl_analyzerlib.node_types.EdgeT({}, {}, {}),",
                src_schema,
                dst_schema,
                self.relationship.to_edge_rel_py()
            )
            + "\n";
        edge_relationship =
            edge_relationship + &format!(r#"            "{}""#, self.reverse_edge_name) + "\n";
        edge_relationship += "        ),\n";
        edge_relationship
    }

    pub fn generate_viewable_get_edge_method(&self) -> String {
        let mut get_method = String::with_capacity(512);
        let edge_name = self.edge_name.as_str();
        let reverse_edge_name = self.reverse_edge_name.as_str();
        let edge_view_name = format!("{}View", self.target_type_name);
        let edge_query_name = format!("{}Query", self.target_type_name);

        let (multi, cached, query_arg, ret) = match self.relationship.to_one() {
            true => (
                "",
                "True",
                format!("Optional[{}] = None", edge_query_name),
                format!("'Optional[{}]'", edge_view_name),
            ),
            false => (
                "*",
                "False",
                edge_query_name.to_string(),
                format!("'List[{}]'", edge_view_name),
            ),
        };

        get_method = get_method
            + &format!(
                "    def get_{edge_name}(self, {multi}{edge_name}: {query_arg}, cached={cached}) -> {ret}:",
                edge_name=edge_name,
                multi=multi,
                query_arg=query_arg,
                cached=cached,
                ret=ret
            )
            + "\n";
        get_method = get_method
            + &format!(
                r#"          return self.get_neighbor({edge_query_name}, "{edge_name}", "{reverse_edge_name}", {edge_name}, cached)"#,
                edge_query_name = edge_query_name,
                edge_name = edge_name,
                reverse_edge_name = reverse_edge_name,
            );
        get_method += "\n";

        get_method
    }
}

impl<'a> TryFrom<(String, &Field<'a, &'a str>)> for Edge {
    type Error = CodeGenError<'a>;

    fn try_from(
        (source_type_name, field): (String, &Field<'a, &'a str>),
    ) -> Result<Self, Self::Error> {
        let edge_name = field.name.to_string();
        let reverse_edge_name = field
            .directives
            .iter()
            .flat_map(|directive| &directive.arguments)
            .find_map(|(argument_name, argument)| {
                if *argument_name == "reverse" {
                    if let graphql_parser::schema::Value::String(argument) = argument {
                        Some(argument.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap();

        let target_type_name = get_type_name(&field.field_type);
        let relationship = field.try_into()?;

        Ok(Edge {
            edge_name,
            reverse_edge_name,
            source_type_name,
            target_type_name,
            relationship,
        })
    }
}

pub fn get_type_name<'a>(ty: &Type<'a, &'a str>) -> String {
    match ty {
        Type::NamedType(t) => t.to_string(),
        Type::ListType(t) => get_type_name(t),
        Type::NonNullType(t) => get_type_name(t),
    }
}
