psql:
    docker compose up -d db
    docker compose exec db psql -Upostgres bus_outbox
