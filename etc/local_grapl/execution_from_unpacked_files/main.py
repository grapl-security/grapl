from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.prelude import *

from grapl_analyzerlib.nodes.sysmon import ProcessView


def process_context(process: ProcessView) -> None:
    process.get_pid()
    process.get_guid()
    process.get_exe()

def expand_trees(process: ProcessView):
    process.get_created_files()
    process.get_process_socket_outbound()
    process.get_process_exe()

    for child in (process.get_children() or []):
        expand_trees(child)

class UnpackedFileExecution(Analyzer):
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        unpacker_names = ["7zip.exe", "winrar.exe", "zip.exe"]

        second_stage_process = ProcessQuery().with_exe()
        return (
            ProcessQuery()
                #TODO we don't want 'eq' we want ends_with - do we have that?
                .with_exe(eq=unpacker_names)
                .with_created_files(
                    FileQuery()
                        .with_path()
                        .with_process_executed_from_exe(second_stage_process)
                )
                .with_children(second_stage_process)
        )

    def on_response(self, response_view: ProcessView, output: Any) -> None:
        process_context(response_view)
        response_view.get_created_files()
        response_view.get_process_exe()
        expand_trees(response_view)

        output.send(
            ExecutionHit(
                analyzer_name="Unpacked file execution",
                node_view=response_view,
                risk_score=75,
                lenses=[
                    ("analyzer_name", "Unpacked file execution"),
                    ("exe", response_view.get_exe()),
                ],
                risky_node_keys=[
                    response_view.node_key,
                    *[child.node_key for child in response_view.children]
                ],
            )
        )

