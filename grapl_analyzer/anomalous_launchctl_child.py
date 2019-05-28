import enum
import json
import time
from multiprocessing import Connection

from typing import Any, Iterator, Optional

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import ProcessView, FileView, SubgraphView, NodeView
from grapl_analyzer.entity_queries import ProcessQuery, FileQuery, Not

from grapl_analyzer.counters import ParentChildCounter

def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):
    counter = ProcessPathCounter(client)

    suspect_processes = []

    for process in graph.process_iter():
        parent = process.get_parent()

        if not parent:
            continue

        parent_image = parent.get_image()

        if not parent_image or 'launchctl' not in parent_image:
            continue

        combo_count = counter.get_count_for(
            parent.image_name, process.image_name, excluding=process.node_key
        )

        image_count = counter.get_count_for(parent.image_name, excluding=process.node_key)

        if combo_count == Seen.Once and image_count == Seen.Many:
            suspect_processes.append(process)
