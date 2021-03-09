use grapl_graph_descriptions::IdentifiedEdge;

use crate::mutations::escape::Escaped;

#[derive(Default)]
pub struct EdgeUpsertGenerator {
    query_buffer: String,
    mutations: Vec<dgraph_tonic::Mutation>,
}

pub(crate) struct IdentifiedEdgeQueries {
    f_src_node: String,
    f_dst_node: String,
    f_src_edge: String,
    f_dst_edge: String,
}

impl EdgeUpsertGenerator {
    pub fn generate_upserts(
        &mut self,
        f_edge: &IdentifiedEdge,
        r_edge: &IdentifiedEdge,
    ) -> (&str, &[dgraph_tonic::Mutation]) {
        let queries = self.gen_edge_queries(f_edge, r_edge);
        self.gen_edge_mutations(&queries, &f_edge.edge_name, &r_edge.edge_name);

        (&self.query_buffer, &self.mutations)
    }

    pub(crate) fn gen_edge_queries(&mut self, f_edge: &IdentifiedEdge, r_edge: &IdentifiedEdge) -> IdentifiedEdgeQueries {
        self.query_buffer.clear();
        let query = format!(
            r#"
            {{
                f_src_node as var(func: uid({src_uid}), first: 1)
                f_dst_node as var(func: uid({dest_uid}), first: 1)

                f_src_edge as var(func: uid(f_src_node)) @cascade {{
                    {f_edge_name} @filter(uid(f_dst_node)) {{
                        uid
                    }}
                }}

                f_dst_edge as var(func: uid(f_dst_node)) @cascade {{
                    {r_edge_name} @filter(uid(f_src_node)) {{
                        uid
                    }}
                }}
            }}
            "#,
            src_uid = Escaped::from(&f_edge.from_uid),
            dest_uid = Escaped::from(&f_edge.to_uid),
            f_edge_name = f_edge.edge_name,
            r_edge_name = r_edge.edge_name,
        );
        self.query_buffer.push_str(&query);
        IdentifiedEdgeQueries {
            f_src_node: "f_src_node".to_string(),
            f_dst_node: "f_dst_node".to_string(),
            f_src_edge: "f_src_edge".to_string(),
            f_dst_edge: "f_dst_edge".to_string(),
        }
    }

    pub(crate) fn gen_edge_mutations(
        &mut self,
        edge_queries: &IdentifiedEdgeQueries,
        f_edge_name: &str,
        r_edge_name: &str,
    ) {
        self.mutations.clear();

        let mut mu_0 = dgraph_tonic::Mutation::new();

        let mu_0_n_quads = format!(
            r#"uid({f_src_node}) <{f_edge_name}> uid({f_dst_node}) ."#,
            f_src_node = edge_queries.f_src_node,
            f_edge_name = f_edge_name,
            f_dst_node = edge_queries.f_dst_node,
        );
        mu_0.set_set_nquads(mu_0_n_quads);
        mu_0.set_cond(format!(
            "@if(eq(len({f_src_edge}), 0))",
            f_src_edge = edge_queries.f_src_edge
        ));

        let mut mu_1 = dgraph_tonic::Mutation::new();

        let mu_1_n_quads = format!(
            r#"uid({f_dst_node}) <{r_edge_name}> uid({f_src_node}) ."#,
            f_dst_node = edge_queries.f_dst_node,
            r_edge_name = r_edge_name,
            f_src_node = edge_queries.f_src_node,
        );

        mu_1.set_set_nquads(mu_1_n_quads);
        mu_1.set_cond(format!(
            "@if(eq(len({f_dst_edge}), 0))",
            f_dst_edge = edge_queries.f_dst_edge
        ));

        self.mutations.extend_from_slice(&[mu_0, mu_1]);
    }
}
