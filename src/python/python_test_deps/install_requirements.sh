#!/bin/bash
set -eu
THIS_DIR="/home/grapl/python_test_deps"

source /home/grapl/venv/bin/activate
pip install --no-index --find-links $THIS_DIR -r $THIS_DIR/requirements.txt
