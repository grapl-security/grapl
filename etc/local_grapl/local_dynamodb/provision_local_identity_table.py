#!/usr/bin/python3
import time
import boto3

table_names = [
    "process_history_table",
    "file_history_table",
    "node_id_retry_table",
    "outbound_connection_history_table",
    "inbound_connection_history_table",
    "network_connection_history_table",
    "ip_connection_history_table",
    "asset_id_mappings",
    "dynamic_session_table",
    "static_mapping_table",
]

table_defs = {
    "network_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "inbound_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "node_id_retry_table": {
        "key_schema": [{"KeyType": "HASH", "AttributeName": "pseudo_key"}],
        "attribute_definitions": [
            {"AttributeName": "pseudo_key", "AttributeType": "S"}
        ],
    },
    "static_mapping_table": {
        "key_schema": [{"KeyType": "HASH", "AttributeName": "pseudo_key"}],
        "attribute_definitions": [
            {"AttributeName": "pseudo_key", "AttributeType": "S"}
        ],
    },
    "dynamic_session_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "process_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "file_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "outbound_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "asset_id_mappings": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "c_timestamp"},
        ],
        "attribute_definitions": [
            {"AttributeName": "c_timestamp", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
    "ip_connection_history_table": {
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "pseudo_key"},
            {"KeyType": "RANGE", "AttributeName": "create_time"},
        ],
        "attribute_definitions": [
            {"AttributeName": "create_time", "AttributeType": "N"},
            {"AttributeName": "pseudo_key", "AttributeType": "S"},
        ],
    },
}

dynamodb = boto3.client(
    "dynamodb",
    region_name="us-west-2",
    endpoint_url="http://dynamodb:8000",
    aws_access_key_id="dummy_cred_aws_access_key_id",
    aws_secret_access_key="dummy_cred_aws_secret_access_key",
)


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
        except Exception as e:
            print(table_name, e)
            time.sleep(2)


for table_name in table_names:
    try:
        try_create_loop(table_name)
    except Exception as e:
        print(table_name, e)
