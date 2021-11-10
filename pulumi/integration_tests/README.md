# Grapl Integration Tests pulumi project

Despite the name of this project, it contains both the e2e and the integration
tests. This is because the two concepts will be merged into one in the semi-near
future.

# Initialization

Do the following to set up an Integration Tests stack against a real AWS
sandbox.

```
set -u
STACK_NAME=# <fill in - same name as your grapl repo>
REGION=# <fill in>
E2E_TESTS_TAG=# <fill in - you'll find it in origin/rc branch>

pulumi stack init "grapl/integration-tests/${STACK_NAME}"
pulumi config set aws:region "${REGION}"
pulumi config set nomad:address "http://localhost:4646"
pulumi config set --path "artifacts.e2e-tests" "${E2E_TESTS_TAG}"
```
