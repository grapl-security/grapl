from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzer.entity_views import SubgraphView


def _analyzer(client: DgraphClient, graph: SubgraphView, sender: Any):
    suspect_processes = []

    for process in graph.process_iter():
        image_name = process.get_image_name()
        if "regsvr32.exe" not in image_name:
            continue



