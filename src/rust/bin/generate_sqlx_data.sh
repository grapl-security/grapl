#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Our Rust code uses a library called `sqlx` to interface with Postgres.
# It *statically* checks the validity of queries by running them against a real,
# migrated SQL database and storing that validity in sqlx-data.json.
#
# Long story short: Changing Rust sql queries? You probably want to run this
# script with `make generate-sqlx-data`.
################################################################################
REPOSITORY_ROOT="$(git rev-parse --show-toplevel)"
readonly REPOSITORY_ROOT
readonly PORT=5432
readonly DB_URL="postgres://postgres@localhost:${PORT}"
readonly CONTAINER_NAME="postgres-for-sqlx-prepare"

start_postgres() {
    echo -e "\n--- Running Postgres as ${CONTAINER_NAME}"
    docker run \
        --shm-size=512m \
        --rm \
        --publish "${PORT}:${PORT}" \
        --env POSTGRES_HOST_AUTH_METHOD=trust \
        --env POSTGRES_USER=postgres \
        --detach \
        --name "${CONTAINER_NAME}" \
        postgres-ext:dev

    # Wait for Postgres port to become available
    local -r ip_address="$(sudo docker inspect --format='{{.NetworkSettings.IPAddress}}' $CONTAINER_NAME)"
    until nc -z "${ip_address}" ${PORT}; do
        echo "...waiting for postgres to start..."
        sleep 1
    done
}

stop_postgres() {
    echo -e "\n--- Stopping Postgres"
    docker rm --force "${CONTAINER_NAME}" || true
}

sqlx_prepare() {
    local -r which_rust_lib="${1}"

    echo -e "\n--- Sqlx Prepare on ${which_rust_lib}"
    start_postgres
    (
        cd "${which_rust_lib}"
        DATABASE_URL="${DB_URL}" cargo sqlx migrate run
        DATABASE_URL="${DB_URL}" cargo sqlx prepare -- --lib --all-features
    )
    stop_postgres
}

# Stop the container if any failures occur.
trap stop_postgres EXIT

sqlx_prepare "${REPOSITORY_ROOT}/src/rust/graph-schema-manager"
sqlx_prepare "${REPOSITORY_ROOT}/src/rust/event-source"
sqlx_prepare "${REPOSITORY_ROOT}/src/rust/plugin-work-queue"
sqlx_prepare "${REPOSITORY_ROOT}/src/rust/plugin-registry"
sqlx_prepare "${REPOSITORY_ROOT}/src/rust/uid-allocator"

# Undo the above trap
trap - EXIT
