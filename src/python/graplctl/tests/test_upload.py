from unittest.mock import ANY

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

    mock_s3_client = mock_session.session.client.return_value
    mock_s3_client.put_object.assert_called_with(
        Body=ANY,  # technically, you could read the contents of `fake_analyzer_main` but whatever
        Bucket="fake-deployment-analyzers-bucket",
        Key="analyzers/fake_analyzer/main.py",
    )
    assert result.exit_code == 0
    assert result.output == "Uploaded analyzer 'fake_analyzer'\n"
