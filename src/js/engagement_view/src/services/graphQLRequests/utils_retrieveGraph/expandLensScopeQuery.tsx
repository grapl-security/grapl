export const expandLensScopeQuery = (lensName: string) => {
    const query = `
    {
        lens_scope(lens_name: "${lensName}") {
            uid,
            node_key,
            lens_type,
            dgraph_type,
            score,
            display,
            scope {
                ... on Process {
                    uid,
                    node_key, 
                    dgraph_type,
                    process_name, 
                    process_id,
                    display,
                    children {
                        uid, 
                        node_key, 
                        dgraph_type,
                        display,
                        process_name, 
                        process_id,
                    }, 
                    risks {  
                        uid,
                        dgraph_type,
                        node_key, 
                        analyzer_name, 
                        risk_score
                    },
                }
                ... on Asset {
                    uid, 
                    node_key, 
                    dgraph_type,
                    display,
                    hostname,
                    asset_ip{
                        ip_address
                    }, 
                    asset_processes{
                        uid, 
                        node_key, 
                        dgraph_type,
                        display,
                        process_name, 
                        process_id,
                    },
                    files_on_asset{
                        uid, 
                        node_key, 
                        dgraph_type,
                        display,
                        file_path
                    }, 
                    risks {  
                        uid,
                        dgraph_type,
                        display,
                        node_key, 
                        analyzer_name, 
                        risk_score
                    },
                }
                ... on File {
                    uid,
                    node_key, 
                    dgraph_type,
                    display,
                    risks {  
                        uid,
                        dgraph_type,
                        display,
                        node_key, 
                        analyzer_name, 
                        risk_score
                    },
                }
                ... on PluginType {
                    predicates,
                }
            }
        }
    }
`;
    return query;
}