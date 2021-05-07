from unittest.mock import ANY, call

from tests.fake_uploads.fake_analyzer import main as fake_analyzer_main_py
from tests.shared import invoke_with_default_args, patch_boto3_session


def test_upload_analyzer__path_is_wrong() -> None:
    main_py_path = "some_inexistent_dir/main.py"
    with patch_boto3_session() as mock_session:
        result = invoke_with_default_args(
            [
                "upload",
                "analyzer",
                "--analyzer_main_py",
                main_py_path,
            ],
        )
    assert result.exit_code != 0
    assert f"File '{main_py_path}' does not exist" in result.output


def test_upload_analyzer__calls_s3() -> None:
    main_py_path = fake_analyzer_main_py.__file__
    with patch_boto3_session() as mock_session:
        result = invoke_with_default_args(
            [
                "upload",
                "analyzer",
                "--analyzer_main_py",
                str(main_py_path),
            ],
        )

    mock_s3_client = mock_session.client("s3")
    mock_s3_client.put_object.assert_called_with(
        Body=ANY,  # technically, you could read the contents of `fake_analyzer_main` but whatever
        Bucket="fake-deployment-analyzers-bucket",
        Key="analyzers/fake_analyzer/main.py",
    )
    assert result.exit_code == 0
    assert result.output == "Uploaded analyzer 'fake_analyzer'\n"


def test_upload_sysmon__calls_s3() -> None:
    with patch_boto3_session() as mock_session:
        sample_data_path = "etc/sample_data/eventlog.xml"
        result = invoke_with_default_args(
            ["upload", "sysmon", "--logfile", sample_data_path],
        )

    mock_s3_client = mock_session.client("s3")
    # Should call to s3 six times
    mock_s3_client.put_object.assert_has_calls(
        [
            call(
                Body=ANY,
                Bucket="fake-deployment-sysmon-log-bucket",
                Key=ANY,
            )
        ]
        * 6
    )

    assert result.exit_code == 0
    assert (
        "Writing events to fake-deployment with 0 seconds between batches of 100"
        in result.output
    )
    assert "Completed uploading 6 chunks" in result.output


def test_upload_osquery__calls_s3() -> None:
    with patch_boto3_session() as mock_session:
        sample_data_path = "etc/sample_data/osquery_data.log"
        result = invoke_with_default_args(
            ["upload", "osquery", "--logfile", sample_data_path],
        )

    mock_s3_client = mock_session.client("s3")
    # Should call to s3 236 times
    mock_s3_client.put_object.assert_has_calls(
        [
            call(
                Body=ANY,
                Bucket="fake-deployment-osquery-log-bucket",
                Key=ANY,
            )
        ]
        * 236
    )

    assert result.exit_code == 0
    assert (
        "Writing events to fake-deployment with 0 seconds between batches of 100"
        in result.output
    )
    assert "Completed uploading 236 chunks" in result.output
