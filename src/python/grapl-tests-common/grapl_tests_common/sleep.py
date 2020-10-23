import time
import logging


def verbose_sleep(secs: int, reason: str) -> None:
    logging.info(f"Sleeping for {secs} secs: {reason}")
    time.sleep(secs)
