import unittest

import pytest

@pytest.mark.integration_test
class TestIntegrationDefault(unittest.TestCase):
    def test_integration_{{cookiecutter.pkg_name}}(self) -> None:
        raise NotImplementedError

