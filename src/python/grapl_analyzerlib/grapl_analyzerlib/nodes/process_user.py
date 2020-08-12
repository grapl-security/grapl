from collections import defaultdict
from typing import Any, TypeVar, List, Set, Type, Dict, Tuple, Optional, Iterator, Union

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.queryable import Queryable, EdgeFilter, ToOneFilter, ToManyFilter
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable, V, Q
from grapl_analyzerlib.comparators import StrCmp, Eq, Distance

from grapl_analyzerlib.nodes.process import (
    ProcessQuery,
    ProcessView,
    ProcessSchema,
    PQ,
    PV,
)


class ProcessWithUserQuery(ProcessQuery):
    def __init__(self):
        super(ProcessWithUserQuery, self).__init__()
        self._user: List[List[StrCmp]] = []
        ProcessQuery.viewable = ProcessQuery.viewable.extend_self(ProcessWithUserView)

    def with_user(
        self: PQ,
        eq: Optional[str] = None,
        contains: Optional[Union[str, List[str]]] = None,
        starts_with: Optional[str] = None,
        ends_with: Optional[str] = None,
        regexp: Optional[Union[str, List[str]]] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> PQ:
        self._property_filters["user"].extend(
            _str_cmps(
                predicate="user",
                eq=eq,
                contains=contains,
                ends_with=ends_with,
                starts_with=starts_with,
                regexp=regexp,
                distance_lt=distance_lt,
            )
        )
        return self


class ProcessWithUserView(ProcessView):
    user = None

    def __init__(
        self,
        uid: str,
        node_key: str,
        graph_client: Any,
        node_types: List[str],
        user: Optional[str] = None,
        **kwargs,
    ):
        super(ProcessWithUserView, self).__init__(
            uid=uid,
            node_key=node_key,
            graph_client=graph_client,
            node_types=node_types,
            **kwargs,
        )
        self.user = user
        ProcessView.queryable = ProcessView.queryable.extend_self(ProcessWithUserQuery)

    def get_user(self, cached=True) -> Optional[str]:
        if self.user and cached:
            return self.user

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_user()
            .query_first(self.graph_client)
        )
        if self_node:
            self.user = self_node.user
        return self.user


from grapl_analyzerlib.comparators import Cmp, _str_cmps

ProcessSchema().properties["user"] = PropType(PropPrimitive.Str, False)
ProcessQuery = ProcessQuery.extend_self(ProcessWithUserQuery)
ProcessView = ProcessView.extend_self(ProcessWithUserView)
