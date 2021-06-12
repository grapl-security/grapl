import json
from http import HTTPStatus
from typing import Any, Dict, Optional, cast

import requests
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_tests_common.clients.common import endpoint_url

# Would be nice to improve this as a TypedDict
GqlLensDict = Dict[str, Any]
GraphqlType = Dict[str, Any]

LOGGER = get_module_grapl_logger(log_to_stdout=True)


class GraphQLException(Exception):
    pass


class GraphqlEndpointClient:
    def __init__(self, jwt: str) -> None:
        self.endpoint = endpoint_url("/graphQlEndpoint")
        self.jwt = jwt

    def query(
        self, query: str, variables: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        resp = requests.post(
            f"{self.endpoint}/graphql",
            params={"query": query, "variables": json.dumps(variables or {})},
            cookies={"grapl_jwt": self.jwt},
        )
        if resp.status_code != HTTPStatus.OK:
            resp_str = "\\n".join(resp.iter_lines(decode_unicode=True))
            raise AssertionError(
                f'Status {resp.status_code} from graphql endpoint for query "{query}" with variables "{variables}"\n'
                f'Response: "{resp_str or "no response"}"'
            )
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
