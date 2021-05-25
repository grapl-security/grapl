from typing import *

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.node_types import PropPrimitive, PropType
from grapl_analyzerlib.nodes.entity import EntityQuery, EntitySchema, EntityView
from grapl_analyzerlib.prelude import *


class IamRoleNodeSchema(EntitySchema):
    def __init__(self) -> None:
        super(IamRoleNodeSchema, self).__init__(
            properties={
                "arn": PropType(PropPrimitive.Str, False),
            },
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
    def node_schema(cls) -> IamRoleNodeSchema:
        return IamRoleNodeSchema()


class IamRoleView(EntityView[SelfV, SelfQ]):
    queryable = IamRoleQuery  # type: ignore[assignment]

    def __init__(  # type: ignore[no-untyped-def]
        self,
        graph_client: GraphClient,
        node_key: str,
        uid: int,
        node_types: Set[str],
        arn: Optional[str] = None,
        role_name: Optional[str] = None,
        **kwargs,
    ) -> None:
        super(IamRoleView, self).__init__(
            uid, node_key, graph_client, node_types, **kwargs
        )

        if arn:
            self.set_predicate("arn", arn)
        if role_name:
            self.set_predicate("role_name", role_name)

    @classmethod
    def node_schema(cls) -> IamRoleNodeSchema:
        return IamRoleNodeSchema()
