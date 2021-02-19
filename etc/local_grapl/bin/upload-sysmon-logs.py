#!/usr/bin/env python3
"""
Despite the path, this is *not* tied just to Local Grapl, and can also be used on true S3 buckets.
"""

import argparse
import sys
import os
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


def parse_args():
    parser = argparse.ArgumentParser(description="Send sysmon logs to Grapl")
    parser.add_argument("--bucket_prefix", dest="bucket_prefix", required=True)
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
    if args.bucket_prefix is None:
        raise Exception("Provide bucket prefix as first argument")

    upload_fn = hack_PATH_to_include_grapl_tests_common()
    upload_fn(
        args.bucket_prefix,
        args.logfile,
        delay=args.delay,
        batch_size=args.batch_size,
    )
