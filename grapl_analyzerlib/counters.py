import enum

from typing import Optional, Any

from pydgraph import DgraphClient

from grapl_analyzerlib.entity_queries import Not
from grapl_analyzerlib.entities import ProcessQuery


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


class ParentChildCounter(object):
    def __init__(self, dgraph_client: DgraphClient, cache: Any = None) -> None:
        self.dgraph_client = dgraph_client
        self.cache = cache

    def get_count_for(
        self,
        parent_process_name: str,
        child_process_name: Optional[str] = None,
        excluding: Optional[str] = None,
        max_count: int = 4,
    ) -> Seen:
        """
        Given an image name, and optionally a path, return the number of times
        they occur (alongside one another) in the table.

        If no path is provided, just count the process_name.
        """

        if self.cache:
            key = parent_process_name + child_process_name or ""

            cached_count = int(self.cache.get(key))
            if cached_count and cached_count >= max_count:
                print(f'Cached count: {cached_count}')
                if cached_count == 0:
                    return Seen.Never
                if cached_count == 1:
                    return Seen.Once
                else:
                    return Seen.Many

        query = (
            ProcessQuery()
            .only_first(max_count)
            .with_process_name(eq=parent_process_name)
            .with_children(ProcessQuery().with_process_name(eq=child_process_name))
        )

        if excluding:
            query.with_node_key(Not(excluding))

        count = query.get_count(self.dgraph_client)

        if self.cache:
            if not cached_count:
                self.cache.set(key, count)
            elif count >= cached_count:
                self.cache.set(key, count)

        if count == 0:
            return Seen.Never
        if count == 1:
            return Seen.Once
        else:
            return Seen.Many
