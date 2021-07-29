import logging
import os
from pathlib import Path

from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.clients.model_plugin_deployer_client import (
    ModelPluginDeployerClient,
)


def test_upload_plugin(jwt: str) -> None:
    # We haven't uploaed `schemas.py` yet, so IamRole shouldn't exist in
    # the graphql schema.
    gql_client = GraphqlEndpointClient(jwt)
    assert "IamRole" not in gql_client.get_scope_query()

    _upload_model_plugin(
        model_plugin_client=ModelPluginDeployerClient.from_env(), jwt=jwt
    )

    # After uploading plugin, we'd expect to see it.
    assert "IamRole" in gql_client.get_scope_query()


def _upload_model_plugin(
    model_plugin_client: ModelPluginDeployerClient,
    jwt: str,
) -> None:
    logging.info("Making request to /deploy to upload model plugins")
    plugin_path = "/tmp/schemas"
    files = os.listdir(plugin_path)
    assert "schemas.py" in files, f"Did not find schemas.py file in {files}"
    plugin_upload = model_plugin_client.deploy(
        Path(plugin_path),
        jwt,
    )
    logging.info(f"UploadRequest: {plugin_upload.json()}")
    upload_status = plugin_upload.json()["success"]["Success"] == True
    assert upload_status
