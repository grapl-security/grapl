import os

from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor
from analyzer_executor_lib.event_retriever import EventRetriever
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger

wait_for_vsc_debugger(service="analyzer_executor")

ANALYZER_EXECUTOR = AnalyzerExecutor.singleton()


def main():
    """
    This will eventually become the basis of the Python equivalent of `process_loop()`.
    Some TODOs:
    - make sure SOURCE_QUEUE_URL is also specified in CDK
    - add
      RETRY_QUEUE_URL
      DEAD_LETTER_QUEUE_URL
      DEST_QUEUE_URL
    - pull the manual eventing out of `lambda_handler_fn` and into an EventEmitter
    """
    queue_url = os.environ["SOURCE_QUEUE_URL"]
    retriever = EventRetriever(queue_url=queue_url)
    for sqs_message_body in retriever.retrieve():
        ANALYZER_EXECUTOR.lambda_handler_fn(sqs_message_body, {})


main()
