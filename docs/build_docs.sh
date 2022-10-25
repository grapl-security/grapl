#!/bin/bash

set -euo pipefail
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

cd "${THIS_DIR}/.."
# Create a virtualenv from Pants
./pants export ::

# Since Pants doesn't support 3.10, this won't use the `.python-version`.
PYTHON_VERSION=$(ls "dist/export/python/virtualenv/")
# shellcheck disable=SC1090
source "dist/export/python/virtualenv/${PYTHON_VERSION}/bin/activate"

cd "${THIS_DIR}"
pip install wheel
pip install -r requirements.txt

export TARGET_DIR="/tmp/grapl_docs"
rm -rf $TARGET_DIR || true
# -W means warnings become errors
sphinx-build ./ $TARGET_DIR -W
echo "$TARGET_DIR/index.html"
