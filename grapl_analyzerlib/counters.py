import enum

from typing import Optional

from pydgraph import DgraphClient

from grapl_analyzerlib.entity_queries import Not
from grapl_analyzerlib.entities import ProcessQuery


class Seen(enum.Enum):
    Never = (0,)
    Once = (1,)
    Many = 2


class ParentChildCounter(object):
    def __init__(self, dgraph_client: DgraphClient) -> None:
        self.dgraph_client = dgraph_client

    def get_count_for(
        self,
        parent_process_name: str,
        child_process_name: Optional[str] = None,
        excluding: Optional[str] = None,
    ) -> Seen:
        """
        Given an image name, and optionally a path, return the number of times
        they occur (alongside one another) in the table.

        If no path is provided, just count the process_name.
        """

        query = (
            ProcessQuery()
            .only_first(2)
            .with_process_name(eq=parent_process_name)
            .with_children(ProcessQuery().with_process_name(eq=child_process_name))
        )

        if excluding:
            query.with_node_key(Not(excluding))

        count = query.get_count(self.dgraph_client)

        if count == 0:
            return Seen.Never
        if count == 1:
            return Seen.Once
        else:
            return Seen.Many
