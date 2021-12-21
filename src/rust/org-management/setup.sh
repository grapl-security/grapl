# The following script is used for debugging purposes only.
# We create a container with a postgres instance with migrations and opens up an interactive psql terminal

#!/usr/bin/env bash

set -euo pipefail

DBNAME=org-management
USER=postgres
export POSTGRES_PASSWORD=foobarmonkeys
CONTAINER_NAME=postgres-org-management-setup

echo "Creating docker container:"

docker run \
    --detach \
    --publish 5432:5432 \
    --rm \
    --name ${CONTAINER_NAME} \
    --env POSTGRES_PASSWORD \
    postgres


echo ""
sleep 5

export DATABASE_URL=postgres://${USER}:${POSTGRES_PASSWORD}@localhost/${DBNAME}
export ORG_MANAGEMENT_PORT=5432

echo "Creating PostGresDB"
sqlx database create

echo "Running migrations"
sqlx migrate run

echo "Entering docker container"
docker exec \
  --interactive \
  --tty ${CONTAINER_NAME} \
  psql --username postgres --dbname ${DBNAME}


