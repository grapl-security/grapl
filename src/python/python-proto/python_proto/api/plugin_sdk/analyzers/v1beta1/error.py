from contextlib import asynccontextmanager
from typing import AsyncIterator, TypeVar

import grpc
from grpc import aio as grpc_aio  # type: ignore

# ^ Type checking doesn't exist yet for gRPC asyncio runtime


E = TypeVar("E", bound=Exception)


@asynccontextmanager
async def exception_as_grpc_abort(
    exception_cls: type[E], context: grpc_aio.ServicerContext
) -> AsyncIterator[None]:
    try:
        yield
    except exception_cls as e:
        details = f"error_as_grpc_abort exception: {str(e)}"
        code = grpc.StatusCode.UNKNOWN
        await context.abort(
            code=code,
            details=details,
        )
