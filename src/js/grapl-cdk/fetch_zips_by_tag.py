#!/usr/bin/env python3
"""
PSA: The 'tag' verbiage gets a bit conflated once we hit release.
Github Tags = vX.Y.Z
Grapl Versions = vX.Y.Z
"""
import argparse
import json
import os
import re
import subprocess
from typing import Any, Dict, List

Asset = Dict[str, Any]

SUCCESS_EMOJI = "\U00002705"

valid_version = re.compile("v\d+\.\d+\.\d+")

GRAPL_CDK_FOLDER_PATH = os.path.dirname(os.path.realpath(__file__))
assert GRAPL_CDK_FOLDER_PATH.endswith("grapl-cdk")
ZIPS_PATH = os.path.join(GRAPL_CDK_FOLDER_PATH, "zips/")
os.chdir(GRAPL_CDK_FOLDER_PATH)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fetch prebuilt release zips from Github"
    )
    parser.add_argument(
        "--version",
        dest="version",
        required=True,
        help="For example, `v0.1.0` - don't forget the v!",
    )
    return parser.parse_args()


def get_assets_info(version: str) -> List[Asset]:
    """
    Query github for all the Grapl releases for a given tag-version
    """
    raw = subprocess.run(
        [
            "curl",
            "-s",
            "https://api.github.com/repos/grapl-security/grapl/releases/tags/%s"
            % version,
        ],
        capture_output=True,
    ).stdout
    assets: List[Asset] = json.loads(raw)["assets"]
    # Sort with smallest filesize first - due to the cheap way we're doing multi-downloading,
    # this makes the user *percieve* that things are being downloaded, as opposed to staring at
    # "Downloading <50mb file>" for 10 seconds
    assets.sort(key=lambda a: a["size"])
    return assets


def download_asset(asset: Asset, args: argparse.Namespace) -> subprocess.Popen:
    url = asset["browser_download_url"]
    filename = asset["name"]
    filename_on_disk = filename.replace(f"-{args.version}.zip", ".zip")

    print(f"Downloading {url}")
    process = subprocess.Popen(
        [
            "wget",
            "--output-document",
            f"{ZIPS_PATH}{filename_on_disk}",
            url,
        ],
        # Capture instead of printing it; combine into one stream
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return process


def main() -> None:
    args = parse_args()
    if not valid_version.match(args.version):
        raise Exception(f"Tag '{args.version}' invalid - must be of format vX.Y.Z")

    assets = get_assets_info(args.version)

    # Quick benchmark: as of Nov 24 2020, before parallel, 71s; after parallel, 34s
    download_processes: List[subprocess.Popen] = [
        download_asset(a, args) for a in assets
    ]

    # Join them back
    for asset, download in zip(assets, download_processes):
        return_code = download.wait()
        url = asset["browser_download_url"]
        if return_code == 0:
            print(f"{SUCCESS_EMOJI} Downloaded {url}")
        else:
            out = b"".join(list(download.stdout)).decode("utf-8")  # type: ignore
            raise Exception(f"Couldn't download {url}: {out}")


if __name__ == "__main__":
    main()
