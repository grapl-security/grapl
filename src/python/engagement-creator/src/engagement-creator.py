import json
import logging
import os
import sys
import time
import traceback
from collections import defaultdict
from typing import *

import boto3
import botocore.exceptions
from grapl_analyzerlib.nodes.lens_node import LensView
from grapl_analyzerlib.prelude import NodeView
from pydgraph import DgraphClient, DgraphClientStub

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def parse_s3_event(s3, event) -> str:
    # Retrieve body of sns message
    # Decode json body of sns message
    LOGGER.debug("event is {}".format(event))
    # msg = json.loads(event["body"])["Message"]
    # msg = json.loads(msg)

    bucket = event["s3"]["bucket"]["name"]
    key = event["s3"]["object"]["key"]
    return download_s3_file(s3, bucket, key)


def download_s3_file(s3, bucket: str, key: str) -> str:
    key = key.replace("%3D", "=")
    LOGGER.info("Downloading s3 file from: {} {}".format(bucket, key))
    obj = s3.Object(bucket, key)
    return obj.get()["Body"].read()


def create_edge(
    client: DgraphClient, from_uid: str, edge_name: str, to_uid: str
) -> None:
    if edge_name[0] == "~":
        mut = {"uid": to_uid, edge_name[1:]: {"uid": from_uid}}

    else:
        mut = {"uid": from_uid, edge_name: {"uid": to_uid}}

    txn = client.txn(read_only=False)
    try:
        res = txn.mutate(set_obj=mut, commit_now=True)
        LOGGER.debug("edge mutation result is: {}".format(res))
    finally:
        txn.discard()


def attach_risk(
    client: DgraphClient,
    node_key: str,
    node_uid: str,
    analyzer_name: str,
    risk_score: int,
) -> None:

    risk_node = {
        "node_key": node_key + analyzer_name,
        "analyzer_name": analyzer_name,
        "risk_score": risk_score,
        "dgraph.type": "Risk",
    }

    risk_node_uid = upsert(client, risk_node)

    create_edge(client, node_uid, "risks", risk_node_uid)


def recalculate_score(client: DgraphClient, lens: LensView) -> int:
    query = """
        query q0($a: string)
        {
          q0(func: eq(lens, $a), first: 1) @cascade
          {
            uid,
            scope {
                node_key,
                risks {
                    analyzer_name,
                    risk_score
                }
            }
          }
        }
        """

    variables = {
        "$a": lens.lens,
    }
    txn = client.txn(read_only=False)
    risk_score = 0
    try:
        res = json.loads(txn.query(query, variables=variables).json)["q0"]

        redundant_risks = set()
        risk_map = defaultdict(list)
        if not res:
            LOGGER.warning("Received an empty response for risk query")
            return 0
        for root_node in res[0]["scope"]:
            for risk in root_node["risks"]:
                if risk["analyzer_name"] in redundant_risks:
                    continue
                redundant_risks.add(risk["analyzer_name"])
                risk_map[risk["analyzer_name"]].append(risk)

        for risks in risk_map.values():
            node_risk = 0
            for risk in risks:
                node_risk += risk["risk_score"]
            risk_multiplier = 0.10 * ((len(risks) or 1) - 1)
            node_risk = node_risk + (node_risk * risk_multiplier)
            risk_score += node_risk

        set_score(client, lens.uid, risk_score, txn=txn)
    finally:
        txn.discard()

    return risk_score


def set_score(client: DgraphClient, uid: str, new_score: int, txn=None) -> None:
    if not txn:
        txn = client.txn(read_only=False)

    try:
        mutation = {"uid": uid, "score": new_score}

        txn.mutate(set_obj=mutation, commit_now=True)
    finally:
        txn.discard()


def set_property(client: DgraphClient, uid: str, prop_name: str, prop_value):
    LOGGER.debug(f"Setting property {prop_name} as {prop_value} for {uid}")
    txn = client.txn(read_only=False)

    try:
        mutation = {"uid": uid, prop_name: prop_value}

        txn.mutate(set_obj=mutation, commit_now=True)
    finally:
        txn.discard()


def upsert(client: DgraphClient, node_dict: Dict[str, Any]) -> str:
    if node_dict.get("uid"):
        node_dict.pop("uid")
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    LOGGER.info(f"Upserting node: {node_dict}")
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,
                    dgraph.type,
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)["q0"]

        if res:
            node_dict["uid"] = res[0]["uid"]
            node_dict = {**node_dict, **res[0]}

        mutation = node_dict

        mut_res = txn.mutate(set_obj=mutation, commit_now=True)
        new_uid = node_dict.get("uid") or mut_res.uids["blank-0"]
        return new_uid

    finally:
        txn.discard()


def get_s3_client():
    if IS_LOCAL:
        return boto3.resource(
            "s3",
            endpoint_url="http://s3:9000",
            aws_access_key_id="minioadmin",
            aws_secret_access_key="minioadmin",
        )
    else:
        return boto3.resource("s3")


def lambda_handler(events: Any, context: Any) -> None:
    mg_alpha_names = os.environ["MG_ALPHAS"].split(",")
    mg_alpha_port = os.environ.get("MG_ALPHA_PORT", "9080")

    mg_client_stubs = [
        DgraphClientStub(f"{name}:{mg_alpha_port}") for name in mg_alpha_names
    ]
    mg_client = DgraphClient(*mg_client_stubs)

    s3 = get_s3_client()
    for event in events["Records"]:
        if not IS_LOCAL:
            event = json.loads(event["body"])["Records"][0]

        data = parse_s3_event(s3, event)
        incident_graph = json.loads(data)

        analyzer_name = incident_graph["analyzer_name"]
        nodes = incident_graph["nodes"]
        edges = incident_graph["edges"]
        risk_score = incident_graph["risk_score"]
        lens_dict = incident_graph["lenses"]

        LOGGER.debug(
            f"AnalyzerName {analyzer_name}, nodes: {nodes} edges: {type(edges)} {edges}"
        )

        nodes = [
            NodeView.from_node_key(mg_client, n["node_key"]) for n in nodes.values()
        ]

        uid_map = {node.node_key: node.uid for node in nodes}

        lenses = {}  # type: Dict[str, LensView]
        for node in nodes:
            LOGGER.debug(f"Copying node: {node}")

            for lens_type, lens_name in lens_dict:
                LOGGER.debug(f"Getting lens for: {lens_type} {lens_name}")
                lens_id = lens_name + lens_type
                lens: LensView = lenses.get(lens_name) or LensView.get_or_create(
                    mg_client, lens_name, lens_type
                )
                lenses[lens_id] = lens

                # Attach to scope
                create_edge(mg_client, lens.uid, "scope", node.uid)

                # If a node shows up in a lens all of its connected nodes should also show up in that lens
                for edge_list in edges.values():
                    for edge in edge_list:
                        from_uid = uid_map[edge["from"]]
                        to_uid = uid_map[edge["to"]]
                        create_edge(mg_client, lens.uid, "scope", from_uid)
                        create_edge(mg_client, lens.uid, "scope", to_uid)

        for node in nodes:
            attach_risk(
                mg_client, node.node.node_key, node.uid, analyzer_name, risk_score
            )

        for edge_list in edges.values():
            for edge in edge_list:
                from_uid = uid_map[edge["from"]]
                edge_name = edge["edge_name"]
                to_uid = uid_map[edge["to"]]

                create_edge(mg_client, from_uid, edge_name, to_uid)

        for lens in lenses.values():
            recalculate_score(mg_client, lens)


if IS_LOCAL:
    os.environ["MG_ALPHAS"] = "master_graph"

    sqs = boto3.client(
        "sqs",
        region_name="us-east-1",
        endpoint_url="http://sqs.us-east-1.amazonaws.com:9324",
        aws_access_key_id="dummy_cred_aws_access_key_id",
        aws_secret_access_key="dummy_cred_aws_secret_access_key",
    )

    alive = False
    while not alive:
        try:
            if "QueueUrls" not in sqs.list_queues(
                QueueNamePrefix="grapl-engagement-creator-queue"
            ):
                LOGGER.info("Waiting for grapl-engagement-creator-queue to be created")
                time.sleep(2)
                continue
        except (
            botocore.exceptions.BotoCoreError,
            botocore.exceptions.ClientError,
            botocore.parsers.ResponseParserError,
        ):
            LOGGER.info("Waiting for SQS to become available")
            time.sleep(2)
            continue
        alive = True

    while True:
        try:
            res = sqs.receive_message(
                QueueUrl="http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-engagement-creator-queue",
                WaitTimeSeconds=10,
                MaxNumberOfMessages=10,
            )

            messages = res.get("Messages", [])
            if not messages:
                LOGGER.warning("queue was empty")

            s3_events = [
                (json.loads(msg["Body"]), msg["ReceiptHandle"]) for msg in messages
            ]
            for s3_event, receipt_handle in s3_events:
                lambda_handler(s3_event, {})

                sqs.delete_message(
                    QueueUrl="http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-engagement-creator-queue",
                    ReceiptHandle=receipt_handle,
                )

        except Exception as e:
            LOGGER.error(f"mainloop exception {e}")
            LOGGER.error(traceback.print_exc())
            time.sleep(2)
