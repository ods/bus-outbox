sqlx-prepare:
    #!/bin/bash
    set -eo pipefail
    docker compose up -d db
    PORT=$(docker compose port db 5432 | cut -d: -f2)
    export DATABASE_URL=postgres://postgres:postgres@localhost:$PORT/bus_outbox
    cargo sqlx migrate run
    cargo sqlx prepare

build:
    docker compose build

test: build
    docker compose run --rm test

psql:
    docker compose up -d db
    docker compose exec db psql -Upostgres bus_outbox
