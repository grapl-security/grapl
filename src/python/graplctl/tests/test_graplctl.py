from contextlib import contextmanager
from pathlib import Path
from typing import ContextManager
from unittest.mock import ANY, Mock, patch

from click.testing import CliRunner
from graplctl import __version__, cli
from tests.fake_uploads.fake_analyzer import main as fake_analyzer_main_py


def test_version() -> None:
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
        _return_fake_queues(mock_session)
        result = runner.invoke(cli.main, [*DEFAULT_ARGS, "queues", "ls"])
    assert result.exit_code == 0
    assert (
        result.output
        == "http://queue1-dead-letter-queue\nhttp://queue2-dead-letter-queue\n"
    )


def test_dev_upload_analyzer__path_is_wrong() -> None:
    runner = CliRunner()
    main_py_path = "some_inexistent_dir/main.py"
    with _patch_boto3_session() as mock_session:
        result = runner.invoke(
            cli.main,
            [
                *DEFAULT_ARGS,
                "upload",
                "analyzer",
                "--analyzer_main_py",
                main_py_path,
            ],
        )
    assert result.exit_code != 0
    assert f"File '{main_py_path}' does not exist" in result.output


def test_dev_upload_analyzer__calls_s3() -> None:
    runner = CliRunner()
    main_py_path = fake_analyzer_main_py.__file__

    with _patch_boto3_session() as mock_session:
        _mock_out_s3(mock_session)
        result = runner.invoke(
            cli.main,
            [
                *DEFAULT_ARGS,
                "upload",
                "analyzer",
                "--analyzer_main_py",
                str(main_py_path),
            ],
        )
    mock_s3_client = mock_session.client.return_value
    mock_s3_client.put_object.assert_called_with(
        Body=ANY,  # technically, you could read the contents of `fake_analyzer_main` but whatever
        Bucket="fake-deployment-analyzers-bucket",
        Key="analyzers/fake_analyzer/main.py",
    )
    assert result.exit_code == 0
    assert result.output == "Uploaded analyzer 'fake_analyzer'\n"


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


def _mock_out_s3(mock_session: Mock) -> None:
    s3_client = mock_session.client.return_value
    s3_client.put_object.return_value = None


@contextmanager
def _patch_boto3_session() -> ContextManager[Mock]:
    with patch.object(
        cli.boto3, cli.boto3.Session.__name__, spec_set=cli.boto3.Session
    ) as p_session_cls:
        session = Mock(name="Session instance")
        p_session_cls.return_value = session
        yield session
