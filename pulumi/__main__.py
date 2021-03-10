import pulumi_aws as aws
from infra import dynamodb, emitter, ui, util
from infra.autotag import register_auto_tags
from infra.service_queue import ServiceQueue

import pulumi

PREFIX = pulumi.get_stack()

if __name__ == "__main__":

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": pulumi.get_stack()})

    dynamodb_tables = dynamodb.DynamoDB(PREFIX)

    ui = ui.UI(PREFIX)

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
