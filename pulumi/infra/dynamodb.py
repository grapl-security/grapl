import json
from typing import Sequence

import pulumi_aws as aws
from infra.config import STACK_NAME

import pulumi


class DynamoDBTable(aws.dynamodb.Table):
    """Specialization of a regular DynamoDB table resource to ensure
    commonalities across all our tables, make things less verbose, and
    provide additional functionality.

    In particular, all tables have a `PAY_PER_REQUEST` billing mode,
    as well as an explicitly-set physical name.

    """

    def __init__(
        self,
        name: str,
        attrs: list[dict[str, str]],
        hash_key: str,
        range_key: str | None = None,
        opts: pulumi.ResourceOptions | None = None,
    ) -> None:

        super().__init__(
            name,
            attributes=[
                aws.dynamodb.TableAttributeArgs(name=a["name"], type=a["type"])
                for a in attrs
            ],
            hash_key=hash_key,
            range_key=range_key,
            billing_mode="PAY_PER_REQUEST",
            opts=opts,
        )


class DynamoDB(pulumi.ComponentResource):
    """Consolidates the creation of all our DynamoDB tables.

    Currently, this is mainly to group all the tables together under a
    single resource so we can conceptually deal with a single
    "thing". However, as we convert more infrastructure over to be
    managed by Pulumi, we may want to break these apart around
    functionality or logical domain, rather than storage method.

    Note that the name of this resource will be prepended to all
    created table names.
    """

    def __init__(self, opts: pulumi.ResourceOptions | None = None) -> None:
        super().__init__("grapl:DynamoDB", STACK_NAME, None, opts)

        self.schema_properties_table = DynamoDBTable(
            f"{STACK_NAME}-grapl_schema_properties_table",
            attrs=[
                {"name": "node_type", "type": "S"},
                # We dynamically create a "type_definition" M (map) type.
            ],
            hash_key="node_type",
            opts=pulumi.ResourceOptions(parent=self),
        )
        pulumi.export("schema-properties-table", self.schema_properties_table.name)

        self.schema_table = DynamoDBTable(
            f"{STACK_NAME}-grapl_schema_table",
            attrs=[{"name": "f_edge", "type": "S"}],
            hash_key="f_edge",
            opts=pulumi.ResourceOptions(parent=self),
        )
        pulumi.export("schema-table", self.schema_table.name)

        self.static_mapping_table = DynamoDBTable(
            f"{STACK_NAME}-static_mapping_table",
            attrs=[{"name": "pseudo_key", "type": "S"}],
            hash_key="pseudo_key",
            opts=pulumi.ResourceOptions(parent=self),
        )
        pulumi.export("static-mapping-table", self.static_mapping_table.name)

        self.user_auth_table = DynamoDBTable(
            f"{STACK_NAME}-user_auth_table",
            attrs=[{"name": "username", "type": "S"}],
            hash_key="username",
            opts=pulumi.ResourceOptions(parent=self),
        )
        pulumi.export("user-auth-table", self.user_auth_table.name)

        self.user_session_table = DynamoDBTable(
            f"{STACK_NAME}-user_session_table",
            attrs=[{"name": "session_token", "type": "S"}],
            hash_key="session_token",
            opts=pulumi.ResourceOptions(parent=self),
        )
        self.dynamic_session_table = DynamoDBTable(
            f"{STACK_NAME}-dynamic_session_table",
            attrs=[
                {"name": "pseudo_key", "type": "S"},
                {"name": "create_time", "type": "N"},
            ],
            hash_key="pseudo_key",
            range_key="create_time",
            opts=pulumi.ResourceOptions(parent=self),
        )
        pulumi.export("dynamic-session-table", self.dynamic_session_table.name)

        self.register_outputs({})


def grant_read_write_on_tables(
    role: aws.iam.Role, tables: Sequence[aws.dynamodb.Table]
) -> None:
    """Rather than granting permissions to each table individually, we
    grant to multiple tables at once in order to keep overall Role sizes
    down.
    """
    aws.iam.RolePolicy(
        f"{role._name}-reads-and-writes-dynamodb-tables",
        role=role.name,
        policy=pulumi.Output.all(*[t.arn for t in tables]).apply(
            lambda arns: json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": [
                                # Read
                                "dynamodb:BatchGetItem",
                                "dynamodb:GetRecords",
                                "dynamodb:GetShardIterator",
                                "dynamodb:Query",
                                "dynamodb:GetItem",
                                "dynamodb:Scan",
                                # Write
                                "dynamodb:BatchWriteItem",
                                "dynamodb:PutItem",
                                "dynamodb:UpdateItem",
                                "dynamodb:DeleteItem",
                            ],
                            "Resource": [a for a in arns],
                        }
                    ],
                }
            )
        ),
        opts=pulumi.ResourceOptions(parent=role),
    )


def grant_read_on_tables(
    role: aws.iam.Role, tables: Sequence[aws.dynamodb.Table]
) -> None:
    """Rather than granting permissions to each table individually, we
    grant to multiple tables at once in order to keep overall Role sizes
    down.
    """
    aws.iam.RolePolicy(
        f"{role._name}-reads-dynamodb-tables",
        role=role.name,
        policy=pulumi.Output.all(*[t.arn for t in tables]).apply(
            lambda arns: json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            "Action": [
                                "dynamodb:BatchGetItem",
                                "dynamodb:GetRecords",
                                "dynamodb:GetShardIterator",
                                "dynamodb:Query",
                                "dynamodb:GetItem",
                                "dynamodb:Scan",
                            ],
                            "Resource": [a for a in arns],
                        }
                    ],
                }
            )
        ),
        opts=pulumi.ResourceOptions(parent=role),
    )
