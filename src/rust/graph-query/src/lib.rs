pub mod node_iterator;
mod var_allocator;

use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashSet,
    ops::Deref,
    rc::Rc,
};

use fnv::FnvHashMap as HashMap;

use crate::{
    node_iterator::NodeIterator,
    var_allocator::VarAllocator,
};

pub trait PropertyFilter {
    // For a given property, pushes a dgraph filter into `filter_string`,
    // generating a GraphQL variable via `var_allocator` if necessary
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        var_allocator: &mut VarAllocator,
    );
    fn boxed(self) -> Box<dyn PropertyFilter>
    where
        Self: Sized + 'static,
    {
        Box::new(self) as Box<dyn PropertyFilter>
    }
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
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        _var_allocator: &mut VarAllocator,
    ) {
        if self.negated {
            filter_string.push_str("NOT(");
        }
        filter_string.push_str("eq(");
        filter_string.push_str(property_name);
        filter_string.push(',');
        filter_string.push_str(&self.to.to_string());
        filter_string.push(')');
        if self.negated {
            filter_string.push(')');
        }
    }
}

#[derive(Clone)]
pub struct UidEq {
    to: u64,
}

impl UidEq {
    pub fn eq(to: u64) -> Self {
        Self { to }
    }
}

impl PropertyFilter for UidEq {
    fn extend_filter_string(
        &self,
        _property_name: &str,
        filter_string: &mut String,
        _var_allocator: &mut VarAllocator,
    ) {
        filter_string.push_str("uid(");
        filter_string.push_str(&self.to.to_string());
        filter_string.push(')');
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
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        _var_allocator: &mut VarAllocator,
    ) {
        if self.negated {
            filter_string.push_str("NOT(");
        }
        filter_string.push_str("lt(");
        filter_string.push_str(property_name);
        filter_string.push(',');
        filter_string.push_str(&self.to.to_string());
        filter_string.push(')');
        if self.negated {
            filter_string.push(')');
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
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        _var_allocator: &mut VarAllocator,
    ) {
        if self.negated {
            filter_string.push_str("NOT(");
        }
        filter_string.push_str("gt(");
        filter_string.push_str(property_name);
        filter_string.push(',');
        filter_string.push_str(&self.to.to_string());
        filter_string.push(')');
        if self.negated {
            filter_string.push(')');
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
}

impl PropertyFilter for StrEq {
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        var_allocator: &mut VarAllocator,
    ) {
        let to = var_allocator.alloc(self.to.to_string());
        if self.negated {
            filter_string.push_str("NOT(");
        }
        filter_string.push_str("eq(");
        filter_string.push_str(property_name);
        filter_string.push(',');
        filter_string.push_str(to);
        filter_string.push(')');
        if self.negated {
            filter_string.push(')');
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
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        var_allocator: &mut VarAllocator,
    ) {
        let to = var_allocator.alloc(self.to.to_string());

        if self.negated {
            filter_string.push_str("NOT(");
        }
        filter_string.push_str("regexp(");
        filter_string.push_str(property_name);
        filter_string.push(',');
        filter_string.push_str(to);
        filter_string.push(')');
        if self.negated {
            filter_string.push(')');
        }
    }
}

impl PropertyFilter for Vec<Box<dyn PropertyFilter>> {
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        var_allocator: &mut VarAllocator,
    ) {
        match self.as_slice() {
            [] => (),
            [filter] => filter.extend_filter_string(property_name, filter_string, var_allocator),
            filters => {
                filter_string.push('(');
                for (i, filter) in filters.iter().enumerate() {
                    filter.extend_filter_string(property_name, filter_string, var_allocator);
                    if i < filters.len() - 1 {
                        filter_string.push_str(" AND ");
                    }
                }
                filter_string.push(')');
            }
        }
    }
}

impl PropertyFilter for Vec<Vec<Box<dyn PropertyFilter>>> {
    fn extend_filter_string(
        &self,
        property_name: &str,
        filter_string: &mut String,
        var_allocator: &mut VarAllocator,
    ) {
        match self.as_slice() {
            [] => (),
            [filter] => filter.extend_filter_string(property_name, filter_string, var_allocator),
            filters => {
                filter_string.push(')');
                for (i, filter) in filters.iter().enumerate() {
                    filter.extend_filter_string(property_name, filter_string, var_allocator);
                    if i < filters.len() - 1 {
                        filter_string.push_str(" OR ");
                    }
                }
                filter_string.push(')');
            }
        }
    }
}

pub struct NodeQuery {
    // In order to properly generate queries we need every node to have a unique id
    // A node is the "root" node when its `id` is 0
    id: u128,
    property_filters: HashMap<String, Vec<Vec<Box<dyn PropertyFilter>>>>,
    edge_filters: HashMap<String, Vec<NodeCell>>,
    // Maps forward edge to reverse edge name
    reverse_edge_names: HashMap<String, String>,
}

impl Default for NodeQuery {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().as_u128(),
            property_filters: Default::default(),
            edge_filters: Default::default(),
            reverse_edge_names: Default::default(),
        }
    }
}

impl NodeQuery {
    fn root() -> Self {
        Self {
            id: 0,
            ..Default::default()
        }
    }

    fn extend_filter_string(&self, filter_string: &mut String, var_allocator: &mut VarAllocator) {
        if self.property_filters.len() >= 1 {
            filter_string.push_str("@filter(");
        }
        for (i, (property_name, property_filters)) in self.property_filters.iter().enumerate() {
            // filter_string.push('(');
            property_filters.extend_filter_string(property_name, filter_string, var_allocator);
            // filter_string.push(')');
            if i < self.property_filters.len() - 1 {
                filter_string.push_str(" AND ");
            }
        }

        if self.property_filters.len() >= 1 {
            filter_string.push(')');
        }
    }

    fn to_query(
        &self,
        binding: u8,
        var_allocator: &mut VarAllocator,
        query_string: &mut String,
        visited: &mut HashSet<(u128, String, u128)>,
        should_filter: bool,
    ) {
        if should_filter {
            self.extend_filter_string(query_string, var_allocator);
        }

        query_string.push('{');
        for property_name in self.property_filters.keys() {
            query_string.push('\n');
            query_string.push_str(property_name);
        }
        for (edge_name, edges) in self.edge_filters.iter() {
            for edge in edges {
                let inner_edge = edge.0.clone();
                let inner_edge: &RefCell<NodeQuery> = inner_edge.borrow();
                let inner_edge = inner_edge.borrow();

                if self.already_visited(edge_name, edge.get_id(), visited) {
                    continue;
                }

                query_string.push('\n');
                if edge.is_root() {
                    query_string.push_str("binding_");
                    query_string.push_str(&binding.to_string());
                    query_string.push_str(" as ");
                }
                query_string.push_str(edge_name);
                inner_edge.to_query(binding, var_allocator, query_string, visited, should_filter);
            }
        }
        query_string.push('}');
    }

    // When building our query we need to understand if we've visited a node either through a forward
    // or a reverse edge
    // We can't just check if we have visited the destination node because multiple nodes may reference
    // another node, or a single node may reference another node with different edge names
    // For example, a Process may both Read and Write to a File
    fn already_visited(
        &self,
        edge_name: &str,
        edge_id: u128,
        visited: &mut HashSet<(u128, String, u128)>,
    ) -> bool {
        // Rust's hashmap is kinda annoying about borrowing keys that have nested owned values. Feel free
        // to remove this unnecessary allocation if you're willing to put the work in (I am not).
        // https://users.rust-lang.org/t/hashmap-with-tuple-keys/12711
        let f_key = (self.id, edge_name.to_owned(), edge_id);
        if visited.contains(&f_key) {
            return true;
        }
        visited.insert(f_key);

        let r_key = (
            edge_id,
            self.reverse_edge_names[edge_name].to_owned(),
            self.id,
        );
        if visited.contains(&r_key) {
            return true;
        }
        visited.insert(r_key);

        false
    }
}

#[derive(Clone, Default)]
pub struct NodeCell(Rc<RefCell<NodeQuery>>);

impl Deref for NodeCell {
    type Target = Rc<RefCell<NodeQuery>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NodeQuery> for NodeCell {
    fn from(value: NodeQuery) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl NodeCell {
    pub fn root() -> Self {
        Self::from(NodeQuery::root())
    }

    pub fn with_property_filters(
        self,
        property_name: impl Into<String>,
        filters: Vec<Box<dyn PropertyFilter>>,
    ) -> Self {
        let mut inner = self.0.borrow_mut();
        let queries = inner
            .property_filters
            .entry(property_name.into())
            .or_insert_with(|| Vec::with_capacity(1));
        queries.push(filters);
        drop(inner);
        self
    }

    pub fn with_edge_filters(
        self,
        edge_name: impl Into<String>,
        reverse_edge_name: impl Into<String>,
        mut filters: Vec<NodeCell>,
    ) -> Self {
        // Add a reverse edge from all filters back to self
        let reverse_edge_name = reverse_edge_name.into();
        let edge_name = edge_name.into();

        for filter in filters.iter_mut() {
            assert!(
                !filter.is_root(),
                "Can not construct a graph with multiple root nodes"
            );
            let mut filter = filter.0.borrow_mut();
            filter
                .edge_filters
                .insert(reverse_edge_name.clone(), vec![self.clone()]);
            filter
                .reverse_edge_names
                .insert(reverse_edge_name.clone(), edge_name.clone());
        }

        let mut inner_self = self.0.borrow_mut();
        inner_self
            .reverse_edge_names
            .insert(edge_name.clone(), reverse_edge_name.clone());

        // Add an edge to all neighbors
        let queries = inner_self
            .edge_filters
            .entry(edge_name)
            .or_insert_with(|| Vec::with_capacity(1));
        queries.extend(filters);
        drop(inner_self);

        self
    }

    // `binding` is used to ensure our query variable names are unique
    pub fn to_root_query(
        &self,
        binding: u8,
        var_allocator: &mut VarAllocator,
        should_filter: bool,
    ) -> String {
        let mut query_string = String::new();
        let mut visited = HashSet::new();

        if self.is_root() {
            query_string.push_str("binding_");
            query_string.push_str(&binding.to_string());
            query_string.push_str(" as var");
        } else {
            query_string.push_str("query_");
            query_string.push_str(&binding.to_string());
        }
        query_string.push_str("(func:");
        // generate func
        self.place_func(&mut query_string, var_allocator);
        query_string.push_str(") @cascade");

        let inner_self = self.0.clone();
        let inner_self: &RefCell<NodeQuery> = inner_self.borrow();
        inner_self.borrow().to_query(
            binding,
            var_allocator,
            &mut query_string,
            &mut visited,
            should_filter,
        );

        query_string
    }

    pub fn to_full_root_query(&self, should_filter: bool) -> String {
        let mut var_allocator = VarAllocator::default();
        let query_string = self.to_root_query(0, &mut var_allocator, should_filter);

        let arguments = var_allocator.variable_string();
        format!(
            r#"
            query root_query({}) @cascade {{
                root{}
            }}"#,
            arguments, query_string
        )
    }

    pub fn with_uid(&self, uid: u64) -> (String, VarAllocator) {
        let node = self.clone();
        let mut var_allocator = VarAllocator::default();

        let mut queries = String::new();

        let mut last_binding = 0;
        for (binding, neighbor) in node.iter().enumerate() {
            last_binding = binding;
            let mut neighbor = neighbor.clone();
            // The base query should be constrained to a uid
            neighbor = neighbor.with_property_filters("uid", vec![UidEq::eq(uid).boxed()]);
            // We need to pass in the "true root"
            let next_query = neighbor.to_root_query(binding as u8, &mut var_allocator, true);
            queries.push_str(&next_query);
            queries.push_str("\n\n");
        }

        let mut full_query = format!(
            r#"
        query root({}) {{
            {}

    "#,
            var_allocator.variable_string(),
            queries
        );

        full_query.push_str("coalesce(func: uid(");
        for binding in 0..=last_binding {
            full_query.push_str("binding_");
            full_query.push_str(&binding.to_string());
            if binding < last_binding {
                full_query.push(',');
            }
        }
        full_query.push_str(")) @cascade");
        // generate the "coalescing" query,
        // node.to_query(binding, var_allocator, &mut query_string, &mut visited);
        let root_node = node.0.clone();
        let root_node: &RefCell<NodeQuery> = root_node.borrow();
        let root_node = root_node.borrow();
        let mut visited = HashSet::default();
        root_node.to_query(0, &mut var_allocator, &mut full_query, &mut visited, false);

        full_query.push('}');

        (full_query, var_allocator)
    }

    fn place_func(&self, query_string: &mut String, var_allocator: &mut VarAllocator) {
        let inner_self = self.0.clone();
        let inner_self: &RefCell<NodeQuery> = inner_self.borrow();
        let inner_self = inner_self.borrow();
        // When `with_uid` is called this is guaranteed to be populated
        if let Some(uids) = inner_self.property_filters.get("uid") {
            uids[0][0].extend_filter_string("uid", query_string, var_allocator);
        }
    }

    pub fn iter(&self) -> NodeIterator {
        NodeIterator::new(self.clone())
    }

    pub(crate) fn get_id(&self) -> u128 {
        let inner = self.0.clone();
        let inner: &RefCell<NodeQuery> = inner.borrow();
        let id = inner.borrow().id;
        id
    }

    pub(crate) fn is_root(&self) -> bool {
        self.get_id() == 0
    }
}
