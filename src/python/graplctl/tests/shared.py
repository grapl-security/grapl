from contextlib import contextmanager
from typing import ContextManager, Sequence
from unittest.mock import ANY, Mock, patch

from click.testing import CliRunner, Result
from graplctl import cli

DEFAULT_ARGS = [
    "--grapl-region",
    "us-west-2",
    "--grapl-deployment-name",
    "fake-deployment",
    "--grapl-version",
    "fake-version",
]


class BotoSessionMock:
    """ Add any useful mutations to the session mock to this class. """

    def __init__(self, session: Mock) -> None:
        self.session = session

    def return_fake_queues(self) -> None:
        sqs_client = self.session.client.return_value
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
def patch_boto3_session() -> ContextManager[BotoSessionMock]:
    with patch.object(
        cli.boto3, cli.boto3.Session.__name__, spec_set=cli.boto3.Session
    ) as p_session_cls:
        session = Mock(name="Session instance")
        p_session_cls.return_value = session
        yield BotoSessionMock(session)


def invoke_with_default_args(args: Sequence[str]) -> Result:
    return CliRunner().invoke(cli.main, [*DEFAULT_ARGS, *args])
