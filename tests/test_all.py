import time
import psycopg2


def insert_message(
    db: psycopg2.extensions.cursor,
    topic: str,
    *,
    payload: bytes | None = None,
    key: bytes | None = None,
    headers: dict[str, str| None] | None = None,
) -> None:
    db.execute(
        "INSERT INTO bus_outbox_messages (topic, payload, key, headers) "
        "VALUES (%s, %s, %s, %s)",
        (topic, payload, key, headers),
    )

def wait_queue_is_empty(db: psycopg2.extensions.cursor, timeout: float = 3.0) -> None:
    started = time.monotonic()
    while time.monotonic() - started < timeout:
        db.execute("SELECT COUNT(*) FROM bus_outbox_messages")
        count = db.fetchone()[0]
        if count == 0:
            return
    else:
        raise TimeoutError(f"Queue is not processed in {timeout=} seconds")


def test_send(
    db: psycopg2.extensions.cursor,
    bus_topic: str,
) -> None:
    # Given
    for _ in range(100):
        insert_message(db, bus_topic, payload=b"test")

    # Then
    wait_queue_is_empty(db)
