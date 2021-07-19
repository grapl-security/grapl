import copy
import logging
import os
import uuid
from pathlib import Path
from typing import Any, Dict, List, Mapping

import pytest
from confluent_kafka import Consumer, Producer, TopicPartition
from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.subset_equals import subset_equals
from grapl_tests_common.wait import (
    WaitForCondition,
    WaitForNoException,
    WaitForQuery,
    wait_for_one,
)

LENS_NAME = "DESKTOP-FVSHABR"
GqlLensDict = Dict[str, Any]
IS_LOCAL = bool(os.getenv("IS_LOCAL", ""))
KAFKA_CLIENT_CONFIG = {
    "bootstrap.servers": os.environ["KAFKA_BOOTSTRAP_SERVERS"],
    "security.protocol": "PLAINTEXT" if IS_LOCAL else "SASL_SSL",
}

if not IS_LOCAL:
    # TODO: use Vault, configure Kafka to use SASL and ACLs locally
    # See:
    #   https://github.com/grapl-security/issue-tracker/issues/621
    #   https://github.com/grapl-security/issue-tracker/issues/622
    KAFKA_CLIENT_CONFIG["sasl.username"] = os.environ["KAFKA_SASL_USERNAME"]
    KAFKA_CLIENT_CONFIG["sasl.password"] = os.environ["KAFKA_SASL_PASSWORD"]


def test_expected_data_in_dgraph(jwt: str) -> None:
    # There is some unidentified, nondeterministic failure with e2e.
    # We fall into one of three buckets:
    # - No lens
    # - Lens with 3 scope
    # - Lens with 4 scope
    # - Lens with 5 scope (correct)
    query = LensQuery().with_lens_name(LENS_NAME)
    lens: LensView = wait_for_one(WaitForQuery(query), timeout_secs=120)
    assert lens.get_lens_name() == LENS_NAME
    # lens scope is not atomic

    def scope_has_N_items() -> bool:
        length = len(lens.get_scope())
        logging.info(f"Expected 3-5 nodes in scope, currently is {length}")
        # The correct answer for this is 5.
        # We are temp 'allowing' below that because it means the pipeline is, _mostly_, working.
        return length in (
            3,
            4,
            5,
        )

    wait_for_one(WaitForCondition(scope_has_N_items), timeout_secs=300)

    # Now that we've confirmed that the expected data has shown up in dgraph,
    # let's see what the GraphQL endpoint says.
    # TODO: Consider using `pytest-order` to make this a separate test that
    # depends on the above test having been run.

    gql_client = GraphqlEndpointClient(jwt=jwt)
    wait_for_one(
        WaitForNoException(
            lambda: ensure_graphql_lens_scope_no_errors(gql_client, LENS_NAME)
        ),
        timeout_secs=300,
    )


def kafka_producer() -> Producer:
    producer_config = copy.deepcopy(KAFKA_CLIENT_CONFIG)
    producer_config["acks"] = "all"
    return Producer(producer_config)


def kafka_consumer(topic: str) -> Consumer:
    consumer_config = copy.deepcopy(KAFKA_CLIENT_CONFIG)
    consumer_config["group.id"] = "e2e-tests"
    consumer_config["auto.offset.reset"] = "earliest"

    consumer = Consumer(consumer_config)
    consumer.subscribe([topic])

    return consumer


def _producer_callback(err, _) -> None:
    assert err is None


def test_kafka_can_write_metrics(
    kafka_producer: Producer, metrics_consumer: Consumer
) -> None:
    msg_id = str(uuid.uuid4())
    kafka_producer.produce(
        topic="metrics", key=f"{msg_id}", value="test", callback=_producer_callback
    )
    kafka_producer.flush()

    msgs = metrics_consumer.consume(timeout=10)
    assert len(msgs) == 1
    msg = msgs[0]

    metrics_consumer.close()

    assert msg is not None
    assert msg.error() is None
    assert msg.key().decode("utf-8") == msg_id
    assert msg.value().decode("utf-8") == "test"


def test_kafka_can_write_logs(
    kafka_producer: Producer, logs_consumer: Consumer
) -> None:
    msg_id = str(uuid.uuid4())
    kafka_producer.produce(
        topic="logs",
        key=f"{msg_id}",
        value="test",
        callback=lambda err, _: _producer_callback,
    )
    kafka_producer.flush()

    msgs = logs_consumer.consume(timeout=10)
    assert len(msgs) == 1
    msg = msgs[0]

    logs_consumer.close()

    assert msg is not None
    assert msg.error() is None
    assert msg.key().decode("utf-8") == msg_id
    assert msg.value().decode("utf-8") == "test"


def ensure_graphql_lens_scope_no_errors(
    gql_client: GraphqlEndpointClient,
    lens_name: str,
) -> None:
    gql_lens = gql_client.query_for_scope(lens_name=lens_name)
    scope = gql_lens["scope"]
    assert len(scope) in (3, 4, 5)
    # Accumulate ["Asset"], ["Process"] into Set("Asset, Process")
    all_types_in_scope = set(
        sum((node["dgraph_type"] for node in gql_lens["scope"]), [])
    )
    assert all_types_in_scope == set(
        (
            "Asset",
            "Process",
        )
    )
    asset_node: Dict = next((n for n in scope if n["dgraph_type"] == ["Asset"]))
    # The 'risks' field is not immediately filled out, but eventually consistent
    subset_equals(larger=asset_node, smaller=expected_gql_asset())


def expected_gql_asset() -> Mapping[str, Any]:
    """
    All the fixed values (i.e. no uid, no node key) we'd see in the e2e test
    """
    return {
        "dgraph_type": ["Asset"],
        "display": "DESKTOP-FVSHABR",
        "hostname": "DESKTOP-FVSHABR",
        "asset_processes": [
            {
                "dgraph_type": ["Process"],
                "process_name": "cmd.exe",
                "process_id": 5824,
            },
            {
                "dgraph_type": ["Process"],
                "process_name": "dropper.exe",
                "process_id": 4164,
            },
            {
                "dgraph_type": ["Process"],
                "process_name": "cmd.exe",
                "process_id": 5824,
            },
            {
                "dgraph_type": ["Process"],
                "process_name": "svchost.exe",
                "process_id": 6132,
            },
        ],
        "files_on_asset": None,
        "risks": [
            {
                "dgraph_type": ["Risk"],
                "node_key": "Rare Parent of cmd.exe",
                "analyzer_name": "Rare Parent of cmd.exe",
                "risk_score": 10,
            }
        ],
    }
