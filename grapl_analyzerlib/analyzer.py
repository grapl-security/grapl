import abc
from typing import Any, Type, TypeVar, List, Union

from grapl_analyzerlib.nodes.viewable import Viewable
from grapl_analyzerlib.nodes.queryable import Queryable
from pydgraph import DgraphClient

A = TypeVar("A", bound="Analyzer")

T = TypeVar("T")
OneOrMany = Union[T, List[T]]


class Analyzer(abc.ABC):
    def __init__(self, dgraph_client: DgraphClient) -> None:
        self.dgraph_client = dgraph_client

    @classmethod
    def build(cls: Type[A], dgraph_client: DgraphClient) -> A:
        return cls(dgraph_client)

    @abc.abstractmethod
    def get_queries(self) -> OneOrMany[Queryable]:
        pass

    @abc.abstractmethod
    def on_response(self, response: Viewable, output: Any):
        pass
