from collections.abc import Callable, Generator
from contextlib import suppress
import os
import subprocess
import sys
from time import sleep
from typing import Protocol
from uuid import uuid4

import psycopg2
import pytest
from _pytest.terminal import TerminalReporter
from _pytest._code.code import ReprExceptionInfo, ExceptionChainRepr
from confluent_kafka.admin import AdminClient, NewTopic


###############################################################################
# Trick to force pytest to show "native" traceback on KeyboardInterrupt
#
# By default doesn't show traceback on KeyboardInterrupt, unless `--full-trace`
# option is used.  And with `--full-trace` option the traceback is effectively
# unreadable.

_keyboardinterrupt_memo: ReprExceptionInfo | ExceptionChainRepr | None = None


def pytest_keyboard_interrupt(excinfo: pytest.ExceptionInfo[BaseException]) -> None:
    global _keyboardinterrupt_memo
    _keyboardinterrupt_memo = excinfo.getrepr(style="native")


def pytest_terminal_summary(
    terminalreporter: TerminalReporter,
    exitstatus: pytest.ExitCode,
    config: pytest.Config,
) -> None:
    if (
        exitstatus == pytest.ExitCode.INTERRUPTED
        and not config.option.fulltrace
    ):
        excrepr = _keyboardinterrupt_memo
        assert excrepr is not None
        assert excrepr.reprcrash is not None
        msg = excrepr.reprcrash.message
        terminalreporter.write_sep("!", msg)
        if "KeyboardInterrupt" in msg:
            excrepr.toterminal(terminalreporter._tw)

# End of trick
###############################################################################


@pytest.fixture(scope='session')
def migrate() -> None:
    subprocess.check_call(["/bus-outbox", "migrate"], timeout=1.0)


@pytest.fixture
def db(migrate: None) -> psycopg2.extensions.cursor:
    conn = psycopg2.connect(os.getenv("DB_DSN"))
    conn.autocommit = True

    cursor = conn.cursor()
    cursor.execute("TRUNCATE TABLE bus_outbox_messages")
    return cursor


@pytest.fixture(scope='session')
def bus_conf() -> dict[str, str | None]:
    return {
        "bootstrap.servers": os.getenv("BOOTSTRAP_SERVERS"),
    }


@pytest.fixture
def bus_admin(bus_conf: dict[str, str | None]) -> AdminClient:
    return AdminClient(bus_conf)


@pytest.fixture
def bus_topic(bus_admin: AdminClient) -> Generator[str, None, None]:
    topic = uuid4().hex
    bus_admin.create_topics([NewTopic(topic, num_partitions=1, replication_factor=1)])
    yield topic

    bus_admin.delete_topics([topic])


class InsertMessage(Protocol):
    def __call__(
        self,
        *,
        payload: str | None = None,
        key: str | None = None,
        headers: dict[str, str | None] | None = None,
    ) -> None:
        ...


@pytest.fixture
def insert_message(db: psycopg2.extensions.cursor, bus_topic: str) -> InsertMessage:
    def fixture_value(
        *,
        payload: str | None = None,
        key: str | None = None,
        headers: dict[str, str| None] | None = None,
    ) -> None:
        db.execute(
            "INSERT INTO bus_outbox_messages (topic, payload, key, headers) "
            "VALUES (%s, %s, %s, %s)",
            (bus_topic, payload, key, headers),
        )

    return fixture_value


@pytest.fixture(autouse=True)
def _produce(db: psycopg2.extensions.connection) -> Generator[None, None, None]:
    process = subprocess.Popen(["/bus-outbox", "produce"])
    try:
        # Give some time to check args and connect to database and kafka
        return_code = process.wait(timeout=0.01)
    except subprocess.TimeoutExpired:
        pass
    else:
        raise subprocess.CalledProcessError(return_code, process.args)
    yield

    return_code = process.poll()
    if return_code is not None:
        raise subprocess.CalledProcessError(return_code, process.args)

    process.terminate()
    process.wait(timeout=0.01)


@pytest.fixture
def bus_consumer(bus_conf: dict[str, str | None], topic: str) -> Generator[None, None, None]:
    conf = {
        **bus_conf,
        "auto.offset.reset": "earliest",
    }
    pass  # TODO
