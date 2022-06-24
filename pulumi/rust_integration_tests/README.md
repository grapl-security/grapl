# Grapl Integration Tests pulumi project

This project contains the integration tests.

# Initialization

Do the following to set up an Integration Tests stack against a real AWS
sandbox.

```
set -u
STACK_NAME=# <fill in - same name as your grapl repo>
REGION=# <fill in>

pulumi stack init "grapl/integration-tests/${STACK_NAME}"
pulumi config set aws:region "${REGION}"
pulumi config set nomad:address "http://localhost:4646"
```
