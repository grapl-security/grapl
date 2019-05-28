from typing import Any, Optional

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import SubgraphView, ProcessView
from grapl_analyzer.entity_queries import ProcessQuery

def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):
    shells = ['/bin/sh', '/bin/dash', '/bin/bash']
    suspect_processes = []

    for process in graph.process_iter():
        shell = process.get_image_name()
        if shell not in shells:
            continue

        parent = shell.get_parent()
        if not parent:
            continue

        parent_image = parent.get_image_name()
        if 'python' not in parent_image:
            continue

        if 'ruby' not in parent_image:
            continue

        suspect_processes.append(parent)