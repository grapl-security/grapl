use std::convert::{
    TryFrom,
    TryInto,
};

use color_eyre::eyre::Result;
use graphql_parser::schema::{
    Definition,
    Directive,
    Document,
    ObjectType,
    TypeDefinition,
};

use crate::{
    edge::Edge,
    errors::CodeGenError,
    field_type::FieldType,
    identification_algorithm::IdentificationAlgorithm,
    node_predicate::NodePredicate,
};

// Python Queryable generation
pub fn generate_parameter_from_predicate(predicate: &NodePredicate) -> String {
    let parameter_name = predicate.predicate_name.as_str();
    let parameter_ty = predicate.predicate_type.into_python_primitive_type();
    format!(
        r#"{}: Optional["{}"] = None,"#,
        parameter_name, parameter_ty
    )
}

pub fn generate_parameter_from_edge(edge: &Edge) -> String {
    let parameter_name = edge.edge_name.as_str();
    let parameter_ty = format!("{}View", edge.target_type_name.as_str());
    match edge.relationship.to_one() {
        true => format!(
            r#"{}: Optional["{}"] = None,"#,
            parameter_name, parameter_ty
        ),
        false => format!(
            r#"{}: Optional[List["{}"]] = None,"#,
            parameter_name, parameter_ty
        ),
    }
}

pub fn generate_set_predicate_from_predicate(predicate: &NodePredicate) -> String {
    let predicate_name = predicate.predicate_name.as_str();
    format!(
        r#"if {predicate_name}: self.set_predicate("{predicate_name}", {predicate_name})"#,
        predicate_name = predicate_name,
    )
}

pub fn generate_set_predicate_from_edge(edge: &Edge) -> String {
    let predicate_name = edge.edge_name.as_str();
    match edge.relationship.to_one() {
        true => format!(
            r#"if {predicate_name}: self.set_predicate("{predicate_name}", {predicate_name})"#,
            predicate_name = predicate_name
        ),
        false => format!(
            r#"if {predicate_name}: self.set_predicate("{predicate_name}", {predicate_name} or [])"#,
            predicate_name = predicate_name
        ),
    }
}

/// NodeType represents the schema of a Grapl node
#[derive(Debug)]
pub struct NodeType {
    pub type_name: String,
    pub identification_algorithm: IdentificationAlgorithm,
    pub predicates: Vec<NodePredicate>,
    pub edges: Vec<Edge>,
}

/// NodeExtension represents an 'extension' of a node, which is to say
/// that they are a mutation of existing NodeType's
#[derive(Debug)]
pub struct NodeExtension {
    pub extends_type: String,
    pub predicates: Vec<NodePredicate>,
    pub edges: Vec<Edge>,
}

/// NodeTypes and NodeExtensions are the main drivers for this code, representing the
/// top level abstraction for defining or working with Grapl schemas
#[derive(Debug)]
pub enum NodeTypeOrExtension {
    NodeType(NodeType),
    NodeExtension(NodeExtension),
}

#[derive(thiserror::Error, Debug)]
pub enum MergeFailure {
    #[error("Type Name Mismatch")]
    TypeNameMismatch(String, String),
}

impl NodeType {
    #[tracing::instrument(skip(self, other))]
    pub fn extend_node_type(&mut self, other: NodeExtension) -> Result<(), MergeFailure> {
        tracing::trace!(
            message="Extending NodeType",
            self_type_name=?self.type_name,
            extended_edge_count=?other.edges.len(),
            predicates_edge_count=?other.predicates.len(),
        );
        if self.type_name != other.extends_type {
            return Err(MergeFailure::TypeNameMismatch(
                self.type_name.clone(),
                other.extends_type,
            ));
        }
        self.predicates.extend(other.predicates.into_iter());

        Ok(())
    }

    pub fn generate_python_code(&self) -> String {
        let mut pycode = String::with_capacity(256);

        pycode += &self.generate_python_schema();
        pycode += "\n";

        pycode += &self.generate_python_queryable();
        pycode += "\n";

        pycode += &self.generate_python_viewable();
        pycode += "\n";

        pycode
    }

    fn get_query_name(&self) -> String {
        format!("{}Query", self.type_name)
    }

    fn get_view_name(&self) -> String {
        format!("{}View", self.type_name)
    }

    pub fn generate_queryable_node_schema_method(&self) -> String {
        let mut node_schema_method = String::with_capacity(128);
        let schema_name = format!("{}Schema", self.type_name);

        node_schema_method = node_schema_method + r#"    @classmethod"# + "\n";
        node_schema_method = node_schema_method
            + r#"    def node_schema(cls) -> "grapl_analyzerlib.schema.Schema":"#
            + "\n";
        node_schema_method =
            node_schema_method + &format!(r#"        return {}()"#, schema_name) + "\n";
        node_schema_method
    }

    #[tracing::instrument(skip(self))]
    pub fn generate_python_queryable(&self) -> String {
        let (query_name, view_name) = (self.get_query_name(), self.get_view_name());

        tracing::trace!(
            message="Generating Python Queryable",
            node_type=?self.type_name,
            query_name=?query_name,
            view_name=?view_name,
        );

        let superclasses =
            format!("grapl_analyzerlib.nodes.entity.EntityQuery['{view_name}', '{query_name}']");
        let mut queryable = String::with_capacity(256);
        queryable += "\n";
        queryable += "@dataclass(init=False)\n";
        queryable += &format!("class {query_name}({superclasses}):\n");

        for predicate in self.predicates.iter() {
            queryable.push_str(&predicate.generate_python_query_def());
            queryable.push('\n');
        }

        for edge in self.edges.iter() {
            queryable.push_str(&edge.generate_python_query_def());
            queryable.push('\n');
        }

        queryable += &self.generate_queryable_node_schema_method();

        queryable
    }

    #[tracing::instrument(skip(self))]
    pub fn generate_python_viewable(&self) -> String {
        let (query_name, view_name) = (self.get_query_name(), self.get_view_name());

        tracing::trace!(
            message="Generating Python Viewable",
            node_type=?self.type_name,
            query_name=?query_name,
            view_name=?view_name,
        );

        let superclasses =
            format!("grapl_analyzerlib.nodes.entity.EntityView['{view_name}', '{query_name}']");

        let mut viewable = String::with_capacity(512);
        viewable += "\n";
        viewable += "@dataclass(init=False)\n";
        viewable += &format!("class {view_name}({superclasses}):\n");
        viewable += &format!("    queryable = {query_name}\n\n");
        viewable += "    def __init__(\n";
        viewable += "        self,\n";
        viewable += "        uid: int,\n";
        viewable += "        node_key: str,\n";
        viewable += "        graph_client: Any,\n";
        viewable += "        node_types: Set[str],\n";

        for predicate in self.predicates.iter() {
            let parameter = generate_parameter_from_predicate(predicate);
            viewable = viewable + "        " + &parameter + "\n";
        }

        for edge in self.edges.iter() {
            let parameter = generate_parameter_from_edge(edge);
            viewable = viewable + "        " + &parameter + "\n";
        }

        viewable += "        **kwargs,\n";
        viewable += "    ) -> None:\n";
        viewable += "        super().__init__(uid, node_key, graph_client, node_types, **kwargs)\n";

        for predicate in self.predicates.iter() {
            let predicate = generate_set_predicate_from_predicate(predicate);
            viewable = viewable + "        " + &predicate + "\n";
        }

        for edge in self.edges.iter() {
            let predicate = generate_set_predicate_from_edge(edge);
            viewable = viewable + "        " + &predicate + "\n";
        }
        viewable.push('\n');
        viewable += &self.generate_viewable_get_methods();

        viewable.push('\n');
        viewable += &self.generate_queryable_node_schema_method();

        viewable
    }

    pub fn generate_viewable_get_methods(&self) -> String {
        let mut get_methods = String::with_capacity(512);

        for predicate in self.predicates.iter() {
            tracing::trace!(
                message="Generating Python Viewable predicate get methods",
                node_type=?self.type_name,
                predicate_name=?predicate.predicate_name,
                predicate_type=?predicate.predicate_type,
                query_name=?self.get_query_name(),
                view_name=?self.get_view_name(),
            );

            let get_method = predicate.generate_viewable_get_predicate_method();
            get_methods += &get_method;
        }

        for edge in self.edges.iter() {
            tracing::trace!(
                message="Generating Python Viewable edge get methods",
                node_type=?self.type_name,
                edge_name=?edge.edge_name,
                reverse_edge_name=?edge.reverse_edge_name,
                source_type_name=?edge.source_type_name,
                target_type_name=?edge.target_type_name,
                relationship=?edge.relationship,
                query_name=?self.get_query_name(),
                view_name=?self.get_view_name(),
            );

            let get_method = edge.generate_viewable_get_edge_method();
            get_methods += &get_method;
        }

        get_methods
    }

    // Python Schema generation
    pub fn generate_python_schema(&self) -> String {
        let mut schema_str = String::with_capacity(256);
        tracing::trace!(
            message="Generating Python Schema",
            node_type=?self.type_name,
        );

        schema_str += &self.generate_python_default_schema_properties();
        schema_str += "\n";
        schema_str += &self.generate_python_default_schema_edges();
        schema_str += "\n";
        schema_str += &self.generate_python_schema_class();
        schema_str += "\n";
        schema_str += &self.generate_python_schema_self_type();
        schema_str += "\n";

        schema_str
    }

    pub fn generate_python_schema_self_type(&self) -> String {
        let mut schema_str = String::with_capacity(256);
        schema_str += "    @staticmethod\n";
        schema_str += "    def self_type() -> str:\n";
        schema_str = schema_str + &format!(r#"        return "{}""#, self.type_name) + "\n";
        schema_str
    }

    pub fn generate_python_schema_class(&self) -> String {
        let mut schema_str = String::with_capacity(256);
        let schema_name = format!("{}Schema", self.type_name);
        let view_name = format!("{}View", self.type_name);
        let lower_node_name = self.type_name.to_lowercase();

        schema_str += "\n";
        schema_str = schema_str
            + "class "
            + &schema_name
            + "(grapl_analyzerlib.nodes.entity.EntitySchema):\n";
        schema_str += "    def __init__(self):\n";
        schema_str = schema_str + "        super(" + &schema_name + ", self).__init__(\n";
        schema_str = schema_str
            + "            default_"
            + &lower_node_name
            + "_properties(),"
            + " default_"
            + &lower_node_name
            + "_edges(),"
            + " lambda: "
            + &view_name
            + "\n";
        schema_str += "        );\n";
        schema_str
    }

    pub fn generate_python_default_schema_properties(&self) -> String {
        let mut def = String::with_capacity(256);

        let lower_node_name = self.type_name.to_lowercase();

        def = def
            + r#"def default_"#
            + &lower_node_name
            + r#"_properties() -> Dict[str, grapl_analyzerlib.node_types.PropType]:"#
            + "\n";
        def = def + r#"    return {"# + "\n";
        for predicate in self.predicates.iter() {
            let predicate_name = format!(r#""{}""#, &predicate.predicate_name);
            let prop_primitive_t = predicate.predicate_type.into_python_prop_primitive();
            def = def + &format!("        {}: {},\n", predicate_name, prop_primitive_t);
        }

        def += r#"    }"#;
        def
    }

    pub fn generate_python_default_schema_edges(&self) -> String {
        let mut def = String::with_capacity(256);

        let lower_node_name = self.type_name.to_lowercase();

        def = def
            + r#"def default_"#
            + &lower_node_name
            + r#"_edges() -> Dict[str, Tuple[grapl_analyzerlib.node_types.EdgeT, str]]:"#
            + "\n";
        def = def + r#"    return {"# + "\n";
        for edge in self.edges.iter() {
            def = def + &edge.generate_edge_relationship();
        }
        def += r#"    }"#;
        def
    }
}

impl<'a> TryFrom<&ObjectType<'a, &'a str>> for NodeType {
    type Error = CodeGenError<'a>;

    fn try_from(object: &ObjectType<'a, &'a str>) -> Result<Self, Self::Error> {
        let type_name = object.name.to_string();
        let mut predicates = vec![];
        let mut edges = vec![];
        for field in object.fields.iter() {
            match FieldType::from(field) {
                FieldType::Predicate => predicates.push(field.try_into()?),
                FieldType::Edge => edges.push((type_name.clone(), field).try_into()?),
            }
        }
        let identification_algorithm = object.directives.as_slice().try_into()?;
        Ok(NodeType {
            type_name,
            identification_algorithm,
            predicates,
            edges,
        })
    }
}

fn get_extends_type_name<'a>(
    directives: &[Directive<'a, &'a str>],
) -> Result<String, CodeGenError<'a>> {
    let grapl_directive = directives
        .iter()
        .find_map(|d| if d.name == "grapl" { Some(Ok(d)) } else { None })
        .unwrap_or_else(|| {
            Err(CodeGenError::MissingGraplDirective {
                directives: directives.to_vec(),
            })
        })?;

    let extends_type = grapl_directive
        .arguments
        .iter()
        .find_map(|(arg_name, arg)| match (*arg_name, arg) {
            ("extends", graphql_parser::schema::Value::String(arg)) => Some(Ok(arg)),
            _ => None,
        })
        .unwrap_or_else(|| {
            Err(CodeGenError::MissingGraplDirectiveArguments {
                directives: directives.to_vec(),
            })
        })?;

    Ok(extends_type.to_string())
}

impl<'a> TryFrom<&ObjectType<'a, &'a str>> for NodeExtension {
    type Error = CodeGenError<'a>;

    fn try_from(object: &ObjectType<'a, &'a str>) -> Result<Self, Self::Error> {
        let extends_type = get_extends_type_name(&object.directives)?;

        let mut predicates = vec![];
        let mut edges = vec![];

        for field in object.fields.iter() {
            match FieldType::from(field) {
                FieldType::Predicate => predicates.push(field.try_into()?),
                FieldType::Edge => edges.push((extends_type.clone(), field).try_into()?),
            }
        }
        Ok(NodeExtension {
            extends_type,
            predicates,
            edges,
        })
    }
}

impl<'a> TryFrom<&ObjectType<'a, &'a str>> for NodeTypeOrExtension {
    type Error = CodeGenError<'a>;

    fn try_from(object: &ObjectType<'a, &'a str>) -> Result<Self, Self::Error> {
        let grapl_directive = object
            .directives
            .iter()
            .find_map(|d| if d.name == "grapl" { Some(Ok(d)) } else { None })
            .unwrap_or_else(|| {
                Err(CodeGenError::MissingGraplDirective {
                    directives: object.directives.to_vec(),
                })
            })?;

        let n = grapl_directive
            .arguments
            .iter()
            .find_map(|(arg_name, _)| match *arg_name {
                "identity_algorithm" => match NodeType::try_from(object) {
                    Ok(n) => Some(Ok(NodeTypeOrExtension::NodeType(n))),
                    Err(e) => Some(Err(e)),
                },
                "extends" => match NodeExtension::try_from(object) {
                    Ok(n) => Some(Ok(NodeTypeOrExtension::NodeExtension(n))),
                    Err(e) => Some(Err(e)),
                },
                _ => None,
            })
            .unwrap_or_else(|| {
                Err(CodeGenError::MissingGraplDirectiveArguments {
                    directives: object.directives.to_vec(),
                })
            })?;

        Ok(n)
    }
}

#[tracing::instrument(skip(document))]
pub fn parse_into_node_types<'a>(
    document: &Document<'a, &'a str>,
) -> Result<Vec<NodeType>, CodeGenError<'a>> {
    let mut types = std::collections::BTreeMap::new();
    let mut node_types = vec![];

    for definition in document.definitions.iter() {
        let type_definition =
            if let Definition::TypeDefinition(TypeDefinition::Object(type_definition)) = definition
            {
                type_definition.clone()
            } else {
                continue;
            };
        let type_name = type_definition.name.clone();
        types.insert(type_name.clone(), type_definition);
    }

    let mut node_extensions = std::collections::BTreeMap::new();
    for (type_name, node_type) in types.into_iter() {
        tracing::debug!(
            message="Parsing GraphQL ObjectType",
            type_name=?type_name,
        );
        let nt: NodeTypeOrExtension = NodeTypeOrExtension::try_from(&node_type)?;
        match nt {
            NodeTypeOrExtension::NodeType(node_type) => node_types.push(node_type),
            NodeTypeOrExtension::NodeExtension(node_extension) => {
                node_extensions
                    .entry(type_name.to_string())
                    .or_insert(vec![])
                    .push(node_extension);
            }
        }
    }

    tracing::debug!(
        message="Parsed GraphQL ObjectTypes",
        node_types_len=?node_types.len(),
        node_extensions_len=?node_extensions.len(),
    );

    for node_type in node_types.iter_mut() {
        let extensions = {
            match node_extensions.remove(node_type.type_name.as_str()) {
                Some(extensions) => extensions,
                None => continue,
            }
        };
        for extension in extensions.into_iter() {
            node_type.extend_node_type(extension)?;
        }
    }
    Ok(node_types)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_contains_str(bigger_string: String, subset: &str) {
        assert!(
            bigger_string.contains(subset),
            "Expected \n'{bigger_string}'\nto contain \n'{subset}'\n"
        );
    }

    fn get_test_node_type() -> NodeType {
        NodeType {
            type_name: "Type".to_owned(),
            identification_algorithm: IdentificationAlgorithm::Static,
            predicates: vec![],
            edges: vec![],
        }
    }

    #[test]
    fn test_generate_python_queryable() {
        let generated = get_test_node_type().generate_python_queryable();
        let expected_str = r#"
@dataclass(init=False)
class TypeQuery(grapl_analyzerlib.nodes.entity.EntityQuery['TypeView', 'TypeQuery']):
"#;
        assert_contains_str(generated, expected_str);
    }

    #[test]
    fn test_generate_python_viewable() {
        let generated = get_test_node_type().generate_python_viewable();
        let expected_str = r#"
@dataclass(init=False)
class TypeView(grapl_analyzerlib.nodes.entity.EntityView['TypeView', 'TypeQuery']):
    queryable = TypeQuery
"#;
        assert_contains_str(generated, expected_str);
    }
}
