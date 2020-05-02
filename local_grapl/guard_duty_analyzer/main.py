from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.execution import ExecutionHit
from model_plugins.aws_plugin.guard_duty.guard_duty_finding_node import GuardDutyFindingQuery, GuardDutyFindingView


class GuardDutyFindingAnalyzer(Analyzer):

    def get_queries(self) -> OneOrMany[GuardDutyFindingQuery]:
        return (
            GuardDutyFindingQuery()
        )

    def on_response(self, response: GuardDutyFindingView, output: Any):
        account_id = response.get_account_id()

        output.send(
            ExecutionHit(
                analyzer_name="GuardDuty Finding",
                node_view=response,
                risk_score=75,
                lenses=account_id
            )
        )

