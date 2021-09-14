"""
TODO (wimax July 2020): I don't see anything in here that indicates that
screams "e2e test", this certainly seems like more of an integration test.

There's nothing here that does anything cross-service.
Perhaps it's just "does it work in AWS?"
"""
from grapl_tests_common.clients.grapl_web_client import GraplWebClient


def test_check__invalid_creds() -> None:
    resp = GraplWebClient().invalid_creds()
    assert resp.status_code == 403, "This should be a 403"


def test_check__empty_creds() -> None:
    resp = GraplWebClient().empty_creds()
    assert resp.status_code == 500, "Expected 500 permissions error"


def test_check__no_content_type() -> None:
    resp = GraplWebClient().no_content_type()
    assert resp.status_code == 500, "Expected 400 bad rotue or something"
