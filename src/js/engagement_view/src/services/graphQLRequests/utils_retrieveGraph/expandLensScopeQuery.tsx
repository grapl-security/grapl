export const expandLensScopeQuery = (lensName: string) => {
    const query = `
    {
        lens_scope(lens_name: "${lensName}") {
            uid,
            node_key,
            lens_type,
            dgraph_type,
            score,
            scope {
                ... on Process {
                    uid,
                    dgraph_type,
                    process_name, 
                    process_id,
                    children {
                        uid, 
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
                    dgraph_type,
                    hostname,
                    asset_ip{
                        ip_address
                    }, 
                    asset_processes{
                        uid, 
                        dgraph_type,
                        process_name, 
                        process_id,
                    },
                    files_on_asset{
                        uid, 
                        dgraph_type,
                        file_path
                    }, 
                    risks {  
                        uid,
                        dgraph_type,
                        analyzer_name, 
                        risk_score
                    },
                }
                ... on File {
                    uid,
                    dgraph_type,
                    risks {  
                        uid,
                        dgraph_type,
                        node_key, 
                        analyzer_name, 
                        risk_score
                    },
                }
            }
        }
    }
`;
    return query;
}
