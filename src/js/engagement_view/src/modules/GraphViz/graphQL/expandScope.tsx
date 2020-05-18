import {BaseNode, LensScopeResponse} from '../../GraphViz/CustomTypes'

const isLocal = true;

export const getEngagementEdge = (port?: undefined | string) => {
    if (isLocal) {
        return "http://" + window.location.hostname + (port || ":8900/") 
    } else {
        return "__engagement_ux_standin__hostname__"
    }
}

const graphql_edge = getEngagementEdge(":5000/");

export const retrieveGraph = async (lens: string): Promise<(LensScopeResponse & BaseNode)> => {
    const query = expandScope(lens);

    const res = await fetch(`${graphql_edge}graphql`,
        {
            method: 'post',
            body: JSON.stringify({ query }),
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
        })
        .then(res => res.json())
        .then((res) => res.data)
        .then((res) => res.lens_scope);

    return (await res);
};

export const expandScope = (lensName: string) => {
    
    const query = `
    {
        lens_scope(lens_name: "${lensName}") {
            uid,
            node_key,
            lens_name,
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
            }
        }
    }
`;

    return query;
}

