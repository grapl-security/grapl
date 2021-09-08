#!/usr/bin/env python3
from __future__ import annotations

import dataclasses
import logging
import os
import shutil
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from nomad import Nomad
from typing_extensions import Literal

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

OutOrErr = Union[Literal["stdout", "stderr"]]
OUTPUT_TYPES: List[OutOrErr] = ["stdout", "stderr"]

nomad_agent_log_path = Path("/tmp/nomad-agent.log").resolve()
consul_agent_log_path = Path("/tmp/consul-agent.log").resolve()


def dump_all(artifacts_dir: Path) -> None:
    nomad_artifacts_dir = artifacts_dir / "nomad"
    os.makedirs(nomad_artifacts_dir, exist_ok=True)
    _dump_nomad_consul_agent_logs(nomad_artifacts_dir)
    _get_nomad_logs_for_each_service(nomad_artifacts_dir)


def _dump_nomad_consul_agent_logs(artifacts_dir: Path) -> None:
    shutil.copy2(nomad_agent_log_path, artifacts_dir)
    shutil.copy2(consul_agent_log_path, artifacts_dir)


@dataclasses.dataclass
class NomadAllocation:
    allocation_id: str
    allocation_name: str
    tasks: List[NomadTask]

    def __init__(self, input: Dict[str, Any]) -> None:
        self.allocation_id = input["ID"]
        self.allocation_name = input["Name"]
        # Remove tasks we don't super care about
        task_names = [
            t for t in input["TaskStates"].keys() if not t.startswith("connect-proxy")
        ]
        self.tasks = [NomadTask(parent=self, name=t) for t in task_names]

    def short_alloc_id(self) -> str:
        return self.allocation_id.split("-")[0]


@dataclasses.dataclass
class NomadTask:
    name: str
    parent: NomadAllocation = dataclasses.field(repr=False)

    def get_logs(self, nomad_client: Nomad, type: OutOrErr) -> str:
        return nomad_client.client.stream_logs.stream(
            self.parent.allocation_id, self.name, type=type, plain=True
        )


def _get_nomad_logs_for_each_service(
    artifacts_dir: Path,
) -> Dict[str, List[NomadAllocation]]:
    nomad = Nomad(host="localhost", timeout=5)
    job_to_allocs: Dict[str, List[NomadAllocation]] = {
        job_name: [NomadAllocation(a) for a in nomad.job.get_allocations(job_name)]
        for job_name in ("grapl-core", "grapl-local-infra", "integration-tests")
    }

    for job, allocs in job_to_allocs.items():
        _write_nomad_logs(nomad, artifacts_dir, job_name=job, allocs=allocs)

    return job_to_allocs


def _write_nomad_logs(
    nomad_client: Nomad,
    artifacts_dir: Path,
    job_name: str,
    allocs: List[NomadAllocation],
) -> None:
    write_to_dir = artifacts_dir / job_name
    os.makedirs(write_to_dir, exist_ok=True)

    for alloc in allocs:
        for task in alloc.tasks:
            for output_type in OUTPUT_TYPES:
                logs = task.get_logs(nomad_client, output_type)
                if not logs:
                    continue
                filename = f"{task.name}.{output_type}.{alloc.short_alloc_id()}.log"
                with (write_to_dir / filename).open("w") as file:
                    file.write(logs)
