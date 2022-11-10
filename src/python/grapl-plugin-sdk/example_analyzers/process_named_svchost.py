"""
This is meant as an extremely simple Analyzer to get the pipeline to fire
during integration tests.
"""
from datetime import datetime

from grapl_plugin_sdk.analyzer.analyzer import (
    Analyzer,
    AnalyzerContext,
    AnalyzerServiceConfig,
    serve_analyzer,
)
from grapl_plugin_sdk.analyzer.query_and_views import NodeQuery, NodeView
from python_proto.api.graph_query.v1beta1.messages import (
    NodePropertyQuery,
    StringFilter,
    StringOperation,
)
from python_proto.api.plugin_sdk.analyzers.v1beta1.messages import (
    AnalyzerName,
    ExecutionHit,
)
from python_proto.common import Timestamp
from python_proto.grapl.common.v1beta1.messages import NodeType, PropertyName


class ProcessNamedSvchost(Analyzer):
    @staticmethod
    def query() -> NodeQuery:
        # Describes a Process where `process_name` = `svchost.exe`
        node_query = NodeQuery(
            NodePropertyQuery(node_type=NodeType(value="Process"))
        ).with_string_filters(
            property_name=PropertyName(value="process_name"),
            filters=[
                StringFilter(
                    operation=StringOperation.EQUAL,
                    value="svchost.exe",
                    negated=False,
                )
            ],
        )

        return node_query

    async def analyze(
        self, matched: NodeView, ctx: AnalyzerContext
    ) -> ExecutionHit | None:
        print(f"analyze() was called: {matched}")
        return ExecutionHit(
            graph_view=matched.graph,
            lens_refs=[],
            idempotency_key=12345,  # ???
            time_of_match=Timestamp.from_datetime(datetime.utcnow()),
            score=100,
            # implies the return type here should not be the pure python-proto type
            # https://github.com/grapl-security/issue-tracker/issues/1032
            analyzer_name=AnalyzerName(
                "TODO: This should be set by AnalyzerServiceImpl"
            ),
        )

    async def add_context(self, matched: NodeView, ctx: AnalyzerContext) -> None:
        pass


def main() -> None:
    """
    main() is invoked by the pex_binary() entrypoint=
    """
    analyzer = ProcessNamedSvchost()
    # Perhaps `serve_analyzer` should just take `(analyzer=analyzer)`?
    # We shouldn't pass on the `AnalyzerServiceConfig` to the consumer, right?
    # https://github.com/grapl-security/issue-tracker/issues/1032
    serve_analyzer(
        analyzer_name=AnalyzerName(
            value="suspicious_svchost"
        ),  # Why is this configured here?
        analyzer=analyzer,
        service_config=AnalyzerServiceConfig.from_env(),
    )
