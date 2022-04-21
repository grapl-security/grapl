import json
import re
from pathlib import Path
from typing import List, Optional

# Declares the order in which to print diagnoses out.
# Models the order of messages flowing through the pipeline.
PRIORITIZATION = [
    "sysmon-generator.stdout",
    "osquery-generator.stdout",
    "node-identifier.stdout",
    "node-identifier-retry.stdout",
    "graph-merger.stdout",
    "analyzer-dispatcher.stdout",
    "analyzer-executor.stdout",
    "engagement-creator.stdout",
]


def analyze_grapl_core(artifacts_dir: Path, analysis_dir: Path) -> None:
    logs_in_grapl_core = list((artifacts_dir / "grapl-core").iterdir())
    # Find any logs that match the patterns listed in PRIORITIZATION,
    # in that order.
    maybe_paths = [
        next((l for l in logs_in_grapl_core if l.name.startswith(pattern)), None)
        for pattern in PRIORITIZATION
    ]
    paths = [p for p in maybe_paths if p]
    output = analyze_paths(paths)
    with open(analysis_dir / "pipeline_message_flow.txt", "w") as f:
        f.write(output)


def analyze_paths(paths: List[Path]) -> str:
    output = []
    for path in paths:
        with open(path, "r") as file:
            lines = file.readlines()
            num_received_messages = get_num_received_messages(lines)
            output.append(f"File {path.name}: received {num_received_messages}")
    return "\n".join(output)


########################################
# Helpers
########################################


def get_num_received_messages(lines: List[str]) -> int:
    """
    Given a split-lines input, count up all receives.
    """
    total_receives = 0
    for line in lines:
        total_receives += get_receive_count_for_line(line) or 0
    return total_receives


def get_receive_count_for_line(line: str) -> Optional[int]:
    """
    Given a single line, parse for receive-count data.
    """
    # Try different heuristics for the same line depending on which log format
    res = get_receive_count_for_rust_sqs_executor_line(line)
    if res is None:
        res = get_receive_count_for_py_sqs_timeout_manager(line)
    return res


def get_receive_count_for_rust_sqs_executor_line(line: str) -> Optional[int]:
    """
    Corresponds to logs output from
    info!(message_batch_len = message_batch_len, "Received messages");
    in sqs-executor/lib.rs
    """
    try:
        json_line = json.loads(line)
    except json.decoder.JSONDecodeError:
        # Definitely not handlable by the rust sqs_executor parser
        return None
    try:
        message = json_line["fields"]["message"]
        if not message == "Received messages":
            return None
        received_batch_len: int = json_line["fields"]["message_batch_len"]
        return received_batch_len
    except KeyError:
        return None


PY_SQS_TIMEOUT_MANAGER_PATTERN = re.compile(r"SQS MessageID [0-9a-f\-]+\: Loop 1 .*")


def get_receive_count_for_py_sqs_timeout_manager(line: str) -> Optional[int]:
    """
    Corresponds to logs output from
    sqs_timeout_manager.py
    """
    # Not necessarily the best signal, but since SQS is being deprecated soon
    # due to kafka, it's fine for now
    if re.match(PY_SQS_TIMEOUT_MANAGER_PATTERN, line):
        return 1
    return None
