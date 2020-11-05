"""
Mostly copied from etc/local_grapl/bin/upload_plugin.py
"""
import logging
import os
import sys
from http import HTTPStatus
from pathlib import Path
from typing import Dict

import requests

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}


class ModelPluginDeployerException(Exception):
    pass


class ModelPluginDeployerClient:
    def __init__(self, use_docker_links: bool = False) -> None:
        hostname = "grapl-model-plugin-deployer" if use_docker_links else "localhost"
        self.endpoint = f"http://{hostname}:8123"

    def deploy(
        self,
        plugins_folder: Path,
        jwt: str,
    ) -> requests.Response:
        paths = []
        for subdir, _dirs, files in os.walk(plugins_folder):
            for file in files:
                if not self._is_valid_deployable_file(file):
                    continue
                paths.append(os.path.abspath(os.path.join(subdir, file)))

        assert len(paths), "expected at least one file to be uploaded"

        plugin_dict: Dict[str, str] = {}
        for path in paths:
            with open(path, "r") as f:
                clean_path = str(path).split("model_plugins/")[-1]
                contents = f.read()
                if len(contents) == 0:
                    if clean_path.endswith("__init__.py"):
                        # attempted hack: make __init__.py non empty
                        # due to https://github.com/boto/botocore/pull/1328
                        # otherwise the client just hangs
                        contents = "# NON_EMPTY_INIT_PY_HACK"
                    else:
                        logging.debug(f"Empty non-init file, skipping it: {clean_path}")
                        continue

                plugin_dict[clean_path] = contents

        resp = requests.post(
            f"{self.endpoint}/deploy",
            json={"plugins": plugin_dict},
            headers=_JSON_CONTENT_TYPE_HEADERS,
            cookies={"grapl_jwt": jwt},
        )
        if resp.status_code != HTTPStatus.OK:
            raise ModelPluginDeployerException(f"{resp.status_code}: {resp.text}")
        return resp

    def _is_valid_deployable_file(self, file_path: str) -> bool:
        """
        not exactly the most bulletproof heuristic in the world, feel free to modify it
        """
        if file_path.endswith(".pyc"):
            return False
        if file_path.endswith(".ipynb"):
            return False
        return True
