import json
import os
from http import HTTPStatus
from typing import Any, Dict, Optional, cast

import requests


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
