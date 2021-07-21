"""
A very minor smoke test to prevent regressions where Engagement UX is no longer accessible.
This should be replaced by Selenium or something like that.
But - a small smoke test is beneficial here.
"""

from http import HTTPStatus

import requests
from grapl_tests_common.clients.common import endpoint_url


class EngagementViewScraper:
    def __init__(self) -> None:
        self.endpoint = endpoint_url("/index.html")

    def get_index(self) -> requests.Response:
        resp = requests.get(url=self.endpoint)
        if resp.status_code != HTTPStatus.OK:
            raise Exception(f"{resp.status_code}: {resp.text}")
        return resp


def test_engagement_view_reachable_when_is_local() -> None:
    scraper = EngagementViewScraper()
    resp = scraper.get_index()
    # I could file-read it, but - this is a cheap smoke test, whatever
    contents_substr = (
        "Grapl: A Graph Analytics Platform for Detection & Response"  # from index.html
    )
    assert contents_substr in resp.content.decode("utf8")
