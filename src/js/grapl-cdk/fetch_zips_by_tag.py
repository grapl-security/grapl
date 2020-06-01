#!/usr/bin/env python3
import sys
import json
import subprocess
import os
import re

valid_tag = re.compile("v\d+\.\d+\.\d+")

try:
    tag = sys.argv[1]
except IndexError:
    print("Tag must be specified")
    sys.exit(1)

if not valid_tag.match(tag):
    print("Tag invalid - must be of format v\d+\.\d+\.\d+")
    sys.exit(1)
try:
    stable_or_beta = sys.argv[2]
except IndexError:
    stable_or_beta = "latest"

if stable_or_beta not in ("latest", "beta"):
    print("Version invalid, mut be latest or beta")
    sys.exit(1)

raw = subprocess.run(
    [
        "curl",
        "-s",
        "https://api.github.com/repos/grapl-security/grapl/releases/tags/%s" % tag,
    ],
    capture_output=True,
).stdout
assets = json.loads(raw)["assets"]

for a in assets:
    url = a["browser_download_url"]
    filename = a["name"]
    linkname = filename.replace(f"-{tag}-{stable_or_beta}.zip", ".zip")

    pwd = os.path.abspath(".")
    zips = os.path.join(pwd, "zips/")

    print(url)
    subprocess.run(["wget", "-P", "zips", url])
    subprocess.run(["unlink", zips + linkname])
    subprocess.run(["ln", "-s", filename, zips + linkname])
