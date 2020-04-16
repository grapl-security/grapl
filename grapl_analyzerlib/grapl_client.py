from pydgraph import DgraphClient, DgraphClientStub


class GraphClient(DgraphClient):
    pass


class MasterGraphClient(GraphClient):
    def __init__(self) -> None:
        super(MasterGraphClient, self).__init__(
            DgraphClientStub('alpha0.mastergraphcluster.grapl:9080')
        )


class LocalMasterGraphClient(GraphClient):
    def __init__(self) -> None:
        super(LocalMasterGraphClient, self).__init__(
            DgraphClientStub('master_graph:9080')
        )


class EngagementGraphClient(GraphClient):
    def __init__(self) -> None:
        super(EngagementGraphClient, self).__init__(
            DgraphClientStub('alpha0.engagementgraphcluster.grapl:9080')
        )


class LocalEngagementGraphClient(GraphClient):
    def __init__(self) -> None:
        super(LocalEngagementGraphClient, self).__init__(
            DgraphClientStub('engagement_graph:9080')
        )
