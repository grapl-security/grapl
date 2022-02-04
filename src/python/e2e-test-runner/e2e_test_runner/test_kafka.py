import copy
import os
import uuid
from typing import Any

import pytest
from confluent_kafka import Consumer, Producer  # type: ignore

IS_LOCAL = not os.environ["KAFKA_BOOTSTRAP_SERVERS"].startswith("SASL_SSL")
KAFKA_CLIENT_CONFIG = {
    "bootstrap.servers": os.environ["KAFKA_BOOTSTRAP_SERVERS"],
    "security.protocol": "PLAINTEXT" if IS_LOCAL else "SASL_SSL",
}

if not IS_LOCAL:
    # https://docs.confluent.io/cloud/current/client-apps/config-client.html#librdkafka-based-c-clients
    KAFKA_CLIENT_CONFIG["sasl.mechanisms"] = "PLAIN"
    KAFKA_CLIENT_CONFIG["sasl.username"] = os.environ["KAFKA_SASL_USERNAME"]
    KAFKA_CLIENT_CONFIG["sasl.password"] = os.environ["KAFKA_SASL_PASSWORD"]
    KAFKA_CLIENT_CONFIG["broker.address.ttl"] = "30000"
    KAFKA_CLIENT_CONFIG["api.version.request"] = "true"
    KAFKA_CLIENT_CONFIG["api.version.fallback.ms"] = "0"
    KAFKA_CLIENT_CONFIG["broker.version.fallback"] = "0.10.0.0"


@pytest.fixture
def kafka_producer() -> Producer:
    producer_config = copy.deepcopy(KAFKA_CLIENT_CONFIG)
    producer_config["acks"] = "all"

    producer = Producer(producer_config)

    yield producer


def _kafka_consumer(topic: str) -> Consumer:
    consumer_config = copy.deepcopy(KAFKA_CLIENT_CONFIG)
    consumer_config["group.id"] = os.environ["KAFKA_CONSUMER_GROUP_NAME"]
    consumer_config["enable.auto.commit"] = "true"
    consumer_config["auto.offset.reset"] = "earliest"
    consumer_config["session.timeout.ms"] = "45000"

    consumer = Consumer(consumer_config)
    consumer.subscribe([topic])

    return consumer


@pytest.fixture
def logs_consumer() -> Consumer:
    consumer = _kafka_consumer(topic="logs")
    yield consumer
    consumer.close()


@pytest.fixture
def metrics_consumer() -> Consumer:
    consumer = _kafka_consumer(topic="metrics")
    yield consumer
    consumer.close()


def _producer_callback(err: Any, msg: Any) -> None:
    assert err is None
    assert msg is not None
    assert msg.error() is None


def test_can_write_metrics(kafka_producer: Producer) -> None:
    for msg_id in (str(uuid.uuid4()) for _ in range(1000)):
        kafka_producer.produce(
            topic="metrics",
            key=f"e2e-test|{msg_id}",  # TODO: write valid metrics instead
            value=f"e2e-test|{msg_id}",
            on_delivery=_producer_callback,
        )

    kafka_producer.flush()


def test_can_read_metrics(metrics_consumer: Consumer) -> None:
    msgs = metrics_consumer.consume(num_messages=1000, timeout=10)
    assert len(msgs) == 1000

    for msg in msgs:
        assert msg is not None
        assert msg.error() is None


def test_can_write_logs(kafka_producer: Producer) -> None:
    for msg_id in (str(uuid.uuid4()) for _ in range(1000)):
        kafka_producer.produce(
            topic="logs",
            key=f"e2e-test|{msg_id}",
            value=f"e2e-test|{msg_id}",
            on_delivery=_producer_callback,
        )

    kafka_producer.flush()


def test_can_read_logs(logs_consumer: Consumer) -> None:
    msgs = logs_consumer.consume(num_messages=1000, timeout=10)
    assert len(msgs) == 1000

    for msg in msgs:
        assert msg is not None
        assert msg.error() is None
