from pydgraph import DgraphClient, DgraphClientStub


class GraphClient(DgraphClient):
    pass


class MasterGraphClient(GraphClient):
    def __init__(self) -> None:
        super(MasterGraphClient, self).__init__(
            DgraphClientStub("alpha0.mastergraphcluster.grapl:9080")
        )


class LocalMasterGraphClient(GraphClient):
    def __init__(self) -> None:
        super(LocalMasterGraphClient, self).__init__(
            DgraphClientStub("master_graph:9080")
        )