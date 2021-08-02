#!/bin/bash

set -euo pipefail

python3 -m venv venv
source venv/bin/activate
pip install wheel
pip install -r requirements.txt

export TARGET_DIR="/tmp/grapl_docs"
rm -r $TARGET_DIR
# -W means warnings become errors
sphinx-build ./ $TARGET_DIR -W
echo "$TARGET_DIR/index.html"
