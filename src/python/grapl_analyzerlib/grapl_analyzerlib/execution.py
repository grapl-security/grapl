import json
from typing import Union, cast, List, Sequence, Optional


class ExecutionHit(object):
    def __init__(
        self,
        analyzer_name: str,
        node_view: "Accepts",
        risk_score: int,
        lenses: Union[List[str], str],
        risky_nodes: Optional[Sequence[str]] = None,
    ) -> None:
        """
        When an Analyzer finds a risk, its' :py:meth:`~grapl_analyzerlib.analyzer.Analyzer.on_response` method will
        send ExecutionHit(s) over to EngagementCreator. Basically, an ExecutionHit is a minimal, serializable
        representation of a future Risk that has yet to be written to the db.

        :param analyzer_name:
        :param node_view:
        :param risk_score:
        :param lenses:
        :param risky_nodes: identify which of the nodes in the graph should be attached to the Risk,
        versus those that are just supplying context. If left as None, all nodes are considered risks.

        .. TODO wimax Aug 2020: Update `implementing.md` to mention `risky_nodes`
        """
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
        self.risky_nodes = risky_nodes


class ExecutionComplete(object):
    pass


class ExecutionFailed(object):
    pass


from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeView
from grapl_analyzerlib.nodes.file_node import FileView
from grapl_analyzerlib.nodes.process_node import ProcessView
from grapl_analyzerlib.prelude import NodeView

Accepts = Union[NodeView, ProcessView, FileView, DynamicNodeView]
