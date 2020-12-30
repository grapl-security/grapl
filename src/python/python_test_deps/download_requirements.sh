#!/bin/bash
set -eu
THIS_DIR="${PWD}/python_test_deps"
source venv/bin/activate 

cd "$THIS_DIR"
pip download -r requirements.txt
