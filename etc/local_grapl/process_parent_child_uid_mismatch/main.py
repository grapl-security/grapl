from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView

class ProcessParentChildUidMismatch(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessSpawnQuery]:
        return (
            ProcessSpawnQuery()
                .with_user()
                .with_parent_user()
        )

    def on_response(self, proc_spawn_view: ProcessSpawnView, output: Any) -> None:
        child_user = proc_spawn_view.get_user()
        parent_user = proc_spawn_view.get_parent_user()

        if child_user != parent_user:
            output.send(
                ExecutionHit(
                    analyzer_name="ProcessParentChildUidMismatch",
                    node_view=proc_spawn_view,
                    risk_score=75,
                    lenses=[
                        ("analyzer_name", "Process user mismatch"),
                        ("user", proc_spawn_view.get_user()),
                    ],
                    # Mark the dropper and its child processes as risky
                    risky_node_keys=[
                        proc_spawn_view.node_key,
                        proc_spawn_view.spawned_b.node_key,
                        proc_spawn_view.spawned_by_a.node_key,
                    ],
                )
            )