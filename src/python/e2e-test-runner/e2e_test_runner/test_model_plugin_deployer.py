import logging
import os
from pathlib import Path

import pytest
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.model_plugin_deployer_client import (
    ModelPluginDeployerClient,
)


def test_upload_plugin() -> None:
    _upload_model_plugin(model_plugin_client=ModelPluginDeployerClient.from_env())


@pytest.mark.xfail  # TODO: Remove once list plugins is resolved
def test_list_plugin() -> None:
    _get_plugin_list(model_plugin_client=ModelPluginDeployerClient.from_env())


@pytest.mark.xfail  # TODO: once list plugins is resolved, we can fix delete plugins :)
def test_delete_plugin() -> None:
    # Hard Code for now
    _delete_model_plugin(
        model_plugin_client=ModelPluginDeployerClient.from_env(),
        plugin_to_delete="aws_plugin",
    )  # TODO: we need to change the plugin name when this endpoint gets fixed


# The below functions seem nice and composable, so I will keep them separate from
# the above tests (despite the fact that they could all just be 1 thing together)


def _upload_model_plugin(
    model_plugin_client: ModelPluginDeployerClient,
) -> None:
    logging.info("Making request to /deploy to upload model plugins")
    plugin_path = "/tmp/schemas"
    jwt = EngagementEdgeClient().get_jwt()
    files = os.listdir(plugin_path)
    assert "schemas.py" in files, f"Did not find schemas.py file in {files}"
    plugin_upload = model_plugin_client.deploy(
        Path(plugin_path),
        jwt,
    )
    logging.info(f"UploadRequest: {plugin_upload.json()}")
    upload_status = plugin_upload.json()["success"]["Success"] == True
    assert upload_status


def _get_plugin_list(model_plugin_client: ModelPluginDeployerClient) -> None:
    jwt = EngagementEdgeClient().get_jwt()
    get_plugin_list = model_plugin_client.list_plugins(
        jwt,
    )
    logging.info(f"UploadRequest: {get_plugin_list.json()}")
    upload_status = get_plugin_list.json()["success"]["plugin_list"]
    assert upload_status


def _delete_model_plugin(
    model_plugin_client: ModelPluginDeployerClient,
    plugin_to_delete: str,
) -> None:
    jwt = EngagementEdgeClient().get_jwt()
    delete_plugin = model_plugin_client.delete_model_plugin(
        jwt,
        plugin_to_delete,
    )
    logging.info(f"Deleting Plugin: {plugin_to_delete}")
    deleted = delete_plugin.json()["success"]["plugins_to_delete"]
    assert deleted
