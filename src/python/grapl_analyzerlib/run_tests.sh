rm -r ./mut_grapl_analyzerlib/ 2> /dev/null;

cp -r ./grapl_analyzerlib/ ./mut_grapl_analyzerlib/;

MG_ALPHAS="localhost:9080"
python3 ./grapl_provision.py
time pytest --workers 8 --cov-report term-missing --cov=./grapl_analyzerlib/nodes/  --cov-branch  ./tests/ $@

# time mutmut run;
