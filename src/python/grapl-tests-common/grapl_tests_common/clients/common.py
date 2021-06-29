import os
from typing import Optional

from grapl_common.utils.primitive_convertors import to_bool

IS_LOCAL = to_bool(os.getenv("IS_LOCAL", default=False))


def endpoint_url(suffix: Optional[str]) -> str:
    """Builds the URL for the Grapl API endpoint corresponding to
    the given suffix. This function expects that GRAPL_API_HOST
    is set, and if running locally, that GRAPL_HTTP_FRONTEND_PORT
    is set also.
    """
    port = int(os.environ["GRAPL_HTTP_FRONTEND_PORT"]) if IS_LOCAL else 443
    protocol = "http" if IS_LOCAL else "https"
    stage = "" if IS_LOCAL else "/prod"
    return f'{protocol}://{os.environ["GRAPL_API_HOST"]}:{port}{stage}{suffix}'
