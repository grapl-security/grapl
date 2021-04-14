import json
import os
from http import HTTPStatus
from typing import Any, Dict, Optional, cast

import requests

# Would be nice to improve this as a TypedDict
GqlLensDict = Dict[str, Any]
GraphqlType = Dict[str, Any]


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
        query = self.get_scope_query()
        resp = self.query(query, {"lens_name": lens_name})
        return cast(GqlLensDict, resp["lens_scope"])

    def get_scope_query(self) -> str:
        query = """
        {
            lens_scope_query {
                query_string
            }
        }
        """
        resp = self.query(query)
        return cast(str, resp["lens_scope_query"]["query_string"])

    def query_type(self, type_name: str) -> GraphqlType:
        query = """
        query QueryGraphqlAboutType($type_name: String!) {
            __type(name: $type_name) {
                name
                fields {
                    name
                    type {
                        name
                        kind
                    }
                }
            }
        }
        """
        resp = self.query(query, {"type_name": type_name})
        return cast(GraphqlType, resp["__type"])
