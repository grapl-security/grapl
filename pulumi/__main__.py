import pulumi_aws as aws
from infra import dynamodb, emitter, ui
from infra.service_queue import ServiceQueue

import pulumi

PREFIX = pulumi.get_stack()

if __name__ == "__main__":

    dynamodb_tables = dynamodb.DynamoDB(PREFIX)

    ui = ui.UI(PREFIX)

    buckets = (
        "analyzer-dispatched-bucket",
        "analyzers-bucket",
        "model-plugins-bucket",
    )

    for logical_bucket_name in buckets:
        physical_bucket_name = f"{PREFIX}-{logical_bucket_name}"
        bucket = aws.s3.Bucket(
            logical_bucket_name,
            bucket=physical_bucket_name,
        )
        pulumi.export(f"Bucket: {physical_bucket_name}", bucket.id)


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
