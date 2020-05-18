import json
import time
import boto3

table_names = [
    'process_history_table',
    'file_history_table',
    'node_id_retry_table',
    'outbound_connection_history_table',
    'inbound_connection_history_table',
    'network_connection_history_table',
    'ip_connection_history_table',
    'asset_id_mappings',
    'dynamic_session_table',
    'static_mapping_table',
]

tables = {}
for table_name in table_names:
    dynamodb = boto3.client('dynamodb', region_name='us-east-1')
    table_description = dynamodb.describe_table(TableName=table_name)

    tables[table_name] = {
        'attribute_definitions': table_description['Table']['AttributeDefinitions'],
        'key_schema': table_description['Table']['KeySchema'],
    }
    # print(table_description['Table']['AttributeDefinitions'])
    # print('')
    # print(table_description['Table']['KeySchema'])

print(json.dumps(tables, indent=2))
    # delete table

    # try:
    #     dynamodb.delete_table(TableName=table_name)
    # except Exception as e:
    #     print('failed to delete {}'.format(e))
    #     try:
    #         dynamodb.delete_table(TableName=table_name)
    #     except Exception as e:
    #         print('failed to delete {}'.format(e))
    #
    # time.sleep(18)
    #
    # try:
    #     dynamodb.create_table(
    #         TableName=table_name,
    #         AttributeDefinitions=table_description['Table']['AttributeDefinitions'],
    #
    #         BillingMode='PAY_PER_REQUEST',
    #
    #         KeySchema=table_description['Table']['KeySchema'],
    #
    #     )
    # except:
    #     time.sleep(22)
    #
    #     dynamodb.create_table(
    #         TableName=table_name,
    #         AttributeDefinitions=table_description['Table']['AttributeDefinitions'],
    #
    #         BillingMode='PAY_PER_REQUEST',
    #
    #         KeySchema=table_description['Table']['KeySchema'],
    #
    #     )
