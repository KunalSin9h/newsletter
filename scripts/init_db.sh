#!/usr/bin/env bash

set -eox pipefail

# Check if the psql client is install on the system
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi

# Check if the sqlx-cli is install on the system
if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 'Error: sqlx is not installed.'
    echo >&2 'Use:'
    echo >&2 '    cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres'
    echo >&2 'to install it.'
    exit 1
fi

# Check if the custom user has been set, otherwise default to 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"

# Check if the custom password has been set, otherwise default to 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"

# Check if a custom post has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

# Launch postgres using docker if not running
if [[ -z "${SKIP_DOCKER}" ]]; then
docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres:15.2-alpine3.17  \
    postgres -N 1000
    # ^^^^^^^^^^^^^ Increased maximum number of connections for testing prupos
fi

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "127.0.0.1" -U  "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c "\q"; do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 2
done

>&2 echo "Postgres is up and running on port ${DB_PORT}"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@127.0.0.1:${DB_PORT}/${DB_NAME}
sqlx database create
# sqlx migrate add create_subscription_table -- On the shell to create migration file
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
