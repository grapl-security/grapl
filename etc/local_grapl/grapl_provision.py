import json
import threading
import time

from typing import *

import boto3
import pydgraph
from grapl_analyzerlib.schemas import *
from grapl_analyzerlib.schemas.asset_schema import AssetSchema
from grapl_analyzerlib.schemas.lens_node_schema import LensSchema
from grapl_analyzerlib.schemas.risk_node_schema import RiskSchema
from grapl_analyzerlib.schemas.schema_builder import ManyToMany
from pydgraph import DgraphClient, DgraphClientStub


def set_schema(client, schema, engagement=False) -> None:
    op = pydgraph.Operation(schema=schema)
    client.alter(op)


def drop_all(client) -> None:
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)


def format_schemas(schema_defs) -> str:
    schemas = "\n\n".join([schema.to_schema_str() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join([
        "  # Type Definitions",
        types,
        "\n  # Schema Definitions",
        schemas,
    ])


def get_type_dict(client, type_name) -> Dict[str, Any]:
    query = f"""
    schema(type: {type_name}) {{
      type
      index
    }}
    """

    txn = client.txn(read_only=True)

    try:
        res = json.loads(txn.query(query).json)
    finally:
        txn.discard()

    type_dict = {}

    for d in res['types'][0]['fields']:
        if d['name'][0] == "~":
            name = f"<{d['name']}>"
        else:
            name = d['name']
        type_dict[name] = d['type']

    return type_dict


def update_reverse_edges(client, schema):
    type_dicts = {}

    rev_edges = set()
    for edge in schema.forward_edges:
        edge_n = edge[0]
        edge_t = edge[1]._inner_type.self_type()
        if edge_t == 'Any':
            continue

        rev_edges.add(('<~' + edge_n + '>', edge_t))

        if not type_dicts.get(edge_t):
            type_dicts[edge_t] = get_type_dict(client, edge_t)

    if not rev_edges:
        return

    for (rev_edge_n, rev_edge_t) in rev_edges:
        type_dicts[rev_edge_t][rev_edge_n] = 'uid'

    type_strs = ""

    for t in type_dicts.items():
        type_name = t[0]
        type_d = t[1]

        predicates = []
        for predicate_name, predicate_type in type_d.items():
            predicates.append(f"\t{predicate_name}: {predicate_type}")

        predicates = "\n".join(predicates)
        type_str = f"""
type {type_name} {{
{predicates}
            
    }}
        """
        type_strs += "\n"
        type_strs += type_str

    op = pydgraph.Operation(schema=type_strs)
    client.alter(op)


def provision_mg(mclient) -> None:
    # drop_all(mclient)
    # drop_all(___local_dg_provision_client)

    schemas = (
        AssetSchema(),
        ProcessSchema(),
        FileSchema(),
        IpConnectionSchema(),
        IpAddressSchema(),
        IpPortSchema(),
        NetworkConnectionSchema(),
        ProcessInboundConnectionSchema(),
        ProcessOutboundConnectionSchema(),
    )

    mg_schema_str = format_schemas(schemas)
    set_schema(mclient, mg_schema_str)


def provision_eg(eclient) -> None:
    # drop_all(mclient)
    # drop_all(___local_dg_provision_client)

    schemas = [
        AssetSchema(),
        ProcessSchema(),
        FileSchema(),
        IpConnectionSchema(),
        IpAddressSchema(),
        IpPortSchema(),
        NetworkConnectionSchema(),
        ProcessInboundConnectionSchema(),
        ProcessOutboundConnectionSchema(),
    ]

    eg_schemas = [s.with_forward_edge('risks', ManyToMany(RiskSchema), 'risky_nodes') for s in schemas]

    eg_schemas.append(RiskSchema())
    eg_schemas.append(LensSchema())
    eg_schema_str = format_schemas(eg_schemas)
    set_schema(eclient, eg_schema_str)



BUCKET_PREFIX = 'local-grapl'

services = (
    'sysmon-graph-generator',
    'generic-graph-generator',
    'node-identifier',
    'graph-merger',
    'analyzer-dispatcher',
    'analyzer-executor',
    'engagement-creator',
    'aws-guardduty-graph-generator'
)

buckets = (
    BUCKET_PREFIX + '-sysmon-log-bucket',
    BUCKET_PREFIX + '-unid-subgraphs-generated-bucket',
    BUCKET_PREFIX + '-subgraphs-generated-bucket',
    BUCKET_PREFIX + '-subgraphs-merged-bucket',
    BUCKET_PREFIX + '-analyzer-dispatched-bucket',
    BUCKET_PREFIX + '-analyzers-bucket',
    BUCKET_PREFIX + '-analyzer-matched-subgraphs-bucket',
    BUCKET_PREFIX + '-model-plugins-bucket',
    BUCKET_PREFIX + '-aws-guardduty-log-bucket',
)


def provision_sqs(sqs, service_name: str) -> None:
    redrive_queue = sqs.create_queue(
        QueueName=service_name + '-retry-queue',
        Attributes={
            'MessageRetentionPeriod': '86400'
        }

    )

    redrive_url = redrive_queue['QueueUrl']
    redrive_arn = sqs.get_queue_attributes(
        QueueUrl=redrive_url,
        AttributeNames=['QueueArn']
    )['Attributes']['QueueArn']

    redrive_policy = {
        'deadLetterTargetArn': redrive_arn,
        'maxReceiveCount': '10',
    }

    queue = sqs.create_queue(
        QueueName=service_name + '-queue',
    )

    sqs.set_queue_attributes(
        QueueUrl=queue['QueueUrl'],
        Attributes={
            'RedrivePolicy': json.dumps(redrive_policy)
        }
    )
    print(queue['QueueUrl'])

    sqs.purge_queue(QueueUrl=queue['QueueUrl'])
    sqs.purge_queue(QueueUrl=redrive_queue['QueueUrl'])


def provision_bucket(s3, bucket_name: str) -> None:
    try:
        s3.create_bucket(Bucket=bucket_name)
    except Exception as e:
        print(e)
        pass
    print(bucket_name)


def bucket_provision_loop() -> None:
    s3_succ = {bucket for bucket in buckets}
    s3 = None
    for i in range(0, 150):
        try:
            s3 = s3 or boto3.client(
                's3',
                endpoint_url="http://s3:9000",
                aws_access_key_id='minioadmin',
                aws_secret_access_key='minioadmin',
            )
        except Exception as e:
            print('failed to connect to sqs or s3')

        for bucket in buckets:
            if bucket in s3_succ:
                try:
                    provision_bucket(s3, bucket)
                    s3_succ.discard(bucket)
                except Exception as e:
                    print(e)
                    time.sleep(1)

        if not s3_succ:
            return

    raise Exception("Failed to provision s3")


def sqs_provision_loop() -> None:
    sqs_succ = {service for service in services}
    sqs = None
    for i in range(0, 150):
        try:
            sqs = sqs or boto3.client(
                'sqs',
                region_name="us-east-1",
                endpoint_url="http://sqs.us-east-1.amazonaws.com:9324",
                aws_access_key_id='dummy_cred_aws_access_key_id',
                aws_secret_access_key='dummy_cred_aws_secret_access_key',
            )
        except Exception as e:
            print('failed to connect to sqs or s3', e)
            time.sleep(1)
            continue

        for service in services:
            if service in sqs_succ:
                try:
                    provision_sqs(sqs, service)
                    sqs_succ.discard(service)
                except Exception as e:
                    print(e)
                    time.sleep(1)
        if not sqs_succ:
            return

    raise Exception("Failed to provision sqs")


def drop_all(client) -> None:
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)


if __name__ == '__main__':
    time.sleep(10)
    local_dg_provision_client = DgraphClient(DgraphClientStub('master_graph:9080'))
    local_eg_provision_client = DgraphClient(DgraphClientStub('engagement_graph:9080'))

    # local_dg_provision_client = DgraphClient(DgraphClientStub('localhost:9080'))
    # local_eg_provision_client = DgraphClient(DgraphClientStub('localhost:9081'))

    for i in range(0, 150):
        try:
            drop_all(local_dg_provision_client)
            drop_all(local_eg_provision_client)
            break
        except Exception as e:
            time.sleep(2)
            print('Failed to drop', e)

    mg_succ = False
    eg_succ = False

    sqs_t = threading.Thread(target=sqs_provision_loop)
    s3_t = threading.Thread(target=bucket_provision_loop)

    sqs_t.start()
    s3_t.start()

    for i in range(0, 150):
        try:
            if not mg_succ:
                print('attempting mg_succ')
                time.sleep(1)
                provision_mg(
                    local_dg_provision_client,
                )
                mg_succ = True
                break
        except Exception as e:
            print('mg provision failed with: ', e)

    for i in range(0, 150):
        try:
            if not eg_succ:
                print('attempting eg_succ')
                time.sleep(1)
                provision_eg(
                    local_eg_provision_client,
                )
                eg_succ = True
                break
        except Exception as e:
            print('eg provision failed with: ', e)


    sqs_t.join(timeout=300)
    s3_t.join(timeout=300)
