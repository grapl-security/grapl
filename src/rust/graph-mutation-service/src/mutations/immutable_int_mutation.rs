use crate::mutations::{UpsertGenerator, QueryInput};
use crate::v1beta1::ImmutableIntProp;
use crate::mutations::escape::{Escaped, escape_quote};

pub struct ImmutableIntUpsertGenerator {
    query_buffer: String,
    mutations: Vec<dgraph_tonic::Mutation>,
}

impl UpsertGenerator for ImmutableIntUpsertGenerator {
    type Input = ImmutableIntProp;
    fn generate_upserts(&mut self, creation_query: &QueryInput<'_>, predicate_name: &str, value: &Self::Input) -> (&str, &[dgraph_tonic::Mutation]) {
        let predicate_name = escape_quote(predicate_name);
        let ImmutableIntProp {prop: ref value} = value;
        let value = Escaped::from(value);
        let query_suffix = format!("{}_{}_{}", &creation_query.unique_id, &creation_query.node_id, &creation_query.predicate_id);
        let query_name = self.gen_query(
            &creation_query.creation_query_name,
            &query_suffix,
            &predicate_name,
            &value,
        );

        self.gen_mutations(
            &creation_query.creation_query_name,
            &query_name,
            &predicate_name,
            &value,
        );

        (&self.query_buffer, &self.mutations)
    }
}


impl ImmutableIntUpsertGenerator {
    fn gen_query(&mut self, node_exists: &str, query_suffix: &str, prop_name: &Escaped, value: &Escaped) -> String {
        self.query_buffer.clear();
        let query_name = format!("exists_but_unset_{query_suffix}", query_suffix=query_suffix);
        let query = format!(
            r#"
            {query_name} as var(func: uid({node_exists}), first: 1) @cascade
                @filter(NOT eq({prop_name}, {value}))
            {{
                uid
            }}
            "#,
            query_name = query_name,
            node_exists = node_exists,
            prop_name = prop_name,
            value = value,
        );
        self.query_buffer.push_str(&query);
        query_name
    }

    fn gen_mutations(&mut self, node_exists: &str, query_name: &str, prop_name: &Escaped, prop_value: &Escaped) {
        self.mutations.clear();
        let mut mu_0 = dgraph_tonic::Mutation::new();

        let mut mu_0_n_quads = format!(
            r#"uid({node_exists}) <{prop_name}> "{prop_value}" ."#,
            node_exists = node_exists,
            prop_name = prop_name,
            prop_value = prop_value,
        );

        mu_0.set_set_nquads(mu_0_n_quads);
        mu_0.set_cond(
            format!("@if(eq(len({query_name}), 0) AND eq(len({node_exists}), 1))", query_name = query_name, node_exists = node_exists)
        );

        let mut mu_1 = dgraph_tonic::Mutation::new();

        // condition if the node does not exist
        let mut mu_1_n_quads = format!(
            concat!(
            r#"_:{node_exists} <{prop_name}> "{prop_value}" ."#,
            ),
            node_exists = node_exists,
            prop_name = prop_name,
            prop_value = prop_value,
        );

        mu_1.set_set_nquads(mu_1_n_quads);
        mu_1.set_cond(format!("@if(eq(len({node_exists}), 0))", node_exists = node_exists));
        self.mutations.extend_from_slice(&[mu_0, mu_1]);
    }

}
