from typing import ContextManager
from unittest.mock import Mock, patch

from click.testing import CliRunner
from graplctl import __version__, cli


def test_version():
    assert __version__ == "0.1.0"


def test_queues_ls() -> None:
    runner = CliRunner()
    with _patch_boto3_session() as mock_session:
        _return_fake_queues(mock_session)
        result = runner.invoke(cli.main, ["queues", "ls"])
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


def _patch_boto3_session() -> ContextManager[Mock]:
    return patch.object(
        cli,
        "SESSION",
    )
