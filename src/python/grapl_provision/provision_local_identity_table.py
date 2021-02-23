#!/usr/bin/python3
import logging
import os
import time

import boto3
import botocore
from grapl_common.env_helpers import DynamoDBClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger(default_log_level="INFO")

table_names = [
    "local-grapl-process_history_table",
    "local-grapl-file_history_table",
    "local-grapl-outbound_connection_history_table",
    "local-grapl-inbound_connection_history_table",
    "local-grapl-network_connection_history_table",
    "local-grapl-ip_connection_history_table",
    "local-grapl-asset_id_mappings",
    "local-grapl-dynamic_session_table",
    "local-grapl-static_mapping_table",
    "local-grapl-user_auth_table",
    "local-grapl-grapl_schema_table",
]

table_defs = {
    "local-grapl-network_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-inbound_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-static_mapping_table": {
        "key_schema": [{"KeyType": "HASH", "AttributeName": "pseudo_key"}],
        "attribute_definitions": [
            {"AttributeName": "pseudo_key", "AttributeType": "S"}
        ],
    },
    "local-grapl-dynamic_session_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-process_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-file_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-outbound_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-asset_id_mappings": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "c_timestamp"},
        ],
        "attribute_definitions": [
            {"AttributeName": "c_timestamp", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-ip_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "local-grapl-user_auth_table": {
        "key_schema": [{"KeyType": "HASH", "AttributeName": "username"},],
        "attribute_definitions": [{"AttributeName": "username", "AttributeType": "S"},],
    },
    "local-grapl-grapl_schema_table": {
        "key_schema": [{"KeyType": "HASH", "AttributeName": "f_edge"},],
        "attribute_definitions": [{"AttributeName": "f_edge", "AttributeType": "S"},],
    },
}

dynamodb = DynamoDBClientFactory(boto3).from_env()


def try_create_loop(table_name):
    for i in range(0, 10):
        try:
            d = dynamodb.create_table(
                TableName=table_name,
                BillingMode="PAY_PER_REQUEST",
                AttributeDefinitions=table_defs[table_name]["attribute_definitions"],
                KeySchema=table_defs[table_name]["key_schema"],
            )

            return
        except botocore.exceptions.ClientError as e:
            if "ResourceInUseException" in e.__class__.__name__:
                break
            else:
                if i >= 5:
                    LOGGER.warn(f"failed to provision dynamodb table: {table_name} {e}")
                else:
                    LOGGER.debug(
                        f"failed to provision dynamodb table: {table_name} {e}"
                    )
                time.sleep(2)
        except Exception as e:
            if i >= 5:
                LOGGER.warn(f"failed to provision dynamodb table: {table_name} {e}")
            else:
                LOGGER.debug(f"failed to provision dynamodb table: {table_name} {e}")
            time.sleep(2)


for table_name in table_names:
    try:
        try_create_loop(table_name)
    except Exception as e:
        LOGGER.error(table_name, e)
        raise e

LOGGER.info("Provisioned DynamoDB")
