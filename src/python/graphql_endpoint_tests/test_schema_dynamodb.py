"""
Schema generation happens at the provision stage, which doesn't really have an 
associated test suite yet. So, for the time being, just gonna shoehorn it
into graphql endpoint tests (which are *consumers* of the dynamodb node schemas)
"""
from typing import cast
from unittest import TestCase
import os

import pytest
import boto3
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.env_helpers import DynamoDBResourceFactory
from grapl_common.provision import SchemaDict

LOGGER = get_module_grapl_logger()

@pytest.mark.integration_test
class TestSchemaStoredInDynamodb(TestCase):
    def test_asset_stored_in_dynamodb(
        self,
    ) -> None:
        resource = DynamoDBResourceFactory(boto3).from_env()
        schema_props_table = resource.Table(f"{os.environ['DEPLOYMENT_NAME']}-grapl_schema_properties_table")
        asset = cast(SchemaDict, schema_props_table.get_item(Key={"node_type": "Asset"}))["Item"]["type_definition"]
        LOGGER.info(asset)
        assert asset["properties"][0]["name"] == "uid"
        # TODO: More robust testing of properties, in particular the edges!

