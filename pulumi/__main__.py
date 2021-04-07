from infra import dynamodb, emitter
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME, LOCAL_GRAPL
from infra.dgraph_ttl import DGraphTTL
from infra.engagement_creator import EngagementCreator
from infra.metric_forwarder import MetricForwarder
from infra.secret import JWTSecret
from infra.service_queue import ServiceQueue
from infra.ux import EngagementUX

if __name__ == "__main__":

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": DEPLOYMENT_NAME})

    dgraph_ttl = DGraphTTL()

    secret = JWTSecret()

    dynamodb_tables = dynamodb.DynamoDB()

    ux = EngagementUX()

    Bucket("model-plugins-bucket", sse=False)
    Bucket("analyzers-bucket", sse=True)

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

    if LOCAL_GRAPL:
        from infra.local import user

        user.local_grapl_user(
            dynamodb_tables.user_auth_table, "grapluser", "graplpassword"
        )
