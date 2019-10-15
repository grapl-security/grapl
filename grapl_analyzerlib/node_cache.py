import os

from redis import Redis

from grapl_analyzerlib.querying import Queryable


class NodeCache(object):
    def __init__(self, redis_client: Redis):
        self.redis_client = redis_client

    @staticmethod
    def from_env() -> 'NodeCache':
        COUNTCACHE_ADDR = os.environ['COUNTCACHE_ADDR']
        COUNTCACHE_PORT = os.environ['COUNTCACHE_PORT']

        redis_client = Redis(host=COUNTCACHE_ADDR, port=COUNTCACHE_PORT, db=0, decode_responses=True)
        return NodeCache(redis_client=redis_client)

    def check_count_cache(self, queryable: Queryable, count=False, first=100):
        raw_query = queryable.to
