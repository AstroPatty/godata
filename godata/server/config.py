import json
from pathlib import Path
from typing import Optional

import appdirs
import portalocker
import pydantic

from .install import DEFAULT_SERVER_INSTALL_LOCATION

SERVER_CONFIG_PATH = Path(appdirs.user_config_dir("godata")) / "godata_server.json"


class ServerConfig(pydantic.BaseModel):
    """
    Configuration specification for the godata server
    """

    is_running: bool = False
    server_url: Optional[str] = None
    server_path: Path = DEFAULT_SERVER_INSTALL_LOCATION / "godata_server"
    port: Optional[int] = pydantic.Field(ge=0, le=65535, default=None)

    def stop(self):
        self.is_running = False
        self.server_url = None
        self.port = None


# Create the default server config file if it does not exist
def create_default_config():
    if not SERVER_CONFIG_PATH.parent.exists():
        SERVER_CONFIG_PATH.parent.mkdir(parents=True)
    if not SERVER_CONFIG_PATH.exists():
        with portalocker.Lock(SERVER_CONFIG_PATH, "w") as f:
            json_data = ServerConfig().model_dump_json(indent=2)
            f.write(json_data)


def get_config():
    with portalocker.Lock(SERVER_CONFIG_PATH, "r") as f:
        config = json.load(f)
        config = ServerConfig(**config)
    return config


create_default_config()
