from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import TYPE_CHECKING, Coroutine

from grapl_common.sqs.sqs_types import SQSMessageId, SQSReceiptHandle
from grapl_common.grapl_logger import get_module_grapl_logger

if TYPE_CHECKING:
    from mypy_boto3_sqs import SQSClient

LOGGER = get_module_grapl_logger()


class SqsTimeoutManagerException(Exception):
    pass


@dataclass
class SqsTimeoutManager:
    sqs_client: SQSClient
    queue_url: str
    receipt_handle: SQSReceiptHandle
    # only used for tracing/logging, not super important
    message_id: SQSMessageId
    # the coroutine that we want to schedule and best-effort complete
    coroutine: Coroutine
    # queue's vis timeout
    visibility_timeout: int = 30
    num_loops: int = 10

    def __post_init__(self) -> None:
        assert self.visibility_timeout > 10

    async def keep_alive(self) -> None:
        """
        (Visibility timeout means, "if you haven't finished processing a message by then,
        SQS will put it back on the queue")

        You grab a message from SQS
        which has, by default, a visibility timeout of 30s

        at 20s you wanna tell SQS
            "yo chill out I'm definitely still working on this message.
            don't resurrect it in the queue yet.
            let's set a new visibility timeout: 60"

        SQS starts counting from 0 again, up to 60...
        but at 50s you wanna tell SQS "chill out!
            let's set a new visibility timeout: 90"

        rinse repeat
        """
        # Schedule the coroutine once
        task = asyncio.create_task(self.coroutine)

        for i in range(1, self.num_loops + 1):
            time_to_wait = self._get_next_sleep(i)
            LOGGER.info(
                f"SQS MessageID {self.message_id}: Loop {i} - waiting {time_to_wait}s for task"
            )
            try:
                await asyncio.wait_for(
                    # shield() prevents task from being canceled by the timeout.
                    asyncio.shield(task),
                    timeout=time_to_wait,
                )
                LOGGER.info(f"SQS MessageID {self.message_id}: Task completed")
                return
            except asyncio.TimeoutError as e:
                new_visibility = self._get_next_visibility(i)
                LOGGER.info(
                    f"SQS MessageID {self.message_id}: still processing, telling SQS to raise visibility to {new_visibility}"
                )
                self._change_visibility(new_visibility)
                # do the SQS message visibility thing

        # If we got here, it means the above `return` never happened
        task.cancel()
        raise SqsTimeoutManagerException(
            f"SQS MessageID {self.message_id}: processing never completed"
        )

    def _get_next_sleep(self, loop_i: int) -> int:
        """
        so with timeout 30:
        20, then 50, then 80
        """
        assert loop_i > 0
        return (self.visibility_timeout * loop_i) - 10

    def _get_next_visibility(self, loop_i: int) -> int:
        """
        so with timeout 30:
        (the message is, by default 30; and then:)
        60, then 90, then 120...
        """
        assert loop_i > 0
        return self.visibility_timeout * (loop_i + 1)

    def _change_visibility(self, new_visibility: int) -> None:
        """
        Worth noting: the message's elapsed timeout resets when you change message visibility;
        it is as if you'd just popped it off of SQS at 0 seconds.
        """
        try:
            self.sqs_client.change_message_visibility(
                QueueUrl=self.queue_url,
                ReceiptHandle=self.receipt_handle,
                VisibilityTimeout=new_visibility,
            )
        except:
            LOGGER.error(
                f"SQS MessageID {self.message_id}: Failed to change message visibility",
                exc_info=True,
            )
