"""
Schema generation happens at the provision stage, which doesn't really have an 
associated test suite yet. So, for the time being, just gonna shoehorn it
into graphql endpoint tests (which are *consumers* of the dynamodb node schemas)
"""
import os
from typing import cast
from unittest import TestCase

import boto3
import pytest
from grapl_analyzerlib.node_types import PropPrimitive
from grapl_analyzerlib.provision import provision_common
from grapl_common.env_helpers import DynamoDBResourceFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.resources import known_dynamodb_tables

LOGGER = get_module_grapl_logger()


@pytest.mark.integration_test
class TestSchemaStoredInDynamodb(TestCase):
    def test_asset_stored_in_dynamodb(
        self,
    ) -> None:
        resource = DynamoDBResourceFactory(boto3).from_env()
        schema_props_table = known_dynamodb_tables.schema_properties_table(
            dynamodb=resource
        )

        asset_type = cast(
            provision_common.SchemaDict,
            schema_props_table.get_item(Key={"node_type": "Asset"})["Item"][
                "type_definition"
            ],
        )
        prop_map = {prop["name"]: prop for prop in asset_type["properties"]}
        assert (
            prop_map["uid"]["primitive"] == PropPrimitive.Int.name
        )  # special case for uid
        assert prop_map["uid"]["is_set"] == False
        assert prop_map["hostname"]["primitive"] == PropPrimitive.Str.name
        assert prop_map["hostname"]["is_set"] == False

        # how about forward edges?
        assert "asset_ip" in prop_map
        assert prop_map["asset_ip"]["primitive"] == "IpAddress"
        assert prop_map["asset_ip"]["is_set"] == True
