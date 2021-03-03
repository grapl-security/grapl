use crate::mutations::{QueryInput, UpsertGenerator};
use crate::mutations::escape::{Escaped, escape_quote};
use grapl_graph_descriptions::IdentifiedNode;
use crate::mutations::predicate_mutation::NodePropertyUpsertGenerator;

#[derive(Default)]
pub struct NodeUpsertGenerator {
    pub query_buffer: String,
    pub mutations: Vec<dgraph_tonic::Mutation>,
    pub node_property_upsert_generator: NodePropertyUpsertGenerator,
}

impl NodeUpsertGenerator {
    pub fn generate_upserts(&mut self, unique_id: u128, node_id: u128, value: &IdentifiedNode) -> (String, &str, &[dgraph_tonic::Mutation]) {
        let node_key = escape_quote(&value.node_key);
        let creation_query_name = self.gen_query(
            node_id,
            &node_key,
        );
        self.node_creation_quads(
            &creation_query_name,
            &node_key,
            &value.node_type,
        );
        for (predicate_id, (predicate_name, predicate_value)) in value.properties.iter().enumerate() {
            let query_input = QueryInput {
                creation_query_name: &creation_query_name,
                unique_id,
                node_id,
                predicate_id: predicate_id as u128,
            };
            let (predicate_query, mutations) = self.node_property_upsert_generator.generate_upserts(
                &query_input,
                predicate_name,
                predicate_value,
            );
            self.query_buffer.push('\n');
            self.query_buffer.push_str(predicate_query);
            self.mutations.extend_from_slice(mutations);
        }

        println!("---query---");
        println!("{}", self.query_buffer);
        println!("---mutations---");
        println!("{}", format_mutations(&self.mutations));
        println!("------");


        (creation_query_name, &self.query_buffer, &self.mutations)
    }
}

fn format_mutations(muts: &[dgraph_tonic::Mutation]) -> String {
    let mut output = String::new();
    for next_mut in muts {
        let fmt = format!("nquads: {}\ncond: {:?}",
            String::from_utf8_lossy(&next_mut.set_nquads),
            next_mut.cond,
        );
        output.push_str(&fmt);
    }
    output
}

impl NodeUpsertGenerator {
    fn gen_query(&mut self, node_id: u128, node_key: &Escaped) -> String {
        self.query_buffer.clear();
        let creation_var_name = format!("node_exists_{node_id}", node_id = node_id);
        let inner_query = format!(
            r#"
            var(func: eq(node_key, {node_key}), first: 1) @cascade {{
                {creation_var_name} as uid,
            }}
            q_{creation_var_name}(func: uid({creation_var_name}), first: 1) @cascade
            {{
                uid,
                node_key,
            }}
    "#,
            creation_var_name=creation_var_name,
            node_key=node_key,
        );
        self.query_buffer.push_str(&inner_query);
        creation_var_name
    }


    pub(crate) fn node_creation_quads(
        &mut self,
        creation_var_name: &str,
        node_key: &Escaped,
        node_type: &str,
    ) {
        // If the node exists, do nothing, otherwise create it with its type
        let mut mu_1 = dgraph_tonic::Mutation::new();
        let mut mu_1_n_quads = format!(
            concat!(
            r#"_:{creation_var_name} <node_key> {node_key} ."#,
            "\n",
            r#"_:{creation_var_name} <dgraph.type> "{node_type}" ."#,
            ),
            node_key = node_key,
            node_type = node_type,
            creation_var_name = creation_var_name,
        );

        mu_1.set_set_nquads(mu_1_n_quads);
        mu_1.set_cond(format!("@if(eq(len({creation_var_name}), 0))", creation_var_name=creation_var_name));

        self.mutations.push(mu_1);
    }

}
