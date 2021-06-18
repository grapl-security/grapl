from contextlib import contextmanager
from typing import Any, Dict, Iterator, Sequence, cast
from unittest.mock import MagicMock, Mock, patch

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
        # We'll store `session.client("thing")` in a map for easy access
        self.clients: Dict[str, Mock] = {}

        def _memoize_client(service_name: str, **kwargs: Any) -> Mock:
            # This is called when you call `session.client("thing")`
            if service_name not in self.clients:
                self.clients[service_name] = MagicMock(
                    name=f"BotoSessionMock {service_name} client"
                )

            return self.clients[service_name]

        self.session.client.side_effect = _memoize_client
        self.session.resource.side_effect = _memoize_client

    def client(self, client_name: str) -> Mock:
        return cast(Mock, self.session.client(client_name))

    def resource(self, client_name: str) -> Mock:
        return self.client(client_name)

    def return_fake_queues(self) -> None:
        sqs_client = self.client("sqs")
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
def patch_boto3_session() -> Iterator[BotoSessionMock]:
    with patch.object(
        cli.boto3.session,
        cli.boto3.session.Session.__name__,
        spec_set=cli.boto3.session.Session,
    ) as p_session_cls:
        session = Mock(name="Session instance")
        p_session_cls.return_value = session
        yield BotoSessionMock(session)


def invoke_with_default_args(args: Sequence[str]) -> Result:
    return CliRunner().invoke(cli.main, [*DEFAULT_ARGS, *args], catch_exceptions=False)
