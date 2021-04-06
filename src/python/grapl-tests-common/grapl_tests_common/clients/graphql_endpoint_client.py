import json
import os
from http import HTTPStatus
from typing import Any, Dict, Optional, cast

import requests

GqlLensDict = Dict[str, Any]


class GraphqlEndpointClient:
    def __init__(self, jwt: str) -> None:
        hostname = os.environ["GRAPL_GRAPHQL_HOST"]
        port = os.environ["GRAPL_GRAPHQL_PORT"]
        self.endpoint = f"http://{hostname}:{port}"
        self.jwt = jwt

    def query(
        self, query: str, variables: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        resp = requests.post(
            f"{self.endpoint}/graphQlEndpoint/graphql",
            params={"query": query, "variables": json.dumps(variables or {})},
            cookies={"grapl_jwt": self.jwt},
        )
        assert resp.status_code == HTTPStatus.OK, resp.json()
        return cast(Dict[str, Any], resp.json()["data"])

    def query_for_scope(self, lens_name: str) -> GqlLensDict:
        # This query is based off the lens_scope query in /expandLensScopeQuery.tsx

        query = """
        query LensScopeByName($lens_name: String!) {
            
            lens_scope(lens_name: $lens_name) {
                uid,
                node_key,
                lens_type,
                dgraph_type,
                score,
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
        """
        resp = self.query(query, {"lens_name": lens_name})
        return resp["lens_scope"]
