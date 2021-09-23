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

    nomad_client = Nomad(host="localhost", timeout=10)
    allocations = _get_allocations(nomad_client)

    _dump_nomad_consul_agent_logs(nomad_artifacts_dir)
    _get_nomad_logs_for_each_service(nomad_artifacts_dir, nomad_client, allocations)


def _dump_nomad_consul_agent_logs(artifacts_dir: Path) -> None:
    shutil.copy2(nomad_agent_log_path, artifacts_dir)
    shutil.copy2(consul_agent_log_path, artifacts_dir)


@dataclasses.dataclass
class NomadAllocation:
    allocation_id: str
    allocation_name: str
    status: str
    tasks: List[NomadTask]

    def __init__(self, input: Dict[str, Any]) -> None:
        self.allocation_id = input["ID"]
        self.allocation_name = input["Name"]
        self.status = input["ClientStatus"]
        if not input["TaskStates"]:
            raise Exception(f"Why are there no TaskStates? {input}")
        # Remove tasks we don't super care about
        task_names = [
            t for t in input["TaskStates"].keys() if not t.startswith("connect-proxy")
        ]
        self.tasks = [
            NomadTask(
                parent=self,
                name=t,
                events=input["TaskStates"][t]["Events"],
                state=input["TaskStates"][t]["State"],
                restarts=input["TaskStates"][t]["Restarts"],
            )
            for t in task_names
        ]

    @property
    def short_alloc_id(self) -> str:
        return self.allocation_id.split("-")[0]


@dataclasses.dataclass
class NomadTask:
    name: str
    parent: NomadAllocation = dataclasses.field(repr=False)
    events: List[dict]
    state: str
    restarts: int

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

    def get_events(self) -> str:
        if self.parent.status not in ["running", "completed"]:
            event_list = [event["DisplayMessage"] for event in self.events]
            return "\n".join(event_list)
        return ""


JobToAllocDict = Dict[str, List[NomadAllocation]]


def _get_allocations(nomad_client: Nomad) -> JobToAllocDict:
    job_names = _get_nomad_job_names(nomad_client)
    job_to_allocs: JobToAllocDict = {
        job_name: [
            NomadAllocation(a) for a in nomad_client.job.get_allocations(job_name)
        ]
        for job_name in job_names
    }
    return job_to_allocs


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
    nomad_client: Nomad,
    job_to_allocs: JobToAllocDict,
) -> Dict[str, List[NomadAllocation]]:
    for job, allocs in job_to_allocs.items():
        # Dispatch job names look like `integration-tests/dispatch-1632277984-ad265cfe`
        # the second part is largely useless for us.
        simplified_job_name = job.split("/")[0]
        _write_nomad_logs(
            nomad_client, artifacts_dir, job_name=simplified_job_name, allocs=allocs
        )

    return job_to_allocs


def _write_nomad_logs(
    nomad_client: Nomad,
    artifacts_dir: Path,
    job_name: str,
    allocs: List[NomadAllocation],
) -> None:
    write_to_dir = artifacts_dir / job_name
    os.makedirs(write_to_dir, exist_ok=True)

    _write_allocation_task_statuses(job_dir=write_to_dir, allocs=allocs)
    for alloc in allocs:
        for task in alloc.tasks:
            # publish task events
            events = task.get_events()
            if events:
                filename = f"{task.name}.events.{alloc.short_alloc_id}.log"
                with (write_to_dir / filename).open("w") as file:
                    file.write(events)

            # publish logs
            for output_type in OUTPUT_TYPES:
                logs = task.get_logs(nomad_client, output_type)
                if not logs:
                    continue
                filename = f"{task.name}.{output_type}.{alloc.short_alloc_id}.log"
                with (write_to_dir / filename).open("w") as file:
                    file.write(logs)


def _write_allocation_task_statuses(
    job_dir: Path,
    allocs: List[NomadAllocation],
) -> None:
    statuses = "\n".join(
        sorted([
            f"{t.name}: state = {t.state}, restarts = {t.restarts}" 
            for alloc in allocs
            for t in alloc.tasks
        ])
    )
    with (job_dir / f"task_statuses.txt").open("w") as file:
        file.write(statuses)
