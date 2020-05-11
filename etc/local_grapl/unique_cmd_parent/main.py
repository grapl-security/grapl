import os
from typing import Any, Type

import redis
from grapl_analyzerlib.analyzer import Analyzer, OneOrMany, A
from grapl_analyzerlib.counters import ParentChildCounter
from grapl_analyzerlib.prelude import ProcessQuery, ProcessView, Not
from grapl_analyzerlib.execution import ExecutionHit
from pydgraph import DgraphClient


class RareParentOfCmd(Analyzer):

    def __init__(self, dgraph_client: DgraphClient, counter: ParentChildCounter):
        super(RareParentOfCmd, self).__init__(dgraph_client)
        self.counter = counter

    @classmethod
    def build(cls: Type[A], dgraph_client: DgraphClient) -> A:
        counter = ParentChildCounter(dgraph_client)
        return RareParentOfCmd(dgraph_client, counter)

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
            .with_children(
                ProcessQuery()
                .with_process_name(eq="cmd.exe")
            )
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
                    lenses=[asset_id]
                )
            )
