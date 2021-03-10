from contextlib import contextmanager
from typing import ContextManager
from unittest.mock import Mock, patch

from click.testing import CliRunner
from graplctl import __version__


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
        # that we patch out
        from graplctl import cli

        _return_fake_queues(mock_session)
        result = runner.invoke(cli.main, [*DEFAULT_ARGS, "queues", "ls"])
    assert (
        result.output
        == "http://queue1-dead-letter-queue\nhttp://queue2-dead-letter-queue\n"
    )
    assert result.exit_code == 0


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
    from graplctl import cli

    with patch.object(cli, cli.os.__name__):
        with patch.object(
            cli,
            "SESSION",
        ) as p_session:
            yield p_session
