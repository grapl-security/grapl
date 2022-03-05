#!/usr/bin/env bash

set -euo pipefail

DBNAME=organization-management
USER=postgres
export POSTGRES_PASSWORD=testpassword
CONTAINER_NAME=postgres-organization-management-compile

cleanup() {
    docker kill ${CONTAINER_NAME}
}

echo "Creating docker container:"

docker run \
    --detach \
    --publish 5432:5432 \
    --rm \
    --name ${CONTAINER_NAME} \
    --env POSTGRES_PASSWORD \
    postgres

trap cleanup EXIT INT

echo ""
sleep 5

export DATABASE_URL=postgres://${USER}:${POSTGRES_PASSWORD}@localhost/${DBNAME}

echo "Creating PostGresDB"
sqlx database create

echo "Running migrations"
sqlx migrate run


echo "Saving metadata to sqlx.json for offline mode"
cargo sqlx prepare -- --lib   


