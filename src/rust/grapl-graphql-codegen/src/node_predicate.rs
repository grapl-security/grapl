use std::convert::{
    TryFrom,
    TryInto,
};

use graphql_parser::schema::{
    Field,
    Type,
};

use crate::{
    as_static_python::AsStaticPython,
    conflict_resolution::ConflictResolution,
    errors::CodeGenError,
    identity_predicate_type::IdentityPredicateType,
    predicate_type::PredicateType,
};

/// The NodePredicate holds all of the information for a defined property or edge
#[derive(Debug)]
pub struct NodePredicate {
    pub predicate_name: String,
    pub description: Option<String>,
    pub predicate_type: PredicateType,
    pub conflict_resolution: ConflictResolution,
    pub identity_predicate_type: Option<IdentityPredicateType>,
    pub nullable: bool,
}

impl NodePredicate {
    // Python code generation for NodePredicate
    pub fn generate_python_str_comparisons(&self) -> String {
        let mut comparisons = String::with_capacity(256);

        comparisons += "        eq: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,\n";
        comparisons += "        contains: Optional[grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]] = None,\n";
        comparisons +=
            "        starts_with: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,\n";
        comparisons +=
            "        ends_with: Optional[grapl_analyzerlib.comparators.StrOrNot] = None,\n";
        comparisons += "        regexp: Optional[grapl_analyzerlib.comparators.OneOrMany[grapl_analyzerlib.comparators.StrOrNot]] = None,\n";
        comparisons += "        distance_lt: Optional[Tuple[str, int]] = None,";
        comparisons
    }

    pub fn generate_python_int_comparisons(&self) -> String {
        let mut comparisons = String::with_capacity(256);

        comparisons += "        eq: Optional[grapl_analyzerlib.comparators.IntOrNot] = None,\n";
        comparisons += "        gt: Optional[grapl_analyzerlib.comparators.IntOrNot] = None,\n";
        comparisons += "        ge: Optional[grapl_analyzerlib.comparators.IntOrNot] = None,\n";
        comparisons += "        lt: Optional[grapl_analyzerlib.comparators.IntOrNot] = None,\n";
        comparisons += "        le: Optional[grapl_analyzerlib.comparators.IntOrNot] = None,\n";
        comparisons
    }

    pub fn generate_python_query_comparisons(&self) -> String {
        match self.predicate_type {
            PredicateType::String => self.generate_python_str_comparisons(),
            PredicateType::I64 => self.generate_python_int_comparisons(),
            PredicateType::U64 => self.generate_python_int_comparisons(),
        }
    }

    pub fn generate_python_query_def(&self) -> String {
        let mut query_def = String::with_capacity(256);
        let python_ty = self.predicate_type.into_python_primitive_type();

        query_def += &format!("    def with_{}(\n", self.predicate_name);
        query_def += "        self,\n";
        query_def += "        *,\n";
        query_def += &self.generate_python_query_comparisons();
        query_def += "\n    ):\n";
        query_def += "        (\n";
        query_def += &format!("            self.with_{}_property(\n", &python_ty);
        query_def += &format!("                \"{}\",\n", self.predicate_name);

        match self.predicate_type {
            PredicateType::String => {
                query_def += "                eq=eq,\n";
                query_def += "                contains=contains,\n";
                query_def += "                starts_with=starts_with,\n";
                query_def += "                ends_with=ends_with,\n";
                query_def += "                regexp=regexp,\n";
                query_def += "                distance_lt=distance_lt\n";
            }
            PredicateType::U64 | PredicateType::I64 => {
                query_def += "                eq=eq,\n";
                query_def += "                gt=gt,\n";
                query_def += "                ge=ge,\n";
                query_def += "                lt=lt,\n";
                query_def += "                le=le,\n";
            }
        }
        query_def += "            )\n";
        query_def += "        )\n";
        query_def += "        return self\n";
        query_def
    }

    pub fn generate_viewable_get_predicate_method(&self) -> String {
        let mut get_method = String::with_capacity(512);

        let predicate_name = self.predicate_name.as_str();
        let py_ty = self.predicate_type.into_python_primitive_type();
        let cached = self
            .conflict_resolution
            .implies_cacheable()
            .as_static_python();
        get_method += &format!(
            "    def get_{}(self, cached: bool = {}) -> Optional[{}]:\n",
            predicate_name, cached, py_ty
        );
        get_method += &format!(
            "        return self.get_{}('{}', cached=cached)\n\n",
            py_ty, predicate_name
        );

        get_method
    }
}

impl<'a> TryFrom<&Field<'a, &'a str>> for NodePredicate {
    type Error = CodeGenError<'a>;

    fn try_from(value: &Field<'a, &'a str>) -> Result<Self, Self::Error> {
        let predicate_name = value.name.to_string();
        let description = value.description.to_owned();
        let field_type: &graphql_parser::schema::Type<_> = &value.field_type;
        let predicate_type: PredicateType = PredicateType::try_from(field_type)?;
        let identity_predicate_type: Option<IdentityPredicateType> = value
            .directives
            .iter()
            .find_map(|d| IdentityPredicateType::opt_from(d));
        let nullable = is_nullable(field_type);
        let conflict_resolution = value.directives.as_slice().try_into()?;

        Ok(Self {
            predicate_name,
            description,
            predicate_type,
            conflict_resolution,
            identity_predicate_type,
            nullable,
        })
    }
}

pub fn is_nullable<'a>(field_type: &Type<'a, &'a str>) -> bool {
    match field_type {
        Type::NonNullType(_) => false,
        Type::NamedType(_) => true,
        Type::ListType(_) => panic!("ListType not supported"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_viewable_get_predicate_method() {
        let expected_str = "    def get_predicate_name(self, cached: bool = True) -> Optional[str]:\n        return self.get_str('predicate_name', cached=cached)\n\n";
        let node_predicate = NodePredicate {
            predicate_name: String::from("predicate_name"),
            description: None,
            predicate_type: PredicateType::String,
            conflict_resolution: ConflictResolution::Immutable,
            identity_predicate_type: None,
            nullable: true,
        };
        assert_eq!(
            expected_str,
            node_predicate.generate_viewable_get_predicate_method()
        );
    }
}
