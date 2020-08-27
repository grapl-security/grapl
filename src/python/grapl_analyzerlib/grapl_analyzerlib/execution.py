import json
from typing import Union, cast, List, Tuple


class ExecutionHit(object):
    def __init__(
        self,
        analyzer_name: str,
        node_view: "EntityView",
        risk_score: int,
        lenses: List[Tuple[str, str]],
    ) -> None:
        self.root_node_key = node_view.node_key

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


from grapl_analyzerlib.nodes.entity import EntityView
