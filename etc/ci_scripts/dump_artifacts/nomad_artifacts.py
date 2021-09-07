#!/usr/bin/env python3

import logging
import shutil
import sys
import nomad
from pathlib import Path

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


nomad_agent_log_path = Path("/tmp/nomad-agent.log").resolve()
consul_agent_log_path = Path("/tmp/consul-agent.log").resolve()

def dump_all_nomad_consul_logs(artifacts_dir: Path) -> None:
    shutil.copy2(nomad_agent_log_path, artifacts_dir)
    shutil.copy2(consul_agent_log_path, artifacts_dir)
