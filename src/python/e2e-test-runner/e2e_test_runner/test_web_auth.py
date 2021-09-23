"""
TODO (wimax July 2020): I don't see anything in here that indicates that
screams "e2e test", this certainly seems like more of an integration test.

There's nothing here that does anything cross-service.
Perhaps it's just "does it work in AWS?"
"""
from grapl_tests_common.clients.grapl_web_client import GraplWebClient


def test_real_user_fake_password() -> None:
    # Exercises the PasswordVerification case in grapl-web-ui login.rs
    resp = GraplWebClient().real_user_fake_password()
    assert resp.status_code == 401


def test_nonexistent_user() -> None:
    # Exercises the UserRecordNotFound case in grapl-web-ui login.rs
    resp = GraplWebClient().nonexistent_user()
    assert resp.status_code == 401


def test_check__empty_creds() -> None:
    resp = GraplWebClient().empty_creds()
    assert resp.status_code == 500


# TODO: https://github.com/grapl-security/issue-tracker/issues/686
# Add a `test_no_content_type()` (it currently 200s for some reason)
