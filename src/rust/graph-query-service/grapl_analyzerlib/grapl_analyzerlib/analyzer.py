import abc
from typing import Any, Type, TypeVar, List, Union

from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.grapl_client import GraphClient

A = TypeVar("A", bound="Analyzer")

T = TypeVar("T")
OneOrMany = Union[T, List[T]]


class Analyzer(abc.ABC):
    def __init__(self, graph_client: GraphClient) -> None:
        self.graph_client = graph_client

    @classmethod
    def build(cls: Type[A], graph_client: GraphClient) -> A:
        return cls(graph_client)

    @abc.abstractmethod
    def get_queries(self) -> OneOrMany[Queryable]:
        pass

    @abc.abstractmethod
    def on_response(self, response: Viewable, output: Any):
        pass
