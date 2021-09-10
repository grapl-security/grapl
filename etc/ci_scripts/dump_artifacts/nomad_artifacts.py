#!/usr/bin/env python3
from __future__ import annotations

import dataclasses
import logging
import os
import shutil
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Union, cast

from nomad import Nomad
from nomad.api.exceptions import URLNotFoundNomadException
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

    def get_logs(self, nomad_client: Nomad, type: OutOrErr) -> Optional[str]:
        try:
            return cast(
                str,
                nomad_client.client.stream_logs.stream(
                    self.parent.allocation_id, self.name, type=type, plain=True
                ),
            )
        except URLNotFoundNomadException as e:
            LOGGER.info(f"Couldn't get logs for {self.name}")
            return None


def _get_nomad_job_names(nomad_client: Nomad) -> List[str]:
    # Filter out the Parameterized Batch jobs, because those don't themselves have logs;
    # the dispatched instances of them have jobs.
    # Otherwise you'd see both of these:
    # - integration_tests (param batch job) (no logs since nothing ran)
    # - integration_tests/dispatch-12345 (a dispatched instance of integration_tests)

    jobs: List[str] = [j["ID"] for j in nomad_client.jobs if not j["ParameterizedJob"]]
    return jobs


def _get_nomad_logs_for_each_service(
    artifacts_dir: Path,
) -> Dict[str, List[NomadAllocation]]:
    nomad_client = Nomad(host="localhost", timeout=5)
    job_names = _get_nomad_job_names(nomad_client)
    job_to_allocs: Dict[str, List[NomadAllocation]] = {
        job_name: [
            NomadAllocation(a) for a in nomad_client.job.get_allocations(job_name)
        ]
        for job_name in job_names
    }

    for job, allocs in job_to_allocs.items():
        _write_nomad_logs(nomad_client, artifacts_dir, job_name=job, allocs=allocs)

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
