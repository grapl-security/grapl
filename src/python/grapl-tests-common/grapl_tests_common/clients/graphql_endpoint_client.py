import os
from typing import Any, Dict

import requests


class GraphqlEndpointClient:
    def __init__(self, jwt: str) -> None:
        hostname = "grapl-graphql-endpoint"
        port = os.environ["GRAPL_GRAPHQL_PORT"]
        self.endpoint = f"http://{hostname}:{port}"
        self.jwt = jwt

    def query(self, query: str) -> Dict[str, Any]:
        resp = requests.post(
            f"{self.endpoint}/graphQlEndpoint/graphql",
            params={"query": query},
            cookies={"grapl_jwt": self.jwt},
        )
        return resp.json()["data"]
