import unittest

import pytest

@pytest.mark.integration_test
class TestIntegrationDefault(unittest.TestCase):
    def test_integration_grapl_http_service(self) -> None:
        raise NotImplementedError

