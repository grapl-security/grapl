from contextlib import contextmanager
from typing import ContextManager
from unittest.mock import Mock, patch

from click.testing import CliRunner
from graplctl import __version__, cli


def test_version():
    assert __version__ == "0.1.0"


DEFAULT_ARGS = [
    "--grapl-region",
    "us-west-2",
    "--grapl-deployment-name",
    "fake-deployment",
    "--grapl-version",
    "fake-version",
]


def test_queues_ls() -> None:
    runner = CliRunner()
    with _patch_boto3_session() as mock_session:
        # Importing here, because the import causes an `os.getenv`
        # that we need to patch out. See the TODO above `SESSION =`
        from graplctl import cli

        _return_fake_queues(mock_session)
        result = runner.invoke(cli.main, [*DEFAULT_ARGS, "queues", "ls"])
    assert result.exit_code == 0
    assert (
        result.output
        == "http://queue1-dead-letter-queue\nhttp://queue2-dead-letter-queue\n"
    )


# TODO: Add a `test_queues_redrive`


def _return_fake_queues(mock_session: Mock) -> None:
    sqs_client = mock_session.client.return_value
    sqs_client.list_queues.return_value = {
        "QueueUrls": [
            "http://queue1",
            "http://queue1-retry-queue",
            "http://queue1-dead-letter-queue",
            "http://queue2",
            "http://queue2-retry-queue",
            "http://queue2-dead-letter-queue",
        ]
    }


@contextmanager
def _patch_boto3_session() -> ContextManager[Mock]:
    # Patches primarily due to legacy constants at the top
    # of cli.py
    orig_session = cli.boto3.Session
    with patch.object(
        cli.boto3,
        cli.boto3.Session.__name__,
    ) as p_session_cls:
        session = Mock(name="Session instance")
        p_session_cls.return_value = session
        yield session
