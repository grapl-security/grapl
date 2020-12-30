
export const expandScopeQuery = (lensName: string) => {
    console.log("expanding scope for: ", lensName);

    const query = `
    {
        lens_scope(lens_name: "${lensName}") {
            uid,
            node_key,
            lens_name,
            lens_type,
            dgraph_type,
            scope {
                ... on Process {
                    uid,
                    node_key, 
                    dgraph_type,
                    process_name, 
                    process_id,
                    children {
                        uid, 
                        node_key, 
                        dgraph_type,
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
                    hostname,
                    asset_ip{
                        ip_address
                    }, 
                    asset_processes{
                        uid, 
                        node_key, 
                        dgraph_type,
                        process_name, 
                        process_id,
                    },
                    files_on_asset{
                        uid, 
                        node_key, 
                        dgraph_type,
                        file_path
                    }, 
                    risks {  
                        uid,
                        dgraph_type,
                        node_key, 
                        analyzer_name, 
                        risk_score
                    },
                }

                ... on File {
                    uid,
                    node_key, 
                    dgraph_type,
                    risks {  
                        uid,
                        dgraph_type,
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
