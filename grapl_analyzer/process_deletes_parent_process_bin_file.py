from typing import Any, Optional

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import SubgraphView, ProcessView


def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):

    suspect_processes = []

    for process in graph.process_iter():
        parent = process.get_parent()
        if not parent: continue

        deleted_files = process.get_deleted_files()
        if not deleted_files: continue

        bin_file = parent.get_bin_file()

        for deleted_file in deleted_files:
            if deleted_file.path != bin_file.path:
                suspect_processes.append(process)


