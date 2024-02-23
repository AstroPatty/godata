"""
Godata locks files that it is modifying to prevent multiple processes from modifying
the same file at the same time. This uses the portalocker package.

For user machines (or storage) that's directly attached, this can be done
with a basic file lock. Locking on network storage is more subtle. Portalocker
supports using redis as a lock server.

Godata will check to see if a redis server is running and use it. If no redis server
is running, it will fall back on standard file locks.
"""
from os import environ
from pathlib import Path

import loguru
import portalocker
import redis


# check if redis is running
def get_redis_lock(path: Path, client: redis.Redis) -> portalocker.RedisLock:
    lock = portalocker.RedisLock(str(path), client)
    return lock


def get_file_lock(path: Path) -> portalocker.Lock:
    lock = portalocker.Lock(str(path))
    return lock


REDIS_HOST = environ.get("REDIS_HOST") or "localhost"
REDIS_PORT = environ.get("REDIS_PORT")  # NOT YET SUPPORTED
try:
    REDIS_PORT = int(REDIS_PORT)
except (ValueError, TypeError):
    REDIS_PORT = 6379

REDIS_PASSWORD = environ.get("REDIS_PASSWORD")  # NOT YET SUPPORTED
client = redis.Redis(host=REDIS_HOST, port=REDIS_PORT, password=REDIS_PASSWORD)

try:
    client.ping()

    def get_lock(path: Path):
        return get_redis_lock(path, client)

except redis.ConnectionError:
    loguru.logger.warning("No redis server found. Falling back on file locks.")
    get_lock = get_file_lock
    client = None

__all__ = ["get_lock"]
