#!/usr/bin/env python3
from __future__ import annotations

import dataclasses
import logging
import os
import shutil
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, cast

from nomad import Nomad
from nomad.api.exceptions import URLNotFoundNomadException
from typing_extensions import Literal

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

OutOrErr = Literal["stdout", "stderr"]
OUTPUT_TYPES: list[OutOrErr] = ["stdout", "stderr"]

nomad_agent_log_path = Path("/tmp/nomad-agent.log").resolve()
consul_agent_log_path = Path("/tmp/consul-agent.log").resolve()
vault_agent_log_path = Path("/tmp/vault-agent.log").resolve()


@dataclasses.dataclass
class NomadDumpOptions:
    # Dump `nomad-agent` logs
    dump_agent_logs: bool
    # Dump the Consul Connect proxy logs for each service
    dump_connect_proxy_logs: bool


def _get_nomad_client(namespace: str | None = None) -> Nomad:
    address = os.getenv("NOMAD_ADDRESS") or "http://localhost:4646"
    assert address.startswith("http"), f"Your nomad address needs a protocol: {address}"
    nomad_client = Nomad(address=address, timeout=10, namespace=namespace)
    return nomad_client


def dump_all(
    artifacts_dir: Path,
    opts: NomadDumpOptions,
) -> None:
    if opts.dump_agent_logs:
        _dump_hashicorp_agent_logs(artifacts_dir)

    # Get every namespace.
    nomad_client = _get_nomad_client()
    namespaces: list[NomadNamespace] = [
        NomadNamespace(ns) for ns in nomad_client.namespaces
    ]

    # Dump every namespace.
    # The "default" namespace is special-cased to get dumped in the main directory.
    for namespace in namespaces:
        ns = namespace.name
        ns_nomad_client = _get_nomad_client(namespace=ns)
        ns_dir = artifacts_dir if ns == "default" else artifacts_dir / "namespaces" / ns

        allocations = _get_allocations(ns_nomad_client, parent=namespace, opts=opts)

        _get_nomad_logs_for_each_service(ns_dir, ns_nomad_client, allocations)


def _dump_hashicorp_agent_logs(artifacts_dir: Path) -> None:
    hashicorp_log_paths = [
        nomad_agent_log_path,
        consul_agent_log_path,
        vault_agent_log_path,
    ]
    for log_path in hashicorp_log_paths:
        if os.path.exists(log_path):
            shutil.copy2(log_path, artifacts_dir)


@dataclasses.dataclass
class NomadNamespace:
    name: str

    def __init__(self, input: dict[str, Any]) -> None:
        self.name = input["Name"]


@dataclasses.dataclass
class NomadAllocation:
    parent: NomadNamespace
    allocation_id: str
    allocation_name: str
    status: str
    tasks: list[NomadTask]

    def __init__(
        self,
        input: dict[str, Any],
        parent: NomadNamespace,
        opts: NomadDumpOptions,
    ) -> None:
        self.parent = parent
        self.allocation_id = input["ID"]
        self.allocation_name = input["Name"]
        self.status = input["ClientStatus"]
        if not input["TaskStates"]:
            raise Exception(f"Why are there no TaskStates? {input}")

        task_names = [t for t in input["TaskStates"].keys()]
        # Optionally filter out task names we don't care about
        if not opts.dump_connect_proxy_logs:
            task_names = [t for t in task_names if not t.startswith("connect-proxy")]

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


@dataclasses.dataclass(eq=False)
class NomadTask:
    parent: NomadAllocation = dataclasses.field(repr=False)
    name: str
    events: list[dict]
    state: str
    restarts: int

    def get_logs(self, nomad_client: Nomad, type: OutOrErr) -> str | None:
        try:
            return cast(
                str,
                nomad_client.client.stream_logs.stream(
                    self.parent.allocation_id, self.name, type=type, plain=True
                ),
            )
        except URLNotFoundNomadException as e:
            LOGGER.warn(
                f"No logs for task '{self.name}' in namespace '{self.parent.parent.name}'"
            )
            return None

    def get_events(self) -> str:
        if self.parent.status not in ["running", "completed"]:
            event_list = [event["DisplayMessage"] for event in self.events]
            return "\n".join(event_list)
        return ""


JobToAllocDict = dict[str, list[NomadAllocation]]


def _get_allocations(
    nomad_client: Nomad,
    parent: NomadNamespace,
    opts: NomadDumpOptions,
) -> JobToAllocDict:
    job_names = _get_nomad_job_names(nomad_client)
    job_to_allocs: JobToAllocDict = {
        job_name: [
            NomadAllocation(a, parent=parent, opts=opts)
            for a in nomad_client.job.get_allocations(job_name)
        ]
        for job_name in job_names
    }
    return job_to_allocs


def _get_nomad_job_names(nomad_client: Nomad) -> list[str]:
    # Filter out the Parameterized Batch jobs, because those don't themselves have logs;
    # the dispatched instances of them have jobs.
    # Otherwise you'd see both of these:
    # - integration_tests (param batch job) (no logs since nothing ran)
    # - integration_tests/dispatch-12345 (a dispatched instance of integration_tests)

    jobs: list[str] = [j["ID"] for j in nomad_client.jobs if not j["ParameterizedJob"]]
    return jobs


def _get_nomad_logs_for_each_service(
    artifacts_dir: Path,
    nomad_client: Nomad,
    job_to_allocs: JobToAllocDict,
) -> dict[str, list[NomadAllocation]]:
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
    allocs: list[NomadAllocation],
) -> None:
    write_to_dir = artifacts_dir / job_name
    os.makedirs(write_to_dir, exist_ok=True)

    _write_allocation_task_statuses(job_dir=write_to_dir, allocs=allocs)

    # Memoize which allocs a Task occurs in.
    # tasks[task_name] => {task_one, task_two}
    task_allocs: dict[str, set[NomadTask]] = defaultdict(set)

    for alloc in allocs:
        for task in alloc.tasks:
            task_allocs[task.name].add(task)

    for (task_name, task_alloc_set) in task_allocs.items():
        # Skip the `.short_alloc` stuff if there's just one.
        is_singular = len(task_alloc_set) == 1

        for task in task_alloc_set:
            suffix = "" if is_singular else f".{task.parent.short_alloc_id}"
            # publish task events
            events = task.get_events()
            if events:
                filename = f"{task_name}.events{suffix}.log"
                with (write_to_dir / filename).open("w") as file:
                    file.write(events)

            # publish logs
            for output_type in OUTPUT_TYPES:
                logs = task.get_logs(nomad_client, output_type)
                if not logs:
                    continue
                filename = f"{task.name}.{output_type}{suffix}.log"
                with (write_to_dir / filename).open("w") as file:
                    file.write(logs)


def _write_allocation_task_statuses(
    job_dir: Path,
    allocs: list[NomadAllocation],
) -> None:
    statuses = "\n".join(
        sorted(
            f"{t.name}: state = {t.state}, restarts = {t.restarts}"
            for alloc in allocs
            for t in alloc.tasks
        )
    )
    with (job_dir / f"task_statuses.txt").open("w") as file:
        file.write(statuses)
