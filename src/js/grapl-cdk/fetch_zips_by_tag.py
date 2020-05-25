#!/usr/bin/env python3
import sys
import json
import subprocess

tag = "v0.0.16"
raw = subprocess.run(["curl", "-s", "https://api.github.com/repos/grapl-security/grapl/releases/tags/%s" % tag], capture_output=True).stdout
assets = json.loads(raw)['assets']

for a in assets:
    url = a['browser_download_url']
    filename = a['name']
    linkname = filename.replace('-%s-latest.zip' % tag, '.zip')

    print(url)
    subprocess.run(["wget","-P", "zips",  url])
    subprocess.run(["unlink", "zips/%s" % linkname])
    subprocess.run(["ln", "-s", filename, "zips/%s" % linkname])
