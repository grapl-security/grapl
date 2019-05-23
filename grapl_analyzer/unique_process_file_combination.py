import enum
import json
import time
from multiprocessing import Connection

from typing import Any, Iterator, Optional

import boto3

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import ProcessView, FileView, SubgraphView, NodeView
from grapl_analyzer.entity_queries import ProcessQuery, FileQuery

# Search for proc nodes with an image_name and bin_file with a path
# where the process was created in the last N months but has a node_key
# of *not* at least one of the nodes



class Seen(enum.Enum):
    Never = 0,
    Once = 1,
    Many = 2


class ProcessPathCounter(object):
    def __init__(
            self,
            dgraph_client: DgraphClient,
    ) -> None:
        self.dgraph_client = dgraph_client

    def get_count_for(self,
                      image_name: str,
                      path: Optional[str] = None,
                      excluding: Optional[str] = None
                      ) -> Seen:
        """
        Given an image name, and optionally a path, return the number of times
        they occur (alongside one another) in the table.

        If no path is provided, just count the image_name.

        :param image_name:
        :param path:
        :param excluding:
        :return:
        """

        excluding_filter = ""
        if excluding:
            excluding_filter += "@filter(not eq(node_key, {}))".format(excluding)

        path_filter = ""
        if path:
            path_filter = "@filter(eq(path, {}))".format(path)

        query = """
            {{
                var(func: eq(image_name, {}, first: 2) @cascade 
                {}
                {{
                    uid,
                    bin_file {} {{
                        u as uid,
                    }}
    
                }}
                
                result(func: uid(u)) {{
                  count(uid)
                }}
            }}
            """.format(image_name, excluding_filter, path_filter)

        count = json.loads(
            self.dgraph_client
                .txn(read_only=True)
                .query(query)
                .json
        )['count']

        if count == 0: return Seen.Never
        if count == 1: return Seen.Once
        else: return Seen.Many


def process_and_bin_file(dgraph_client: DgraphClient, node_key: str) -> ProcessView:
    query = ProcessQuery() \
        .with_node_key(eq=node_key)\
        .with_create_time() \
        .with_bin_file(
        FileQuery()
            .with_path()
    ) \
        .to_query()

    res = json.loads(
        dgraph_client
            .txn(read_only=True)
            .query(query)
            .json
    )

    process = res['q0']

    return ProcessView.from_dict(dgraph_client, process)


def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):
    counter = ProcessPathCounter(client)

    suspect_processes = []

    for node_key, node in graph.nodes:
        process = node.as_process_view()  # type: Optional[ProcessView]
        combo_count = counter.get_count_for(
            process.image_name,
            process.bin_file.path,
            excluding=node_key
        )

        image_count = counter.get_count_for(
            process.image_name,
            excluding=node_key
        )

        if combo_count == Seen.Once and image_count == Seen.Many:
            suspect_processes.append(process)
