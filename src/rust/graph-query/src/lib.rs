mod var_allocator;
use fnv::FnvHashMap as HashMap;

use crate::var_allocator::VarAllocator;

pub trait PropertyFilter {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String;
    fn boxed(self) -> Box<dyn PropertyFilter>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn PropertyFilter>
    }
}

pub trait NodeFilter {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String;
}

#[derive(Clone)]
pub struct IntEq {
    to: u64,
    negated: bool,
}

impl IntEq {
    pub fn new(to: u64, negated: bool) -> Self {
        Self { to, negated }
    }
}

impl PropertyFilter for IntEq {
    fn to_filter(&self, _var_allocator: &mut VarAllocator) -> String {
        let inner = format!(
            "{{
            eq: {},
        }}",
            self.to
        );
        if self.negated {
            format!("{{not: {}}}", inner)
        } else {
            inner
        }
    }
}

#[derive(Clone)]
pub struct IntLt {
    to: u64,
    negated: bool,
}

impl IntLt {
    pub fn new(to: u64, negated: bool) -> Self {
        Self { to, negated }
    }
}

impl PropertyFilter for IntLt {
    fn to_filter(&self, _var_allocator: &mut VarAllocator) -> String {
        let inner = format!(
            "{{
            lt: {},
        }}",
            self.to
        );
        if self.negated {
            format!("{{not: {}}}", inner)
        } else {
            inner
        }
    }
}

#[derive(Clone)]
pub struct IntGt {
    to: u64,
    negated: bool,
}

impl IntGt {
    pub fn new(to: u64, negated: bool) -> Self {
        Self { to, negated }
    }
}

impl PropertyFilter for IntGt {
    fn to_filter(&self, _var_allocator: &mut VarAllocator) -> String {
        let inner = format!(
            "{{
            gt: {},
        }}",
            self.to
        );
        if self.negated {
            format!("{{not: {}}}", inner)
        } else {
            inner
        }
    }
}

#[derive(Clone)]
pub struct StrEq {
    to: String,
    negated: bool,
}

impl StrEq {
    pub fn new(to: impl Into<String>, negated: bool) -> Self {
        Self {
            to: to.into(),
            negated,
        }
    }

    pub fn eq(to: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            negated: false,
        }
    }

    pub fn neq(to: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            negated: true,
        }
    }
}

impl PropertyFilter for StrEq {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String {
        let to = var_allocator.alloc(self.to.to_string());
        let inner = format!(
            "{{
            eq: {to},
        }}"
        );
        if self.negated {
            format!("{{not: {}}}", inner)
        } else {
            inner
        }
    }
}

#[derive(Clone)]
pub struct StrRegex {
    to: String,
    negated: bool,
}

impl StrRegex {
    pub fn new(to: impl Into<String>, negated: bool) -> Self {
        Self {
            to: to.into(),
            negated,
        }
    }
}

impl PropertyFilter for StrRegex {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String {
        let to = var_allocator.alloc(self.to.to_string());
        let inner = format!(
            "{{
            regexp: {to},
        }}"
        );
        if self.negated {
            format!("{{not: {}}}", inner)
        } else {
            inner
        }
    }
}

impl PropertyFilter for Vec<Box<dyn PropertyFilter>> {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String {
        match self.as_slice() {
            [] => "has".to_string(),
            [filter] => filter.to_filter(var_allocator),
            filters => {
                let ands = filters
                    .iter()
                    .map(|filter| filter.to_filter(var_allocator))
                    .collect::<Vec<_>>()
                    .join(",");

                format!("{{and: [{}]}}", ands)
            }
        }
    }
}

impl PropertyFilter for Vec<Vec<Box<dyn PropertyFilter>>> {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String {
        match self.as_slice() {
            [] => "has".to_string(),
            [filter] => filter.to_filter(var_allocator),
            filters => {
                let ors = filters
                    .iter()
                    .map(|filter| filter.to_filter(var_allocator))
                    .collect::<Vec<_>>()
                    .join(",");

                format!("{{or: [{}]}}", ors)
            }
        }
    }
}

#[derive(Default)]
pub struct NodeQuery {
    property_filters: HashMap<String, Vec<Vec<Box<dyn PropertyFilter>>>>,
    edge_filters: HashMap<String, Vec<NodeQuery>>,
}

impl NodeQuery {
    pub fn with_property_filters(
        mut self,
        property_name: impl Into<String>,
        filters: Vec<Box<dyn PropertyFilter>>,
    ) -> Self {
        let queries = self
            .property_filters
            .entry(property_name.into())
            .or_insert_with(|| Vec::with_capacity(1));
        queries.push(filters);
        self
    }

    pub fn with_edge_filters(
        mut self,
        property_name: impl Into<String>,
        filters: Vec<NodeQuery>,
    ) -> Self {
        let queries = self
            .edge_filters
            .entry(property_name.into())
            .or_insert_with(|| Vec::with_capacity(1));
        queries.extend(filters);
        self
    }

    fn to_query(&self, var_allocator: &mut VarAllocator, query_string: &mut String) {
        query_string.push_str("(filter: ");
        query_string.push_str(&self.to_filter(var_allocator));
        query_string.push(')');

        query_string.push('{');
        for property_name in self.property_filters.keys() {
            query_string.push_str(property_name);
            query_string.push('\n');
        }
        for (edge_name, edges) in self.edge_filters.iter() {
            for edge in edges {
                query_string.push('\n');
                query_string.push_str(edge_name);
                edge.to_query(var_allocator, query_string);
            }
        }
        query_string.push('}');
    }

    pub fn to_root_query(&self) -> String {
        let mut var_allocator = VarAllocator::default();
        let mut query_string = String::new();

        self.to_query(&mut var_allocator, &mut query_string);

        let arguments = var_allocator.variable_string();
        format!(
            r#"
            query post({}) {{
                root{}
            }}"#,
            arguments, query_string
        )
    }
}

impl NodeFilter for NodeQuery {
    fn to_filter(&self, var_allocator: &mut VarAllocator) -> String {
        let mut all_filters = Vec::with_capacity(self.property_filters.len());
        for (property_name, property_filters) in self.property_filters.iter() {
            let filters = property_filters.to_filter(var_allocator);
            all_filters.push(format!("{{{}: {}}}", property_name, filters));
        }

        if all_filters.len() == 1 {
            all_filters.into_iter().next().unwrap()
        } else {
            format!("[{}]", all_filters.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use graphql_parser::query::{
        parse_query,
        Document,
    };

    use super::*;

    #[test]
    fn test_filter() -> Result<(), Box<dyn std::error::Error>> {
        let q = NodeQuery::default()
            .with_property_filters("propname", vec![StrEq::eq("foo").boxed()])
            .with_edge_filters(
                "edgename".to_string(),
                vec![NodeQuery::default()
                    .with_property_filters("otherprop", vec![StrEq::neq("baz").boxed()])],
            );

        let query = q.to_root_query();
        println!("{}", query);

        let _query: Document<&str> = parse_query(&query)?;
        Ok(())
    }
}
