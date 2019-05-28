import enum
import json
import time
from multiprocessing import Connection

from typing import Any, Iterator, Optional

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import ProcessView, FileView, SubgraphView, NodeView
from grapl_analyzer.entity_queries import ProcessQuery, FileQuery, Not


class Seen(enum.Enum):
    Never = (0,)
    Once = (1,)
    Many = 2


class ParentChildCounter(object):
    def __init__(self, dgraph_client: DgraphClient) -> None:
        self.dgraph_client = dgraph_client

    def get_count_for(
            self,
            parent_image_name: str,
            child_image_name: Optional[str] = None,
            excluding: Optional[str] = None,
    ) -> Seen:
        """
        Given an image name, and optionally a path, return the number of times
        they occur (alongside one another) in the table.

        If no path is provided, just count the image_name.
        """

        count = (
            ProcessQuery()
                .only_first(2)
                .with_image_name(eq=parent_image_name)
                .with_node_key(eq=Not(excluding))
                .with_child(ProcessQuery().with_image_name(eq=child_image_name))
                .get_count(self.dgraph_client)
        )

        if count == 0:
            return Seen.Never
        if count == 1:
            return Seen.Once
        else:
            return Seen.Many