from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import Not, AssetQuery, ProcessQuery, ProcessView
from grapl_analyzerlib.execution import ExecutionHit


class SuspiciousSvchost(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        invalid_parents = [
            Not("services.exe"),
            Not("smss.exe"),
            Not("ngentask.exe"),
            Not("userinit.exe"),
            Not("GoogleUpdate.exe"),
            Not("conhost.exe"),
            Not("MpCmdRun.exe"),
        ]

        return (
            ProcessQuery()
            .with_process_name(eq=invalid_parents)
            .with_children(ProcessQuery().with_process_name(eq="svchost.exe"))
            .with_asset(AssetQuery().with_hostname())
        )

    def on_response(self, response: ProcessView, output: Any):
        asset_id = response.get_asset().get_hostname()

        output.send(
            ExecutionHit(
                analyzer_name="Suspicious svchost",
                node_view=response,
                risk_score=75,
                lenses=[
                    ("hostname", asset_id),
                    ("username", "hardcodeduser"),
                    ("lens_type_0", "lens_value_0"),
                    ("lens_type_1", "lens_value_1"),
                    ("lens_type_2", "lens_value_2"),
                    ("lens_type_3", "lens_value_3"),
                    ("lens_type_4", "lens_value_4")
                ],
                risky_node_keys=[
                    # the asset and the process
                    response.get_asset().node_key,
                    response.node_key,
                ],
            )
        )
