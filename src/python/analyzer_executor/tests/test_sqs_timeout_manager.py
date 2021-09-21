from __future__ import annotations

import asyncio
import unittest
from unittest.mock import Mock

import pytest
from grapl_common.sqs.sqs_timeout_manager import (
    SqsTimeoutManager,
    SqsTimeoutManagerException,
)
from grapl_common.sqs.sqs_types import SQSMessageId, SQSReceiptHandle


class TestSqsTimeoutManager(unittest.TestCase):
    def test_keep_alive_fails(self) -> None:
        f = SqsTimeoutManagerFixture()
        with pytest.raises(SqsTimeoutManagerException):
            asyncio.run(f.timeout_manager.keep_alive())

        assert f.state == "started"
        assert f.sqs_client.change_message_visibility.call_count == 2

    def test_keep_alive_completion(self) -> None:
        f = SqsTimeoutManagerFixture()
        f.timeout_manager.visibility_timeout = 20

        asyncio.run(f.timeout_manager.keep_alive())
        assert f.state == "completed"
        assert f.sqs_client.change_message_visibility.call_count == 1

    def test_sleep_and_timeout(self) -> None:
        f = SqsTimeoutManagerFixture()
        f.timeout_manager.visibility_timeout = 30

        # initial visibility is 30. then, 10 seconds before that:
        assert f.timeout_manager._get_next_sleep(1) == 20
        # we promote the visibility to 60.
        assert f.timeout_manager._get_next_visibility(1) == 60

        # okay, so visibility is 60, and SQS treats it as if 0 seconds have elapsed.
        # Let's sleep 60-10 seconds...
        assert f.timeout_manager._get_next_sleep(2) == 50
        # at which point, we promote again, to 90
        assert f.timeout_manager._get_next_visibility(2) == 90

        _quiet_never_awaited_error(f)


def _quiet_never_awaited_error(f: SqsTimeoutManagerFixture) -> None:
    """
    Some tests that consume SqsTimeoutManagerFixture never await. That's OK.
    Solves "coroutine was never awaited"
    """

    async def cancel_it() -> None:
        asyncio.create_task(f.timeout_manager.coroutine).cancel()

    asyncio.run(cancel_it())


class SqsTimeoutManagerFixture:
    def __init__(self) -> None:
        # 22 seconds is not enough for
        self.state = "initialized"

        async def wait_25_secs() -> None:
            self.state = "started"
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
