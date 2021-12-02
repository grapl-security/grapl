import os
from typing import Optional


def endpoint_url(suffix: Optional[str]) -> str:
    """
    Builds the URL for the Grapl API endpoint corresponding to
    the given suffix. This function expects that GRAPL_API_HOST
    and GRAPL_HTTP_FRONTEND_PORT are set.
    """
    host = os.environ["GRAPL_API_HOST"]
    port = int(os.environ["GRAPL_HTTP_FRONTEND_PORT"])
    return f"http://{host}:{port}{suffix}"
