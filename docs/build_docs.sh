#!/bin/bash

set -euo pipefail
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

cd "${THIS_DIR}/.."
# Create a virtualenv from Pants
./pants export ::
# shellcheck disable=SC1090
source "dist/export/python/virtualenvs/grapl/$(cat .python-version)/bin/activate"

cd "${THIS_DIR}"
pip install wheel
pip install -r requirements.txt

export TARGET_DIR="/tmp/grapl_docs"
rm -rf $TARGET_DIR || true
# -W means warnings become errors
sphinx-build ./ $TARGET_DIR -W
echo "$TARGET_DIR/index.html"
