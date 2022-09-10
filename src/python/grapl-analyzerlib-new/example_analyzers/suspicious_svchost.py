from datetime import datetime
from grapl_analyzerlib_new.analyzer import Analyzer, AnalyzerServiceConfig, serve_analyzer, AnalyzerContext
from python_proto.api.graph_query.v1beta1.messages import NodePropertyQuery, StringFilter, StringOperation
from python_proto.api.plugin_sdk.analyzers.v1beta1.messages import AnalyzerName, ExecutionHit, LensRef
from grapl_analyzerlib_new.query_and_views import NodeQuery, NodeView
from python_proto.common import Timestamp
from python_proto.grapl.common.v1beta1.messages import EdgeName, NodeType, PropertyName

class SuspiciousSvchostAnalyzer(Analyzer):
    @staticmethod
    def query() -> NodeQuery:
        # Query for parent Process nodes
        parent_node_query = NodeQuery(
            NodePropertyQuery(node_type=NodeType(value="Process"))
        )

        # Where process_name is not any of the following.
        invalid_parents = [
            "services.exe",
            "smss.exe",
            "ngentask.exe",
            "userinit.exe",
            "GoogleUpdate.exe",
            "conhost.exe",
            "MpCmdRun.exe",
        ]
        for invalid_parent in invalid_parents:
            parent_node_query.with_string_filters(
                property_name=PropertyName(value="process_name"),
                filters=[
                    StringFilter(
                        operation=StringOperation.EQUAL,
                        value=invalid_parent,
                        negated=True,
                    ),
                ]
            )
        
        # Describe the susupicious Child node: a Process named svchost.exe
        child_node_query = NodeQuery(
            NodePropertyQuery(node_type=NodeType(value="Process"))
        ).with_string_filters(
            property_name=PropertyName(value="process_name"),
            filters=[
                StringFilter(
                    operation=StringOperation.EQUAL,
                    value="svchost.exe",
                    negated=False,
                )
            ]
        )
        
        return parent_node_query.with_edge_filter(
            edge_name=EdgeName("children"),
            reverse_edge_name=EdgeName("parent"),
            edge_filter=child_node_query
        )

    async def analyze(
        self, matched: NodeView, ctx: AnalyzerContext
    ) -> ExecutionHit | None:
        print(f"Oh dude we did it {matched}")
        return ExecutionHit(
            lens_refs=[],
            analyzer_name=AnalyzerName(value=self.__class__.__name__),
            idempotency_key=12345, # ???
            time_of_match=Timestamp.from_datetime(datetime.utcnow()),
            score=100,
        )

    async def add_context(self, matched: NodeView, ctx: AnalyzerContext) -> None:
        pass


async def main() -> None:
    analyzer = SuspiciousSvchostAnalyzer()
    # Perhaps `serve_analyzer` should just take `(analyzer=analyzer)`?
    serve_analyzer(
        analyzer_name=AnalyzerName(value="suspicious_svchost"), # Why is this configured here?
        analyzer=analyzer,
        service_config=AnalyzerServiceConfig.from_env(),
    )
    
if __name__ == "__main__":
    main()