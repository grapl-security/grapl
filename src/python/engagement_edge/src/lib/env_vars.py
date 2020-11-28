import os

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))
GRAPL_LOG_LEVEL = os.environ.get("GRAPL_LOG_LEVEL", "ERROR")
BUCKET_PREFIX = os.environ["BUCKET_PREFIX"]
