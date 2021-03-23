from infra import dynamodb, emitter, util
from infra.autotag import register_auto_tags
from infra.engagement_creator import EngagementCreator
from infra.metric_forwarder import MetricForwarder
from infra.service_queue import ServiceQueue
from infra.util import DEPLOYMENT_NAME, IS_LOCAL
from infra.ux import EngagementUX

import pulumi

if __name__ == "__main__":

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": DEPLOYMENT_NAME})

    dynamodb_tables = dynamodb.DynamoDB()

    ux = EngagementUX()

    util.grapl_bucket("model-plugins-bucket", sse=False)
    util.grapl_bucket("analyzers-bucket", sse=True)

    events = [
        "dispatched-analyzer",
        "osquery-log",
        "subgraphs-generated",
        "subgraphs-merged",
        "sysmon-log",
        "unid-subgraphs-generated",
    ]
    for event in events:
        emitter.EventEmitter(event)

    analyzer_matched = emitter.EventEmitter("analyzer-matched-subgraphs")

    # All services that haven't been ported over to the Service
    # abstraction yet. Services will create ServiceQueues
    services = (
        "analyzer-dispatcher",
        "analyzer-executor",
        "graph-merger",
        "node-identifier",
        "osquery-generator",
        "sysmon-generator",
    )
    for service in services:
        ServiceQueue(service)

    forwarder = MetricForwarder()

    ec = EngagementCreator(source_emitter=analyzer_matched, forwarder=forwarder)

    if IS_LOCAL:
        from infra.local import secret, user

        secret.jwt_secret()
        user.local_grapl_user(
            dynamodb_tables.user_auth_table, "grapluser", "graplpassword"
        )
