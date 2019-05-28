from typing import Any

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import SubgraphView

def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):

    suspect_processes = []

    for process in graph.process_iter():
        deleted_files = process.get_deleted_files() or []
        bin_file = process.get_bin_files()

        for deleted_file in deleted_files:
            if deleted_file.path != bin_file.path:
                suspect_processes.append(process)


