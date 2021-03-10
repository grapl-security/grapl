import pulumi_aws as aws
from infra import dynamodb, emitter, util
from infra.autotag import register_auto_tags
from infra.service_queue import ServiceQueue
from infra.util import DEPLOYMENT_NAME
from infra.ux import EngagementUX

import pulumi

if __name__ == "__main__":

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": DEPLOYMENT_NAME})

    dynamodb_tables = dynamodb.DynamoDB(DEPLOYMENT_NAME)

    ux = EngagementUX(DEPLOYMENT_NAME)

    util.grapl_bucket("model-plugins-bucket", sse=False)
    util.grapl_bucket("analyzers-bucket", sse=True)

    events = (
        "analyzer-matched-subgraphs",
        "dispatched-analyzer",
        "osquery-log",
        "subgraphs-generated",
        "subgraphs-merged",
        "sysmon-log",
        "unid-subgraphs-generated",
    )
    for event in events:
        emitter.EventEmitter(event)

    services = (
        "analyzer-dispatcher",
        "analyzer-executor",
        "engagement-creator",
        "graph-merger",
        "node-identifier",
        "osquery-generator",
        "sysmon-generator",
    )

    for service in services:
        ServiceQueue(service)

    if pulumi.get_stack() == "local-grapl":
        from infra.local import secret, user

        secret.jwt_secret()
        user.local_grapl_user(
            dynamodb_tables.user_auth_table, "grapluser", "graplpassword"
        )
