from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import TYPE_CHECKING, Coroutine

from analyzer_executor_lib.grapl_logger import get_module_grapl_logger

if TYPE_CHECKING:
    from mypy_boto3_sqs import SQSClient

LOGGER = get_module_grapl_logger()


class SqsTimeoutManagerException(Exception):
    pass


@dataclass
class SqsTimeoutManager:
    sqs_client: SQSClient
    queue_url: str
    receipt_handle: str
    # only used for tracing/logging, not super important
    message_id: str
    # the coroutine that we want to schedule and best-effort complete
    coroutine: Coroutine
    # queue's vis timeout
    visibility_timeout: int = 30
    num_loops: int = 10

    async def keep_alive(self) -> None:
        """
        you grab a message from SQS
        which has, say, a visibility timeout of 30s
        and at 20s you wanna tell SQS
            "yo chill out I'm definitely still working on this message.
            don't resurrect it in the queue yet.
            let's set a new visibility timeout: 60"
        and at 50s you wanna tell SQS "[...same...]
            let's set a new visibility timeout: 90"
        """
        # Schedule the coroutine once
        task = asyncio.create_task(self.coroutine)

        for i in range(1, self.num_loops + 1):
            time_to_wait = (self.visibility_timeout * i) - 10
            LOGGER.info(f"Loop {i} - waiting {time_to_wait}s for task")
            try:
                await asyncio.wait_for(
                    # shield() prevents task from being canceled by the timeout.
                    asyncio.shield(task),
                    timeout=time_to_wait,
                )
                LOGGER.info(f"Task completed")
                return
            except asyncio.TimeoutError as e:
                new_visibility = self.visibility_timeout * (i + 1)
                LOGGER.info(
                    f"Keep alive timed out, informing SQS to raise visibility to {new_visibility}"
                )
                self._change_visibility(new_visibility)
                # do the SQS message visibility thing

        # If we got here, it means the above `return` never happened
        task.cancel()
        raise SqsTimeoutManagerException(
            f"Couldn't keep message alive: {self.message_id}"
        )

    def _change_visibility(self, new_visibility: int) -> None:
        try:
            self.sqs_client.change_message_visibility(
                QueueUrl=self.queue_url,
                ReceiptHandle=self.receipt_handle,
                VisibilityTimeout=new_visibility,
            )
        except:
            LOGGER.error("Failed to change message visibility", exc_info=True)
