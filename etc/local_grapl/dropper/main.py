from typing import Any

from grapl_analyzerlib.analyzer import Analyzer, OneOrMany
from grapl_analyzerlib.execution import ExecutionHit
from grapl_analyzerlib.prelude import *

def blank_process():
    return ProcessQuery() \
        .with_pid() \
        .with_guid() \
        .with_created_timestamp() \
        .with_cmdline() \
        .with_image() \
        .with_current_directory() \
        .with_user()

def expand_trees(process: ProcessView):
    process.get_created_files()
    process.get_process_connected_to()
    process.get_process_image()

    for child in (process.get_children() or []):
        expand_trees(child)


class Dropper(Analyzer):            
    def get_queries(self) -> OneOrMany[ProcessQuery]:
        dropper_file = FileQuery().with_path()
        second_stage_file = FileQuery().with_path()
        
        second_stage_process = blank_process()
        
        lolbin_process = (
            blank_process()
                .with_children(
                    second_stage_process
                )
            )
        
        dropper_process = blank_process() \
            .with_created_files(
                second_stage_file
                    .with_process_executed_from_image(
                        lolbin_process
                    )
            ) \
            .with_process_connected_to() \
            .with_children(
                lolbin_process
            )
        
        chrome_process = blank_process() \
            .with_process_image(
                FileQuery()
                    .with_path(eq="C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe")
            ) \
            .with_created_files(
                dropper_file \
                    .with_process_executed_from_image(
                        dropper_process
                    )
            ) \
            .with_children(
                dropper_process
            )
        
        return chrome_process

    def on_response(self, response: ProcessView, output: Any):
        response.get_created_files()
        response.get_process_image()

        for child in response.children:
            expand_trees(child)

        output.send(
            ExecutionHit(
                analyzer_name="Dropper",
                node_view=response,
                risk_score=75,
                lenses=[
                    ("dropper", "chrome.exe"),
                ],
                risky_node_keys=[
                    # the asset and the process
                    response.node_key,
                ],
            )
        )
