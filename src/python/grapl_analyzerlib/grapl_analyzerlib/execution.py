import json
from typing import List, Tuple, Optional, Sequence


class ExecutionHit(object):
    def __init__(
        self,
        analyzer_name: str,
        node_view: "EntityView",
        risk_score: int,
        lenses: List[Tuple[str, str]],
        risky_node_keys: Optional[Sequence[str]] = None,
    ) -> None:
        """
        When an Analyzer finds a risk, its :py:meth:`~grapl_analyzerlib.analyzer.Analyzer.on_response` method will
        send ExecutionHit(s) over to EngagementCreator. Basically, an ExecutionHit is a minimal, serializable
        representation of a future Risk that has yet to be written to the db.

        :param analyzer_name:
        :param node_view:
        :param risk_score:
        :param lenses:
        :param risky_node_keys: identify which of the nodes in the graph should be attached to the Risk,
        versus those that are just supplying context. If left as None, all nodes are considered risks.

        .. TODO wimax Aug 2020: Update `implementing.md` to mention `risky_nodes`
        """
        self.root_node_key = node_view.node_key

        node_dict = node_view.to_adjacency_list()
        self.analyzer_name = analyzer_name
        self.nodes = json.dumps(node_dict["nodes"])
        self.edges = json.dumps(node_dict["edges"])
        self.lenses = lenses
        self.risk_score = risk_score
        self.risky_node_keys = risky_node_keys

        for lens_key, lens_value in lenses:
            if lens_key is None or lens_value is None:
                raise TypeError(f"Found an unexpected None k/v in lenses: {lenses}")


class ExecutionComplete(object):
    pass


class ExecutionFailed(object):
    pass


from grapl_analyzerlib.nodes.entity import EntityView
