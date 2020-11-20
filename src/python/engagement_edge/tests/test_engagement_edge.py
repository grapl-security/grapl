import unittest

from chalice.test import Client
from src.engagement_edge import JWT_SECRET, app

# gross hack because engagement edge is pseudo singleton
JWT_SECRET.secret = "hey im a fake secret"


class TestEngagementEdgeChalice(unittest.TestCase):
    def test__a_requires_auth_path_fails_without_cookie_headers(self):
        with Client(app) as client:
            result = client.http.post("/getNotebook")
            assert result.status_code == 400
            assert result.json_body == {'error': 'Must log in'}