import os
from typing import Any, Dict, cast

import requests


class GraphqlEndpointClient:
    def __init__(self, jwt: str) -> None:
        hostname = os.environ["GRAPL_GRAPHQL_HOST"]
        port = os.environ["GRAPL_GRAPHQL_PORT"]
        self.endpoint = f"http://{hostname}:{port}"
        self.jwt = jwt

    def query(self, query: str) -> Dict[str, Any]:
        resp = requests.post(
            f"{self.endpoint}/graphQlEndpoint/graphql",
            params={"query": query},
            cookies={"grapl_jwt": self.jwt},
        )
        return cast(Dict[str, Any], resp.json()["data"])
