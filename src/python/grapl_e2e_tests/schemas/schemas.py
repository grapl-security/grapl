from model_plugins.aws_cloudtrail.schemas import IamRoleNodeSchema
from typing import *
from grapl_analyzerlib.node_types import (
    PropType,
    PropPrimitive,
)
from grapl_analyzerlib.prelude import *
from grapl_analyzerlib.nodes import (
    EntitySchema,
    EntityQuery, 
    EntityView,
)
from grapl_analyzerlib.grapl_client import (
    GraphClient
)


class IamRoleNodeSchema(EntitySchema):
    def __init__(self):
        super(IamRoleNodeSchema, self).__init__(
            properties={"arn": PropType(PropPrimitive.Str, False), },
            edges={},
            view=lambda: IamRoleView,
        )

    @staticmethod
    def self_type() -> str:
        return "IamRole"


SelfQ = TypeVar("SelfQ", bound="IamRoleQuery")
SelfV = TypeVar("SelfV", bound="IamRoleView")


class IamRoleQuery(EntityQuery[SelfV, SelfQ]):
    def __init__(self) -> None:
        super(IamRoleQuery, self).__init__()

    @classmethod
    def node_schema(cls) -> "IamRoleNodeSchema":
        return IamRoleNodeSchema()


class IamRoleView(EntityView[SelfV, SelfQ]):
    queryable = IamRoleQuery

    def __init__(
        self,
        graph_client: GraphClient,
        node_key: str,
        uid: str,
        node_types: Set[str],
        arn: Optional[str] = None,
        role_name: Optional[str] = None,
        **kwargs,
    ):
        super(IamRoleView, self).__init__(
            uid, node_key, graph_client, node_types, **kwargs
        )

        self.set_predicate("arn", arn)
        self.set_predicate("role_name", role_name)

    @classmethod
    def node_schema(cls) -> "IamRoleNodeSchema":
        return IamRoleNodeSchema()