# grapl_analyzerlib
A library for working with [Grapl](https://github.com/insanitybit/grapl)

[readthedocs](https://grapl-analyzerlib.readthedocs.io/en/latest/)

# Development Setup
```
cd <the same folder as this readme>
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
# reads requirements from setup.py
pip install . ".[dev]"
# Give it a shot!
./run_tests.sh
```