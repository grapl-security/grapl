import os

from typing import Iterator, Tuple

from pydgraph import DgraphClient, DgraphClientStub


def mg_alphas() -> Iterator[Tuple[str, int]]:
    mg_alphas = os.environ["MG_ALPHAS"].split(",")
    for mg_alpha in mg_alphas:
        host, port = mg_alpha.split(":")
        yield host, int(port)


class GraphClient(DgraphClient):
    @classmethod
    def from_host_port(cls, host: str, port: int) -> "GraphClient":
        return cls(*(DgraphClientStub(f"{host}:{port}"),))


class MasterGraphClient(GraphClient):
    def __init__(self) -> None:
        super(MasterGraphClient, self).__init__(
            *(DgraphClientStub(f"{host}:{port}") for host, port in mg_alphas())
        )


class LocalMasterGraphClient(GraphClient):
    def __init__(self) -> None:
        super(LocalMasterGraphClient, self).__init__(
            *(DgraphClientStub(f"{host}:{port}") for host, port in mg_alphas())
        )
