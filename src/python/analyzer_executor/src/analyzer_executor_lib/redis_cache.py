from typing import Optional, Union

import redis
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger()


def construct_redis_client(addr: Optional[str], port: Optional[int]) -> redis.Redis:
    if addr and port:
        LOGGER.debug(f"connecting to redis at {addr}:{port}")
        return redis.Redis(host=addr, port=port, db=0)
    else:
        raise ValueError(f"Failed connecting to redis | addr:\t{addr} | port:\t{port}")


class NopCache(object):
    def set(self, key: str, value: str) -> None:
        pass

    def get(self, key: str) -> bool:
        return False

    def delete(self, key: str) -> None:
        pass


EitherCache = Union[NopCache, redis.Redis]
