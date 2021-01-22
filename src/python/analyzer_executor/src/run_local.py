import os

from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor
from analyzer_executor_lib.event_retriever import EventRetriever
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger

<<<<<<< HEAD
ANALYZER_EXECUTOR = AnalyzerExecutor.singleton()

i = 0
LOGGER.info("Starting analyzer-executor")
while True:
    try:
        sqs = SQSClientFactory(boto3).from_env()

        alive = False
        while not alive:
            try:
                if "QueueUrls" not in sqs.list_queues(
                    QueueNamePrefix="grapl-analyzer-executor-queue"
                ):
                    LOGGER.info(
                        "Waiting for grapl-analyzer-executor-queue to be created"
                    )
                    time.sleep(2)
                    continue
            except (
                botocore.exceptions.BotoCoreError,
                botocore.exceptions.ClientError,
                botocore.parsers.ResponseParserError,
            ):
                i += 1
                if i >= 10:
                    LOGGER.error(f"Waiting for SQS to become available {i}")
                else:
                    LOGGER.warning(f"Waiting for SQS to become available {i}")
                time.sleep(2)
                continue
            alive = True
=======
wait_for_vsc_debugger(service="analyzer_executor")
>>>>>>> staging

ANALYZER_EXECUTOR = AnalyzerExecutor.singleton()


<<<<<<< HEAD
    except Exception as e:
        LOGGER.error(traceback.format_exc())
        time.sleep(2)

LOGGER.info("Exiting")
=======
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
>>>>>>> staging
