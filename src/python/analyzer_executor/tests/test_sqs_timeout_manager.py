import asyncio
import unittest
from dataclasses import dataclass
from typing import TYPE_CHECKING, Coroutine
from unittest.mock import Mock

import pytest
from analyzer_executor_lib.sqs_timeout_manager import (
    SqsTimeoutManager,
    SqsTimeoutManagerException,
)
from analyzer_executor_lib.sqs_types import SQSMessageId, SQSReceiptHandle
from grapl_common.grapl_logger import get_module_grapl_logger


class TestSqsTimeoutManager(unittest.TestCase):
    def test_keep_alive_fails(self):
        f = SqsTimeoutManagerFixture()
        with pytest.raises(SqsTimeoutManagerException):
            asyncio.run(f.timeout_manager.keep_alive())

        assert f.state == "started"
        assert f.sqs_client.change_message_visibility.call_count == 2

    def test_keep_alive_completion(self):
        f = SqsTimeoutManagerFixture()
        f.timeout_manager.visibility_timeout = 20

        asyncio.run(f.timeout_manager.keep_alive())
        assert f.state == "completed"
        assert f.sqs_client.change_message_visibility.call_count == 1


class SqsTimeoutManagerFixture:
    def __init__(self) -> None:
        # 22 seconds is not enough for
        self.state = "started"

        async def wait_25_secs() -> None:
            await asyncio.sleep(25)
            self.state = "completed"

        self.sqs_client = Mock()

        self.timeout_manager = SqsTimeoutManager(
            sqs_client=self.sqs_client,
            queue_url="queue_url",
            receipt_handle=SQSReceiptHandle("receipt_handle"),
            message_id=SQSMessageId("message_id"),
            coroutine=wait_25_secs(),
            visibility_timeout=11,
            num_loops=2,
        )
