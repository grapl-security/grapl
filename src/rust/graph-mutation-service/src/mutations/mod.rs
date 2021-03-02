pub mod predicate_mutation;
pub mod node_mutation;
pub mod immutable_string_mutation;
pub mod immutable_uint_mutation;
pub mod immutable_int_mutation;
pub mod incr_only_uint_mutation;
pub mod escape;

pub struct QueryInput<'a> {
    creation_query_name: &'a str,
    unique_id: u128,
    node_id: u128,
    predicate_id: u128,
}

pub trait UpsertGenerator {
    type Input;
    fn generate_upserts(&mut self, creation_query: &QueryInput<'_>, predicate_name: &str, value: &Self::Input) -> (&str, &[dgraph_tonic::Mutation]);
}
