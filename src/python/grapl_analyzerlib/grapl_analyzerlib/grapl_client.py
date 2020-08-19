import os

from typing import Iterable, Iterator, Optional, Tuple

from pydgraph import DgraphClient, DgraphClientStub


def mg_alphas() -> Iterator[Tuple[str, int]]:
    mg_alphas = os.environ["MG_ALPHAS"].split(",")
    for mg_alpha in mg_alphas:
        host, port = mg_alpha.split(":")
        yield host, int(port)


class GraphClient(DgraphClient):
    def __init__(self, alphas: Optional[Iterable[str]] = None):
        if alphas is None:
            super(GraphClient, self).__init__(
                *(DgraphClientStub(f"{host}:{port}") for host, port in mg_alphas())
            )
        else:
            super(GraphClient, self).__init__(
                *(DgraphClientStub(alpha) for alpha in alphas)
            )


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
