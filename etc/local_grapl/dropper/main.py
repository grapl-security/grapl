from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView


def process_context(process: ProcessView) -> None:
    process.get_pid()
    process.get_guid()
    process.get_exe()

# traverse the outbound connection to include TcpConnection and 
# NetworkSocketAddress node that's on the receiving end.
# def process_connection_outbound_context(process: ProcessView) -> None:
#     (
#         process.get_process_socket_outbound()
#     )


def expand_trees(process: ProcessView):
    process.get_created_files()
    process.get_process_socket_outbound()
    process.get_process_exe()

    for child in (process.get_children() or []):
        expand_trees(child)

class Dropper(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        second_stage_process = ProcessQuery().with_exe()
        return (
            ProcessQuery()
            .with_exe()
            .with_process_socket_outbound()
            .with_created_files(
                FileQuery()
                .with_path()
                .with_process_executed_from_exe(second_stage_process)
            )
            .with_children(second_stage_process)
        )

    def on_response(self, dropper: ProcessView, output: Any) -> None:
        process_context(dropper)
        dropper.get_created_files()
        dropper.get_process_exe()
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

