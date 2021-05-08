import unittest

import pytest

@pytest.mark.unit_test
class TestDefault(unittest.TestCase):
    def test_{{cookiecutter.pkg_name}}(self) -> None:
        raise NotImplementedError

