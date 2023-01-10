#!/usr/bin/env bash
set -x
set -eo pipefail

DB_HOST="${POSTGRES_HOST:=localhost}"
# Default port is 54321 so that the dockerized postgres doesn't interfere with locally
# installed postgres.
DB_PORT="${POSTGRES_PORT:=54321}"
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=alloxid}"

# Allow to skip spinning up a docker container if one is already running.
if [[ -z "${SKIP_DOCKER}" ]]
then
docker run \
    --name alloxid_db \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000

# Keep pinging Postgres until it's ready to accept commands.
until PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -c '\conninfo'; do
    >&2 echo "Postgres is still unavailable - sleeping"
    # Limit retries to 5 so we don't loop indefinitely in CI.
    ((c++)) && ((c==5)) && break
    sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}."
fi


export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

sqlx database create
sqlx migrate run --source alloxid-http/migrations

>&2 echo "Postgres has been migrated, ready to go!"

