import unittest

from chalice.test import Client
from src.engagement_edge import JWT_SECRET, app
from src.lib.sagemaker import create_sagemaker_client

# gross hack because engagement edge is pseudo singleton
JWT_SECRET.secret = "hey im a fake secret"


class TestEngagementEdgeChalice(unittest.TestCase):
    def test_requires_auth_fails_without_cookie_headers(self):
        with Client(app) as cient:
            result = client.http.post(
                "/getNotebook",
                headers={
                    "Origin": "https://local-grapl-engagement-ux-bucket.s3.amazonaws.com"
                },
            )
            result = client.http.post("/getNotebook")
            assert result.status_code == 400
            assert result.json_body == {"error": "Must log in"}


class TestSagemakerClient(unittest.TestCase):
    def test_when_is_local_returns_8888(self):
        client = create_sagemaker_client(is_local=True)
        assert client.get_presigned_url() == "http://localost:8888/"
