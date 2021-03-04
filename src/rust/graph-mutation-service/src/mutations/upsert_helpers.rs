use dgraph_tonic;
use crate::mutations::escape::Escaped;

pub(crate) fn gen_query(query_buffer: &mut String, cmp: &str, creation_var_name: &str, query_suffix: &str, prop_name: &str, prop_alias: &str, value: &Escaped) -> (String, String) {
    query_buffer.clear();
    let set_query_name = format!("exists_and_set_{query_suffix}", query_suffix=query_suffix);
    let cmp_query_name = format!("exists_but_cmp_{query_suffix}", query_suffix=query_suffix);
    let query = format!(
        r#"
            {set_query_name} as var(func: uid({creation_var_name}), first: 1) @cascade
            {{
                {prop_name}
            }}

            var(func: {cmp}(val({prop_alias}), {value}), first: 1)
            {{
                {cmp_query_name} as uid
            }}
            "#,
        cmp=cmp,
        creation_var_name=creation_var_name,
        prop_alias=prop_alias,
        set_query_name=set_query_name,
        cmp_query_name=cmp_query_name,
        prop_name = prop_name,
        value = value,
    );
    query_buffer.push_str(&query);
    (set_query_name, cmp_query_name)
}

pub(crate) fn gen_mutations(mutations: &mut Vec<dgraph_tonic::Mutation>, node_exists: &str, set_query_name: &str, cmp_query_name: &str, prop_name: &str, prop_value: &Escaped) {
    mutations.clear();
    let mut mu_0 = dgraph_tonic::Mutation::new();

    // If the value is smaller than the int we're updating with, which also implies the node exists
    let mut mu_0_n_quads = format!(
        r#"uid({node_exists}) <{prop_name}> "{prop_value}" ."#,
        node_exists = node_exists,
        prop_name = prop_name,
        prop_value = prop_value,
    );

    mu_0.set_set_nquads(mu_0_n_quads);
    mu_0.set_cond(
        format!("@if(eq(len({cmp_query_name}), 1))", cmp_query_name = cmp_query_name)
    );

    // If the node exists but the value is still unset
    let mut mu_1 = dgraph_tonic::Mutation::new();

    let mut mu_1_n_quads = format!(
        r#"uid({node_exists}) <{prop_name}> "{prop_value}" ."#,
        node_exists = node_exists,
        prop_name = prop_name,
        prop_value = prop_value,
    );

    mu_1.set_set_nquads(mu_1_n_quads);
    mu_1.set_cond(
        format!("@if(eq(len({node_exists}), 1) AND eq(len({set_query_name}), 0))", node_exists = node_exists, set_query_name = set_query_name)
    );

    let mut mu_2 = dgraph_tonic::Mutation::new();

    // condition if the node does not exist
    let mut mu_2_n_quads = format!(
        concat!(
        r#"_:{node_exists} <{prop_name}> "{prop_value}" ."#,
        ),
        node_exists = node_exists,
        prop_name = prop_name,
        prop_value = prop_value,
    );

    mu_2.set_set_nquads(mu_2_n_quads);
    mu_2.set_cond(format!("@if(eq(len({node_exists}), 0))", node_exists = node_exists));

    mutations.extend_from_slice(&[mu_0, mu_1, mu_2]);
}

pub(crate) fn gen_immutable_query(query_buffer: &mut String, creation_var_name: &str, node_id: u128, query_suffix: &str, prop_name: &str) -> String {
    query_buffer.clear();
    let query_name = format!("exists_but_unset_{query_suffix}", query_suffix=query_suffix);
    // todo - right now we "use" the `val`, but we don't need to
    let query = format!(
        r#"
            {query_name} as var(func: uid({creation_var_name}), first: 1) @cascade
            {{
                {prop_name}
                val({prop_name}_{node_id})
            }}
            "#,
        creation_var_name=creation_var_name,
        query_name = query_name,
        prop_name = prop_name,
        node_id = node_id,
    );

    query_buffer.push_str(&query);
    query_name
}

pub(crate) fn gen_immutable_mutations(mutations: &mut Vec<dgraph_tonic::Mutation>, node_exists: &str, query_name: &str, prop_name: &str, prop_value: &Escaped) {
    mutations.clear();
    let mut mu_0 = dgraph_tonic::Mutation::new();

    let mut mu_0_n_quads = format!(
        r#"uid({node_exists}) <{prop_name}> "{prop_value}" ."#,
        node_exists = node_exists,
        prop_name = prop_name,
        prop_value = prop_value,
    );

    // If the value is unset (since we filter on it having a value)
    mu_0.set_set_nquads(mu_0_n_quads);
    mu_0.set_cond(
        format!("@if(eq(len({node_exists}), 1) AND eq(len({query_name}), 0))", node_exists = node_exists, query_name = query_name)
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

    mutations.extend_from_slice(&[mu_0, mu_1]);
}
