from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView


def process_context(process: ProcessView) -> None:
    process.get_pid()
    process.get_guid()
    process.get_created_timestamp()
    process.get_cmdline()
    process.get_image()
    process.get_current_directory()
    process.get_user()


def expand_trees(process: ProcessView):
    process.get_created_files()
    process.get_process_connected_to()
    process.get_process_image()

    for child in (process.get_children() or []):
        expand_trees(child)

class Dropper(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        second_stage_process = ProcessQuery().with_image()
        return (
            ProcessQuery()
            .with_image()
            .with_process_connected_to()
            .with_created_files(
                FileQuery()
                .with_path()
                .with_process_executed_from_image(second_stage_process)
            )
            .with_children(second_stage_process)
        )

    def on_response(self, dropper: ProcessView, output: Any) -> None:
        process_context(dropper)
        dropper.get_created_files()
        dropper.get_process_image()
        expand_trees(dropper)

        output.send(
            ExecutionHit(
                analyzer_name="Dropper",
                node_view=dropper,
                risk_score=75,
                lenses=[
                    ("analyzer_name", "Dropper"),
                    ("user", dropper.get_user()),
                ],
                # Mark the dropper and its child processes as risky
                risky_node_keys=[
                    dropper.node_key,
                    *[child.node_key for child in dropper.children]
                ],
            )
        )

