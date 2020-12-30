#!/bin/bash
set -eu
THIS_DIR="${PWD}/python_test_deps"

source venv/bin/activate
pip install --no-index --find-links "$THIS_DIR" -r "$THIS_DIR/requirements.txt"
