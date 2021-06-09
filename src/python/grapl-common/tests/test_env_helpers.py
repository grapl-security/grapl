import os
import unittest
from typing import Any
from unittest.mock import patch

from botocore.config import Config
from grapl_common.env_helpers import ClientGetParams, FromEnvException, _client_get

_FAKE_ENDPOINT_URL = "FAKE_ENDPOINT_URL"
_FAKE_AWS_ACCESS_KEY_ID_KEY = "FAKE_AWS_ACCESS_KEY_ID"
_FAKE_AWS_SECRET_ACCESS_KEY_KEY = "FAKE_AWS_SECRET_ACCESS_KEY"
_FAKE_AWS_SESSION_TOKEN_KEY = "FAKE_AWS_SESSION_TOKEN"
_CLIENT_GET_PARAMS = ClientGetParams(
    "fake_boto3",  # boto3_client_name
    _FAKE_ENDPOINT_URL,  # endpoint_url_key
    _FAKE_AWS_ACCESS_KEY_ID_KEY,  # access_key_id_key
    _FAKE_AWS_SECRET_ACCESS_KEY_KEY,  # access_key_secret_key
    _FAKE_AWS_SESSION_TOKEN_KEY,  # access_session_token
)


def _client_create_fn(_: Any, __: Any, ___: Any) -> Any:
    raise AssertionError("unexpected call to _client_create_fn")


class TestEnvHelpers(unittest.TestCase):
    def test_client_get_throws_FromEnvException_when_config_and_env_vars_absent(
        self,
    ) -> None:
        """check that the function errors out in the expected
        way when no aws region is set in the environment and the
        config object is absent"""
        with patch.dict(os.environ, {}) as environ_mock:
            try:
                assert os.getenv("AWS_REGION") is None
                assert os.getenv("AWS_DEFAULT_REGION") is None
                for key in (
                    _FAKE_AWS_ACCESS_KEY_ID_KEY,
                    _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                    _FAKE_AWS_SESSION_TOKEN_KEY,
                ):
                    assert os.getenv(key) is None
                _client_get(
                    client_create_fn=_client_create_fn,
                    params=_CLIENT_GET_PARAMS,
                    config=None,
                )
            except FromEnvException as e:
                return  # great success!
            raise AssertionError("expected FromEnvException but none was thrown")

    def test_client_get_throws_FromEnvException_when_config_present_and_region_None_and_env_vars_absent(
        self,
    ) -> None:
        """check that the function errors out in the expected
        way when no aws region is set in the environment and
        the config object is present but config.region_name is
        absent"""
        with patch.dict(os.environ, {}) as environ_mock:
            try:
                assert os.getenv("AWS_REGION") is None
                assert os.getenv("AWS_DEFAULT_REGION") is None
                for key in (
                    _FAKE_AWS_ACCESS_KEY_ID_KEY,
                    _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                    _FAKE_AWS_SESSION_TOKEN_KEY,
                ):
                    assert os.getenv(key) is None
                config = Config()
                assert config.region_name is None
                _client_get(
                    client_create_fn=_client_create_fn,
                    params=_CLIENT_GET_PARAMS,
                    config=config,
                )
            except FromEnvException:
                return  # great success!
            raise AssertionError("expected FromEnvException but none was thrown")

    def test_client_get_returns_expected_when_config_present_and_env_vars_absent(
        self,
    ) -> None:
        """check that the function returns the expected value
        when no aws region is set in the environment and the
        config object is present with config.region_name set"""
        with patch.dict(os.environ, {}) as environ_mock:
            assert os.getenv("AWS_REGION") is None
            assert os.getenv("AWS_DEFAULT_REGION") is None
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config(region_name="fake-region")
            assert config.region_name == "fake-region"
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "fake-region"

    def test_client_get_returns_expected_when_config_absent_and_AWS_DEFAULT_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_DEFAULT_REGION is set and config object is
        absent"""
        with patch.dict(
            os.environ, {"AWS_DEFAULT_REGION": "fake-region"}
        ) as environ_mock:
            assert os.getenv("AWS_REGION") is None
            assert os.getenv("AWS_DEFAULT_REGION") == "fake-region"
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=None,
            )
            assert result == "fake-region"

    def test_client_get_returns_expected_when_config_absent_and_AWS_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_REGION is set and config object is absent"""
        with patch.dict(os.environ, {"AWS_REGION": "fake-region"}) as environ_mock:
            assert os.getenv("AWS_REGION") == "fake-region"
            assert os.getenv("AWS_DEFAULT_REGION") is None
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=None,
            )
            assert result == "fake-region"

    def test_client_get_returns_expected_when_config_present_and_region_absent_and_AWS_DEFAULT_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_DEFAULT_REGION is set and config object is
        present but config.region_name is absent"""
        with patch.dict(
            os.environ, {"AWS_DEFAULT_REGION": "fake-region"}
        ) as environ_mock:
            assert os.getenv("AWS_REGION") is None
            assert os.getenv("AWS_DEFAULT_REGION") == "fake-region"
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config()
            assert config.region_name is None
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "fake-region"

    def test_client_get_returns_expected_when_config_present_and_region_absent_and_AWS_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_REGION is set and config object is present but
        config.region_name is absent"""
        with patch.dict(os.environ, {"AWS_REGION": "fake-region"}) as environ_mock:
            assert os.getenv("AWS_REGION") == "fake-region"
            assert os.getenv("AWS_DEFAULT_REGION") is None
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config()
            assert config.region_name is None
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "fake-region"

    def test_client_get_returns_expected_when_config_present_and_region_absent_and_AWS_REGION_and_AWS_DEFAULT_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_REGION and AWS_DEFAULT_REGION are set and
        config object is present but config.region_name is
        absent"""
        with patch.dict(
            os.environ,
            {
                "AWS_REGION": "fake-region",
                "AWS_DEFAULT_REGION": "preferred-fake-region",
            },
        ) as environ_mock:
            assert os.getenv("AWS_REGION") == "fake-region"
            assert os.getenv("AWS_DEFAULT_REGION") == "preferred-fake-region"
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config()
            assert config.region_name is None
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "preferred-fake-region"

    def test_client_get_returns_expected_when_config_present_and_region_present_and_AWS_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_REGION is set and config object is present and
        config.region_name is present"""
        with patch.dict(os.environ, {"AWS_REGION": "fake-region"}) as environ_mock:
            assert os.getenv("AWS_REGION") == "fake-region"
            assert os.getenv("AWS_DEFAULT_REGION") is None
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config(region_name="preferred-fake-region")
            assert config.region_name == "preferred-fake-region"
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "preferred-fake-region"

    def test_client_get_returns_expected_when_config_present_and_region_present_and_AWS_DEFAULT_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_DEFAULT_REGION is set and config object is
        present and config.region_name is present"""
        with patch.dict(
            os.environ, {"AWS_DEFAULT_REGION": "fake-region"}
        ) as environ_mock:
            assert os.getenv("AWS_REGION") is None
            assert os.getenv("AWS_DEFAULT_REGION") == "fake-region"
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config(region_name="preferred-fake-region")
            assert config.region_name == "preferred-fake-region"
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "preferred-fake-region"

    def test_client_get_returns_expected_when_config_present_and_region_present_and_AWS_DEFAULT_REGION_and_AWS_REGION_set(
        self,
    ) -> None:
        """check that the function returns the expected value
        when AWS_REGION and AWS_DEFAULT_REGION are set and
        config object is present and config.region_name is
        present"""
        with patch.dict(
            os.environ,
            {"AWS_REGION": "fake-region", "AWS_DEFAULT_REGION": "fake-default-region"},
        ) as environ_mock:
            assert os.getenv("AWS_REGION") == "fake-region"
            assert os.getenv("AWS_DEFAULT_REGION") == "fake-default-region"
            for key in (
                _FAKE_AWS_ACCESS_KEY_ID_KEY,
                _FAKE_AWS_SECRET_ACCESS_KEY_KEY,
                _FAKE_AWS_SESSION_TOKEN_KEY,
            ):
                assert os.getenv(key) is None
            config = Config(region_name="preferred-fake-region")
            assert config.region_name == "preferred-fake-region"
            result = _client_get(
                client_create_fn=lambda _, region_name, config: region_name,
                params=_CLIENT_GET_PARAMS,
                config=config,
            )
            assert result == "preferred-fake-region"
