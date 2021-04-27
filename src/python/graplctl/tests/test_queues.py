from tests.shared import invoke_with_default_args, patch_boto3_session


def test_queues_ls() -> None:
    with patch_boto3_session() as mock_session:
        mock_session.return_fake_queues()
        result = invoke_with_default_args(["queues", "ls"])
    assert result.exit_code == 0
    assert (
        result.output
        == "http://queue1-dead-letter-queue\nhttp://queue2-dead-letter-queue\n"
    )


# TODO: Add a `test_queues_redrive`
