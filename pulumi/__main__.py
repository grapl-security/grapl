import pulumi_aws as aws
from infra import dynamodb
from infra.service_queue import ServiceQueue

import pulumi

PREFIX = pulumi.get_stack()

if __name__ == "__main__":

    dynamodb_tables = dynamodb.DynamoDB(PREFIX)

    buckets = (
        "analyzer-dispatched-bucket",
        "analyzer-matched-subgraphs-bucket",
        "analyzers-bucket",
        "engagement-ux-bucket",
        "model-plugins-bucket",
        "osquery-log-bucket",
        "subgraphs-generated-bucket",
        "subgraphs-merged-bucket",
        "sysmon-log-bucket",
        "unid-subgraphs-generated-bucket",
    )

    for bucket in buckets:
        bucket_name = f"{PREFIX}-{bucket}"
        bucket = aws.s3.Bucket(
            bucket,
            bucket=bucket_name,
        )
        pulumi.export(f"Bucket: {bucket_name}", bucket.id)

    services = (
        "analyzer-dispatcher",
        "analyzer-executor",
        "engagement-creator",
        "generic-graph-generator",
        "graph-merger",
        "node-identifier",
        "osquery-graph-generator",
        "sysmon-graph-generator",
    )

    for service in services:
        ServiceQueue(service)

    if pulumi.get_stack() == "local-grapl":
        from infra.local import secret, user

        secret.jwt_secret()
        user.local_grapl_user(
            dynamodb_tables.user_auth_table, "grapluser", "graplpassword"
        )
