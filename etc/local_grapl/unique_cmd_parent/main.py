from typing import Any, Type

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany, A
from grapl_analyzerlib.counters import ParentChildCounter
from grapl_analyzerlib.execution import ExecutionHit
from grapl_analyzerlib.prelude import (
    AssetQuery,
    ProcessQuery,
    ProcessView,
    Not,
    GraphClient,
)


class RareParentOfCmd(Analyzer):
    def __init__(self, graph_client: GraphClient, counter: ParentChildCounter):
        super(RareParentOfCmd, self).__init__(graph_client)
        self.counter = counter

    @classmethod
    def build(cls: Type[A], graph_client: GraphClient) -> A:
        counter = ParentChildCounter(graph_client)
        return RareParentOfCmd(graph_client, counter)

    def get_queries(self) -> OneOrMany[ProcessQuery]:
        # TODO: We should be checking binary paths for these to ensure we handle impersonation
        parent_whitelist = [
            Not("svchost.exe"),
            Not("RuntimeBroker.exe"),
            Not("chrome.exe"),
            Not("explorer.exe"),
            Not("SIHClient.exe"),
            Not("conhost.exe"),
            Not("MpCmdRun.exe"),
            Not("GoogleUpdateComRegisterShell64.exe"),
            Not("GoogleUpdate.exe"),
            Not("notepad.exe"),
            Not("OneDrive.exe"),
            Not("VBoxTray.exe"),
            Not("Firefox Installer.exe"),
        ]

        return (
            ProcessQuery()
            .with_process_name(eq=parent_whitelist)
            .with_children(ProcessQuery().with_process_name(eq="cmd.exe"))
            .with_asset(AssetQuery().with_hostname())
        )

    def on_response(self, response: ProcessView, output: Any) -> None:
        count = self.counter.get_count_for(
            parent_process_name=response.get_process_name(),
            child_process_name="cmd.exe",
        )

        asset_id = response.get_asset().get_hostname()

        if count <= 3:
            output.send(
                ExecutionHit(
                    analyzer_name="Rare Parent of cmd.exe",
                    node_view=response,
                    risk_score=10,
                    lenses=[("hostname", asset_id)],
                )
            )
