from typing import Optional

from pydantic_settings import BaseSettings


class FileConfig(BaseSettings):
    godata_lock_type: str = "file"
    redis_host: Optional[str] = None
    redis_port: Optional[int] = 6379
    redis_password: Optional[str] = None
