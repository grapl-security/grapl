from grapl_analyzerlib.entities import NodeView


class ExecutionHit(object):
    def __init__(self, analyzer_name: str, node_view: NodeView, risk_score: int = 50):
        self.node_view = node_view
        self.analyzer_name = analyzer_name
        self.risk_score = risk_score


class ExecutionComplete(object):
    pass


class ExecutionFailed(object):
    pass
