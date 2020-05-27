import enum
from typing import Optional, Any

from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.queryable import Queryable, generate_query
from grapl_analyzerlib.prelude import ProcessQuery


class OrderedEnum(enum.Enum):
    def __ge__(self, other):
        if self.__class__ is other.__class__:
            return self.value >= other.value
        return NotImplemented

    def __gt__(self, other):
        if self.__class__ is other.__class__:
            return self.value > other.value
        return NotImplemented

    def __le__(self, other):
        if self.__class__ is other.__class__:
            return self.value <= other.value
        return NotImplemented

    def __lt__(self, other):
        if self.__class__ is other.__class__:
            return self.value < other.value
        return NotImplemented


class Seen(OrderedEnum):
    Never = 0
    Once = 1
    Many = 2


class SubgraphCounter(object):
    def __init__(self, dgraph_client: DgraphClient, cache: Any = None) -> None:
        self.dgraph_client = dgraph_client
        self.cache = cache

    def get_count_for(self, query: Queryable, max_count: int = 4) -> int:
        """
            Generic caching for a subgraph query
        """

        query_str = generate_query(
            query_name="res", binding_modifier="res", root=query, first=1, count=True
        )

        key = type(self).__name__ + query_str
        cached_count = None

        if self.cache:

            cached_count = self.cache.get(key)
            if cached_count:
                cached_count = int(cached_count)
            if cached_count and cached_count >= max_count:
                return int(cached_count)

        count = query.get_count(self.dgraph_client, first=max_count)

        if self.cache:
            if not cached_count:
                self.cache.set(key, count)
            elif count >= cached_count:
                self.cache.set(key, count)

        return int(count)


class ParentChildCounter(object):
    def __init__(self, dgraph_client: DgraphClient, cache: Any = None) -> None:
        self.dgraph_client = dgraph_client
        self.cache = cache

    def get_count_for(
        self,
        parent_process_name: str,
        child_process_name: Optional[str] = None,
        max_count: int = 4,
    ) -> int:
        """
        Given an image name, and optionally a path, return the number of times
        they occur (alongside one another) in the table.

        If no path is provided, just count the process_name.
        """

        key = type(self).__name__ + parent_process_name + (child_process_name or "")
        cached_count = None

        if self.cache:

            cached_count = self.cache.get(key)
            if cached_count:
                cached_count = int(cached_count)
            if cached_count and cached_count >= max_count:
                return int(cached_count)

        query = (
            ProcessQuery()
            .with_process_name(eq=parent_process_name)
            .with_children(ProcessQuery().with_process_name(eq=child_process_name))
        )  # type: ProcessQuery

        count = query.get_count(self.dgraph_client)

        if self.cache:
            if not cached_count:
                self.cache.set(key, count)
            elif count >= cached_count:
                self.cache.set(key, count)

        return int(count)


class GrandParentGrandChildCounter(object):
    def __init__(self, dgraph_client: DgraphClient, cache: Any = None) -> None:
        self.dgraph_client = dgraph_client
        self.cache = cache

    def get_count_for(
        self,
        grand_parent_process_name: str,
        grand_child_process_name: str,
        max_count: int = 4,
    ) -> int:
        """
        Given an image name, and optionally a path, return the number of times
        they occur (alongside one another) in the table.

        If no path is provided, just count the process_name.
        """

        key = (
            type(self).__name__ + grand_parent_process_name + grand_child_process_name
            or ""
        )

        cached_count = None
        if self.cache:
            cached_count = self.cache.get(key)
            if cached_count:
                cached_count = int(cached_count)
            if cached_count and cached_count >= max_count:
                return int(cached_count)

        query = (
            ProcessQuery()
            .with_process_name(eq=grand_parent_process_name)
            .with_children(
                ProcessQuery().with_children(
                    ProcessQuery().with_process_name(eq=grand_child_process_name)
                )
            )
        )  # type: ProcessQuery

        count = query.get_count(self.dgraph_client)

        if self.cache:
            if not cached_count:
                self.cache.set(key, count)
            elif count >= cached_count:
                self.cache.set(key, count)

        return int(count)
