use crate::mutations::{UpsertGenerator, QueryInput};
use grapl_graph_descriptions::Edge;

#[derive(Default)]
pub struct EdgeUpsertGenerator {
    query_buffer: String,
    mutations: Vec<dgraph_tonic::Mutation>,
}

impl UpsertGenerator for EdgeUpsertGenerator {
    type Input = (Edge, Edge);
    fn generate_upserts(&mut self, creation_query: &QueryInput<'_>, predicate_name: &str, value: &Self::Input) -> (&str, &[dgraph_tonic::Mutation]) {
        let (f_edge, r_edge) = value;

        // let value = Escaped::from(value);
        // let query_suffix = format!("{}_{}_{}", &creation_query.unique_id, &creation_query.node_id, &creation_query.predicate_id);
        // let (set_query_name, cmp_query_name) = gen_query(
        //     &mut self.query_buffer,
        //     "gt",
        //     &creation_query.creation_query_name,
        //     &query_suffix,
        //     predicate_name,
        //     &format!("{}_{}", predicate_name, creation_query.node_id),
        //     &value,
        // );
        // 
        // gen_mutations(
        //     &mut self.mutations,
        //     &creation_query.creation_query_name,
        //     &set_query_name,
        //     &cmp_query_name,
        //     &predicate_name,
        //     &value,
        // );

        (&self.query_buffer, &self.mutations)
    }
}

impl EdgeUpsertGenerator {
    pub(crate) fn gen_edge_queries(&mut self, f_edge: &Edge, r_edge: &Edge) -> String {
        self.query_buffer.clear();
        // todo - right now we "use" the `val`, but we don't need to
        let query = format!(
            r#"
            f_src_node as var(func: eq(node_key, {from_node_key}), first: 1) {{
                {f_edge_name} {{
                    f_dest_edge as uid,
                }}
            }}

            r_src_node as var(func: eq(node_key, {dest_node_key}), first: 1) {{
                {r_edge_name} {{
                    r_dest_edge as uid,
                }}
            }}
            "#,
            from_node_key = f_edge.from_node_key,
            dest_node_key = f_edge.to_node_key,
            f_edge_name = f_edge.edge_name,
            r_edge_name = r_edge.edge_name,
        );

        self.query_buffer.push_str(&query);
        query_name
    }
}
