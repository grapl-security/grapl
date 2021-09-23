import json
import logging
import os
import sys
from collections import defaultdict
from typing import (
    Any,
    ContextManager,
    Dict,
    Optional,
    Sequence,
    Tuple,
    Type,
    TypeVar,
    cast,
)

import boto3
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.lens import LensView
from grapl_analyzerlib.prelude import BaseView, RiskView
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.viewable import Viewable
from grapl_common.env_helpers import S3ResourceFactory
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair
from grapl_common.sqs.sqs_types import SQSMessageBody
from mypy_boto3_s3 import S3ServiceResource
from typing_extensions import Final, Literal

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

"""
https://docs.aws.amazon.com/AmazonS3/latest/dev/notification-content-structure.html
"""
EventWithReceiptHandle = Tuple[SQSMessageBody, str]

V = TypeVar("V", bound=Viewable)
Q = TypeVar("Q", bound=Queryable)

SERVICE_NAME: Final[str] = "engagement-creator"


class EngagementCreatorMetrics:
    def __init__(self, service_name: str) -> None:
        self.metric_reporter = MetricReporter.create(service_name)

    def event_processed(self, status: Literal["success", "failure"]) -> None:
        self.metric_reporter.counter(
            metric_name="event_processed", value=1, tags=[TagPair("status", status)]
        )

    def time_to_process_event(self) -> ContextManager:
        return self.metric_reporter.histogram_ctx(metric_name="time_to_process_event")

    def risk_node(self, analyzer_name: str) -> None:
        # A generic "hey, there's a new risky node" metric that we can globally alarm on.
        # Has no dimensions. (See the top of `alarms.ts` to learn why!)
        self.metric_reporter.counter(
            metric_name=f"risk_node",
            value=1,
        )
        # A more-specific, per-analyzer metric, in case you wanted to define your own alarms
        # about just "suspicious svc host", for example.
        self.metric_reporter.counter(
            metric_name=f"risk_node_for_analyzer",
            value=1,
            tags=[
                TagPair("analyzer_name", analyzer_name),
            ],
        )


def parse_s3_event(s3: S3ServiceResource, event: Any) -> bytes:
    # Retrieve body of sns message
    # Decode json body of sns message
    LOGGER.debug("event is {}".format(event))

    bucket = event["s3"]["bucket"]["name"]
    key = event["s3"]["object"]["key"]
    return download_s3_file(s3, bucket, key)


def download_s3_file(s3: S3ServiceResource, bucket: str, key: str) -> bytes:
    key = key.replace("%3D", "=")
    LOGGER.info("Downloading s3 file from: {} {}".format(bucket, key))
    obj = s3.Object(bucket, key)
    return cast(bytes, obj.get()["Body"].read())


def create_edge(
    client: GraphClient, from_uid: int, edge_name: str, to_uid: int
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


def recalculate_score(lens: LensView) -> int:
    scope = lens.get_scope()
    key_to_analyzers = defaultdict(set)
    node_risk_scores = defaultdict(int)
    total_risk_score = 0
    for node in scope:
        node_risks = node.get_risks()
        risks_by_analyzer = {}
        for risk in node_risks:
            risk_score = risk.get_risk_score()
            analyzer_name = risk.get_analyzer_name()
            risks_by_analyzer[analyzer_name] = risk_score
            key_to_analyzers[node.node_key].add(analyzer_name)

        analyzer_risk_sum = sum([a for a in risks_by_analyzer.values() if a])
        node_risk_scores[node.node_key] = analyzer_risk_sum
        total_risk_score += analyzer_risk_sum

    # Bonus is calculated by finding nodes with multiple analyzers
    for key, analyzers in key_to_analyzers.items():
        if len(analyzers) <= 1:
            continue
        overlapping_analyzer_count = len(analyzers)
        bonus = node_risk_scores[key] * 2 * (overlapping_analyzer_count // 100)
        total_risk_score += bonus
    return total_risk_score


def _upsert(client: GraphClient, node_dict: Dict[str, Any]) -> str:
    node_dict["uid"] = "_:blank-0"
    node_key = node_dict["node_key"]
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}"), first: 1) {{
                    uid,
                    dgraph.type
                    expand(_all_)
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)["q0"]
        new_uid = None
        if res:
            node_dict["uid"] = res[0]["uid"]
            new_uid = res[0]["uid"]

        mutation = node_dict

        mut_res = txn.mutate(set_obj=mutation, commit_now=True)
        new_uid = node_dict.get("uid") or mut_res.uids["blank-0"]
        return cast(str, new_uid)
    finally:
        txn.discard()


def upsert(
    client: GraphClient,
    type_name: str,
    view_type: Type[Viewable[V, Q]],
    node_key: str,
    node_props: Dict[str, Any],
) -> Viewable[V, Q]:
    node_props["node_key"] = node_key
    node_props["dgraph.type"] = list({type_name, "Base", "Entity"})
    uid = _upsert(client, node_props)
    node_props["uid"] = uid
    return view_type.from_dict(node_props, client)


def nodes_to_attach_risk_to(
    nodes: Sequence[BaseView],
    risky_node_keys: Optional[Sequence[str]],
) -> Sequence[BaseView]:
    """
    a None risky_node_keys means 'mark all as risky'
    a [] risky_node_keys means 'mark none as risky'.
    """
    if risky_node_keys is None:
        return nodes
    risky_node_keys_set = frozenset(risky_node_keys)
    return [node for node in nodes if node.node_key in risky_node_keys_set]


def create_metrics_client() -> EngagementCreatorMetrics:
    return EngagementCreatorMetrics(SERVICE_NAME)


async def lambda_handler(s3_event: SQSMessageBody, context: Any) -> None:
    graph_client = GraphClient()
    s3 = S3ResourceFactory(boto3).from_env()
    metrics = create_metrics_client()

    for event in s3_event["Records"]:
        with metrics.time_to_process_event():
            try:
                _process_one_event(event, s3, graph_client, metrics)
            except:
                metrics.event_processed(status="failure")
                raise
            else:
                metrics.event_processed(status="success")


def _process_one_event(
    event: Any,
    s3: S3ServiceResource,
    mg_client: GraphClient,
    metrics: EngagementCreatorMetrics,
) -> None:
    event = json.loads(event["body"])["Records"][0]

    data = parse_s3_event(s3, event)
    incident_graph = json.loads(data)

    """
    The `incident_graph` dict is emitted from analyzer-executor.py#emit_event
    """
    analyzer_name = incident_graph["analyzer_name"]
    nodes_raw: Dict[str, Any] = incident_graph[
        "nodes"
    ]  # same type as `.to_adjacency_list()["nodes"]`
    edges = incident_graph["edges"]
    risk_score = incident_graph["risk_score"]
    lens_dict: Sequence[Tuple[str, str]] = incident_graph["lenses"]
    risky_node_keys = incident_graph["risky_node_keys"]

    LOGGER.debug(
        f"AnalyzerName {analyzer_name}, nodes: {nodes_raw} edges: {type(edges)} {edges}"
    )

    _nodes = (
        BaseView.from_node_key(mg_client, n["node_key"]) for n in nodes_raw.values()
    )
    nodes = [n for n in _nodes if n]

    uid_map = {node.node_key: node.uid for node in nodes}

    lenses = {}  # type: Dict[str, LensView]
    for node in nodes:
        LOGGER.debug(f"Copying node: {node}")

        for lens_type, lens_name in lens_dict:
            # i.e. "hostname", "DESKTOP-WHATEVER"
            LOGGER.debug(f"Getting lens for: {lens_type} {lens_name}")
            lens_id = lens_name + lens_type
            lens: LensView = lenses.get(lens_id) or LensView.get_or_create(
                mg_client, lens_name, lens_type
            )
            lenses[lens_id] = lens

            # Attach to scope
            create_edge(mg_client, lens.uid, "scope", node.uid)
            create_edge(mg_client, node.uid, "in_scope", lens.uid)

            # If a node shows up in a lens all of its connected nodes should also show up in that lens
            for edge_list in edges.values():
                for edge in edge_list:
                    from_uid = uid_map[edge["from"]]
                    to_uid = uid_map[edge["to"]]
                    create_edge(mg_client, lens.uid, "scope", from_uid)
                    create_edge(mg_client, lens.uid, "scope", to_uid)

                    create_edge(mg_client, from_uid, "in_scope", lens.uid)
                    create_edge(mg_client, to_uid, "in_scope", lens.uid)

    risk = upsert(
        mg_client,
        "Risk",
        RiskView,
        analyzer_name,
        {
            "analyzer_name": analyzer_name,
            "risk_score": risk_score,
        },
    )

    risky_nodes = nodes_to_attach_risk_to(nodes, risky_node_keys)
    for node in risky_nodes:
        create_edge(mg_client, node.uid, "risks", risk.uid)
        create_edge(mg_client, risk.uid, "risky_nodes", node.uid)

        # Or perhaps we should just emit per-risk instead of per-risky-node?
        # (this alarming path is definitely a candidate for changing later)
        metrics.risk_node(analyzer_name=analyzer_name)

    for edge_list in edges.values():
        for edge in edge_list:
            from_uid = uid_map[edge["from"]]
            edge_name = edge["edge_name"]
            to_uid = uid_map[edge["to"]]

            create_edge(mg_client, from_uid, edge_name, to_uid)

    for lens in lenses.values():
        lens_score = recalculate_score(lens)
        upsert(
            mg_client,
            "Lens",
            LensView,
            lens.node_key,
            {
                "score": lens_score,
            },
        )
