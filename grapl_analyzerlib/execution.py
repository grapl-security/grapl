import json

from grapl_analyzerlib.entities import NodeView


class ExecutionHit(object):
    def __init__(self, analyzer_name: str, node_view: NodeView, risk_score: int = 50):
        self.root_node_key = node_view.node.node_key

        node_dict = node_view.to_adjacency_list()
        self.analyzer_name = analyzer_name
        self.nodes = json.dumps(node_dict['nodes'])
        self.edges = json.dumps(node_dict['edges'])
        self.risk_score = risk_score


class ExecutionComplete(object):
    pass


class ExecutionFailed(object):
    pass
