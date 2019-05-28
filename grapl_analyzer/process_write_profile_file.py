from typing import Any, Iterator, Optional

from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import ProcessView, FileView, SubgraphView, NodeView
from grapl_analyzer.entity_queries import ProcessQuery, FileQuery, Not


def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):

    paths = "\.bash_profile|\.bashrc|\.profile"
    suspect_processes = []

    for process in graph.process_iter():

        suspect = (
            ProcessQuery()
            .with_node_key(eq=process.node_key)
            .with_written_file(
                FileQuery()
                .with_path(contains=paths)
            )
        )

        if suspect:
            suspect_processes.append(suspect[0])