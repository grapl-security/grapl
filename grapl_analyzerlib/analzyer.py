import abc

from typing import Any, Type, TypeVar, List, Union

from grapl_analyzerlib.entities import ProcessQuery, ProcessView, FileQuery
from pydgraph import DgraphClient

from grapl_analyzerlib.execution import ExecutionHit
from grapl_analyzerlib.querying import Viewable, Queryable

A = TypeVar("A", bound="Analyzer")

T = TypeVar("T")
OneOrMany = Union[T, List[T]]


class Analyzer(abc.ABC):
    def __init__(self, dgraph_client: DgraphClient) -> None:
        self.dgraph_client = dgraph_client

    @classmethod
    @abc.abstractmethod
    def build(cls: Type[A], dgraph_client: DgraphClient) -> A:
        pass

    @abc.abstractmethod
    def get_queries(self) -> OneOrMany[Queryable]:
        pass

    @abc.abstractmethod
    def on_response(self, response: Viewable, output: Any):
        pass


class HistoryRemovalAnalyzer(Analyzer):
    def __init__(self, dgraph_client: DgraphClient) -> None:
        super(HistoryRemovalAnalyzer).__init__(dgraph_client)

    @classmethod
    def build(cls, dgraph_client: DgraphClient) -> A:
        return HistoryRemovalAnalyzer(dgraph_client)

    def get_queries(self) -> ProcessQuery:
        return (
            ProcessQuery()
            .with_deleted_files(
                FileQuery()
                .with_file_path(ends_with="_history")
            )
        )

    def on_response(self, response: ProcessView, output: Any):
        output.send(
            ExecutionHit(
                analyzer_name="HistoryRemoval",
                node_view=response,
                risk_score=75
            )
        )


class ExampleAnalyzer(Analyzer):
    def __init__(self, client: DgraphClient, some_other_dep: Any) -> None:
        super(ExampleAnalyzer, self).__init__(client)
        self.some_other_dep = some_other_dep

    @classmethod
    def build(cls, client: DgraphClient):
        return ExampleAnalyzer(client, "otherdep")

    def get_query(self):
        return ProcessQuery().with_process_name(eq="evil.exe")

    def on_response(self, response: ProcessView, output: Any):
        output.send(ExecutionHit(
            "ExampleAnalyzer",
            node_view=response,
            risk_score=10
        ))
