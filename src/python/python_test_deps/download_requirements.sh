#!/bin/bash
set -eu
THIS_DIR="/home/grapl/python_test_deps"
cd /home/grapl
source venv/bin/activate 

cd $THIS_DIR
pip download -r requirements.txt
