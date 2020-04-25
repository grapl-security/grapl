import json
from typing import Union, cast, List


class ExecutionHit(object):
    def __init__(
        self,
        analyzer_name: str,
        node_view: "Accepts",
        risk_score: int,
        lenses: Union[List[str], str],
    ) -> None:
        node_view = cast(NodeView, NodeView.from_view(node_view))
        self.root_node_key = node_view.node.node_key

        if isinstance(lenses, str):
            lenses = [lenses]

        node_dict = node_view.to_adjacency_list()
        self.analyzer_name = analyzer_name
        self.nodes = json.dumps(node_dict["nodes"])
        self.edges = json.dumps(node_dict["edges"])
        self.lenses = lenses
        self.risk_score = risk_score


class ExecutionComplete(object):
    pass


class ExecutionFailed(object):
    pass


from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeView
from grapl_analyzerlib.nodes.file_node import FileView
from grapl_analyzerlib.nodes.process_node import ProcessView
from grapl_analyzerlib.prelude import NodeView

Accepts = Union[NodeView, ProcessView, FileView, DynamicNodeView]
