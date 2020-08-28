#!/usr/bin/python3
import time
import boto3
import botocore

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
        "key_schema": [
            {"KeyType": "HASH", "AttributeName": "username"},
        ],
        "attribute_definitions": [
            {"AttributeName": "username", "AttributeType": "S"},
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
        except botocore.exceptions.ClientError as e:
            if "ResourceInUseException" in e.__class__.__name__:
                break
            else:
                print(table_name, e)
                time.sleep(2)
        except Exception as e:
            print(table_name, e)
            time.sleep(2)


for table_name in table_names:
    try:
        try_create_loop(table_name)
    except Exception as e:
        print(table_name, e)

print("Provisioned DynamoDB")
