python_library(
    skip_mypy=True
)

python_awslambda(
    name="lambda",
    runtime="python3.7",
    handler="grapl_dgraph_ttl.py:prune_expired_subgraphs"
)
