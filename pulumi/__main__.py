from infra import dynamodb, emitter
from infra.api import Api
from infra.autotag import register_auto_tags
from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME, LOCAL_GRAPL
from infra.dgraph_ttl import DGraphTTL
from infra.engagement_creator import EngagementCreator
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.secret import JWTSecret
from infra.service_queue import ServiceQueue

if __name__ == "__main__":

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": DEPLOYMENT_NAME})

    network = Network("grapl-network")

    dgraph_ttl = DGraphTTL(network=network)

    secret = JWTSecret()

    dynamodb_tables = dynamodb.DynamoDB()

    model_plugins_bucket = Bucket("model-plugins-bucket", sse=False)
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

    forwarder = MetricForwarder(network=network)

    ec = EngagementCreator(
        source_emitter=analyzer_matched, network=network, forwarder=forwarder
    )

    ########################################################################

    # TODO: create everything inside of Api class

    import pulumi_aws as aws

    ux_bucket = Bucket(
        "engagement-ux-bucket",
        website_args=aws.s3.BucketWebsiteArgs(
            index_document="index.html",
        ),
    )
    # TODO: How do we get the *contents* of this bucket uploaded?

    api = Api(
        network=network,
        secret=secret,
        ux_bucket=ux_bucket,
        db=dynamodb_tables,
        plugins_bucket=model_plugins_bucket,
    )

    ########################################################################

    if LOCAL_GRAPL:
        from infra.local import user

        user.local_grapl_user(
            dynamodb_tables.user_auth_table, "grapluser", "graplpassword"
        )
