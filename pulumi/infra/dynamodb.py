from typing import Dict, List, Optional

import pulumi_aws as aws
from infra.util import import_aware_opts

import pulumi


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

    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        super().__init__("grapl:DynamoDB", name, None, opts)

        self.schema_table = dynamodb_table(
            f"{name}-grapl_schema_table",
            [{"name": "f_edge", "type": "S"}],
            self,
            hash_key="f_edge",
        )
        self.static_mapping_table = dynamodb_table(
            f"{name}-static_mapping_table",
            [{"name": "pseudo_key", "type": "S"}],
            self,
            hash_key="pseudo_key",
        )
        self.user_auth_table = dynamodb_table(
            f"{name}-user_auth_table",
            [{"name": "username", "type": "S"}],
            self,
            hash_key="username",
        )

        self.asset_id_mappings = dynamodb_table(
            f"{name}-asset_id_mappings",
            [{"name": "pseudo_key", "type": "S"}, {"name": "c_timestamp", "type": "N"}],
            self,
            hash_key="pseudo_key",
            range_key="c_timestamp",
        )

        self.dynamic_session_table = dynamodb_history_table(
            f"{name}-dynamic_session_table", self
        )
        self.file_history_table = dynamodb_history_table(
            f"{name}-file_history_table", self
        )
        self.inbound_connection_history_table = dynamodb_history_table(
            f"{name}-inbound_connection_history_table", self
        )
        self.ip_connection_history_table = dynamodb_history_table(
            f"{name}-ip_connection_history_table", self
        )
        self.network_connection_history_table = dynamodb_history_table(
            f"{name}-network_connection_history_table", self
        )
        self.outbound_connection_hstiory_table = dynamodb_history_table(
            f"{name}-outbound_connection_history_table", self
        )
        self.process_history_table = dynamodb_history_table(
            f"{name}-process_history_table", self
        )

        self.register_outputs({})


# Below are essentially private functions


def dynamodb_table(
    name: str,
    attrs: List[Dict[str, str]],
    parent_resource: pulumi.Resource,
    hash_key: str,
    range_key: Optional[str] = None,
) -> aws.dynamodb.Table:
    """Defines a single DynamoDB table.

    Of particular note:
    - all tables have the "pay per request" billing mode
    """
    return aws.dynamodb.Table(
        name,
        name=name,
        attributes=[
            aws.dynamodb.TableAttributeArgs(name=a["name"], type=a["type"])
            for a in attrs
        ],
        hash_key=hash_key,
        range_key=range_key,
        billing_mode="PAY_PER_REQUEST",
        tags={"grapl deployment": pulumi.get_stack()},
        opts=import_aware_opts(name, parent=parent_resource),
    )


def dynamodb_history_table(
    name: str, parent_resource: pulumi.Resource
) -> aws.dynamodb.Table:
    """A specialization of `dynamodb_table` for our various "history"
    tracking tables, which all share the same indexing structures.

    """
    return dynamodb_table(
        name,
        [{"name": "pseudo_key", "type": "S"}, {"name": "create_time", "type": "N"}],
        parent_resource,
        hash_key="pseudo_key",
        range_key="create_time",
    )
