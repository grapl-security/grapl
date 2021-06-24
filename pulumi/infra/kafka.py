from typing import Optional

import pulumi_kafka as kafka

import pulumi


class Kafka(pulumi.ComponentResource):
    def __init__(self, name: str, opts: Optional[pulumi.ResourceOptions] = None):
        super().__init__("grapl:Kafka", name=name, props=None, opts=opts)
        metrics_topic = kafka.Topic(
            "metrics-topic", name="metrics", replication_factor=1, partitions=1
        )
