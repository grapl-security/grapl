use grapl_graph_descriptions::IncrementOnlyIntProp;

use crate::mutations::{
    escape::Escaped,
    upsert_helpers::{
        gen_mutations,
        gen_query,
    },
    QueryInput,
    UpsertGenerator,
};

#[derive(Default)]
pub struct IncrementOnlyIntUpsertGenerator {
    query_buffer: String,
    mutations: Vec<dgraph_tonic::Mutation>,
}

impl UpsertGenerator for IncrementOnlyIntUpsertGenerator {
    type Input = IncrementOnlyIntProp;
    fn generate_upserts(
        &mut self,
        creation_query: &QueryInput<'_>,
        predicate_name: &str,
        value: &Self::Input,
    ) -> (&str, &[dgraph_tonic::Mutation]) {
        let IncrementOnlyIntProp { prop: ref value } = value;
        let value = Escaped::from(value);
        let query_suffix = format!(
            "{}_{}_{}",
            &creation_query.unique_id, &creation_query.node_id, &creation_query.predicate_id
        );
        let (set_query_name, cmp_query_name) = gen_query(
            &mut self.query_buffer,
            "lt",
            &creation_query.creation_query_name,
            &query_suffix,
            predicate_name,
            &format!("{}_{}", predicate_name, creation_query.node_id),
            &value,
        );

        gen_mutations(
            &mut self.mutations,
            &creation_query.creation_query_name,
            &set_query_name,
            &cmp_query_name,
            &predicate_name,
            &value,
        );

        (&self.query_buffer, &self.mutations)
    }
}
