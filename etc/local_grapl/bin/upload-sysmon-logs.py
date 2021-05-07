#!/usr/bin/env python3
"""
Despite the path, this is *not* tied just to Local Grapl, and can also be used on true S3 buckets.

TODO: This file is on the verge of being deprecated by `graplctl upload sysmon`.
Just need to add support for local-grapl.
https://github.com/grapl-security/issue-tracker/issues/393
"""

import argparse
import os
import sys
from pathlib import Path
from typing import Callable


def hack_PATH_to_include_grapl_tests_common() -> Callable:
    """
    Requirements:
    - this script should be runnable from command line without Docker.
    - the logic should be exposed as a library that's callable from grapl-tests-common.
    It was either this, or forcing a `pip install .` from python code. Gross.
    """
    this_file = Path(__file__).resolve()
    # go to `grapl` base dir
    grapl_repo_root = this_file
    while grapl_repo_root.name != "grapl":
        grapl_repo_root = grapl_repo_root.parent

    for additional_path in (
            "src/python/grapl-tests-common",
            "src/python/grapl-common",
    ):
        additional_fullpath = grapl_repo_root.joinpath(additional_path)
        # Look at the inserted lib before system-installed one
        sys.path.insert(0, str(additional_fullpath))

    from grapl_tests_common.upload_logs import upload_sysmon_logs

    return upload_sysmon_logs


def setup_env(deployment_name: str):
    """Ensures the environment is set up appropriately for interacting
    with Local Grapl (running inside a Docker Compose network locally)
    from *outside* that network (i.e., from your workstation).

    """
    # NOTE: These values are copied from local-grapl.env. It's
    # unfortunate, yes, but in the interests of a decent
    # user-experience, we'll eat that pain for now. In the near term,
    # we should pull this functionality into something like graplctl
    # with a more formalized way of pointing to a specific Grapl
    # instance.
    if deployment_name == "local-grapl":
        kvs = [
            ("AWS_REGION", "us-east-1"),
            ("S3_ENDPOINT", "http://localhost:4566"),
            ("S3_ACCESS_KEY_ID", "test"),
            ("S3_ACCESS_KEY_SECRET", "test"),
            ("SQS_ENDPOINT", "http://localhost:4566"),
            ("SQS_ACCESS_KEY_ID", "test"),
            ("SQS_ACCESS_KEY_SECRET", "test"),
        ]
        for (k, v) in kvs:
            # fun fact: os.putenv is bad and this is preferred
            os.environ[k] = v


def parse_args():
    parser = argparse.ArgumentParser(description="Send sysmon logs to Grapl")
    parser.add_argument("--deployment_name", dest="deployment_name", required=True)
    parser.add_argument(
        "--logfile",
        dest="logfile",
        required=True,
        help="ie $GRAPLROOT/etc/sample_data/eventlog.xml",
    )
    parser.add_argument("--delay", dest="delay", default=0, type=int)
    parser.add_argument("--batch-size", dest="batch_size", default=100, type=int)
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    if args.deployment_name is None:
        raise Exception("Provide deployment name as first argument")

    setup_env(args.deployment_name)
    upload_fn = hack_PATH_to_include_grapl_tests_common()
    upload_fn(
        args.deployment_name,
        args.logfile,
        delay=args.delay,
        batch_size=args.batch_size,
    )
