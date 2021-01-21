import asyncio
from typing import Any

from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor
from analyzer_executor_lib.s3_types import SQSMessageBody


def lambda_handler(events: SQSMessageBody, context: Any) -> None:
    return asyncio.run(
        AnalyzerExecutor.singleton().lambda_handler_fn(events=events, context=context)
    )
