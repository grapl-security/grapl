import json
from typing import Union, cast, List, Optional


class ExecutionHit(object):
    def __init__(
            self,
            analyzer_name: str,
            node_view: 'Accepts',
            risk_score: int,
            correlation_points: Optional[List[str]] = None
    ) -> None:
        node_view = cast(NodeView, NodeView.from_view(node_view))
        self.root_node_key = node_view.node.node_key

        if correlation_points:
            raise NotImplementedError("Correlation points are not currently implemented")

        node_dict = node_view.to_adjacency_list()
        self.analyzer_name = analyzer_name
        self.nodes = json.dumps(node_dict["nodes"])
        self.edges = json.dumps(node_dict["edges"])
        self.correlation_points = correlation_points or []
        self.risk_score = risk_score


class ExecutionComplete(object):
    pass


class ExecutionFailed(object):
    pass

from grapl_analyzerlib.nodes.any_node import _NodeView
from grapl_analyzerlib.nodes.dynamic_node import _DynamicNodeView
from grapl_analyzerlib.nodes.external_ip_node import _ExternalIpView
from grapl_analyzerlib.nodes.file_node import _FileView
from grapl_analyzerlib.nodes.process_node import _ProcessView
from grapl_analyzerlib.prelude import NodeView

Accepts = Union[
    _NodeView,
    _ProcessView,
    _FileView,
    _DynamicNodeView,
    _ExternalIpView,
]
