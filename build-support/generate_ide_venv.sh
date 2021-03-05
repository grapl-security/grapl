#!/usr/bin/env bash

set -euo pipefail

# Install all the python libraries and their dependencies in a single
# virtualenv build-support/.ide-venv for IDEs to use when developing
# Grapl python code. This script calls out to
# build-support/generate_constraints.sh. You should execute this
# script periodically to ensure your IDE virtualenv is fresh.

# NOTE: This script implicitly assumes it is being run from the
# top-level of the repository!

./build-support/generate_constraints.sh

python=python3
# This directory should be .gitignored
IDE_VIRTUALENV=build-support/.ide-venv
pip="${IDE_VIRTUALENV}/bin/pip"
constraints_file=3rdparty/python/constraints.txt

rm -Rf "${IDE_VIRTUALENV}"
"${python}" -m venv "${IDE_VIRTUALENV}"

source "${IDE_VIRTUALENV}/bin/activate"

"${pip}" install pip --upgrade
"${pip}" install \
  --requirement "${constraints_file}"

deactivate
