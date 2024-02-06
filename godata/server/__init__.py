import json
import os
import signal
import subprocess
import time
from pathlib import Path
from typing import Optional
from urllib import parse

import appdirs
import portalocker
import pydantic

from .install import DEFAULT_SERVER_INSTALL_LOCATION, install, upgrade


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
SERVER_CONFIG_PATH = Path(appdirs.user_config_dir("godata")) / "godata_server.json"
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


def set_server_location(path: Path):
    if not path.is_dir():
        raise ValueError(f"{path} is not a valid directory.")
    binary_path = path / "godata_server"

    with portalocker.Lock(SERVER_CONFIG_PATH, "r+") as f:
        config = json.load(f)
        config = ServerConfig(**config)
        if config.is_running:
            raise RuntimeError("Cannot change server path while server is running")
        config.server_path = binary_path
        # clear the file contents
        f.seek(0)
        f.truncate()
        f.write(config.model_dump_json(indent=2))


def start(port: int = None):
    # check if a godata_server process is already running
    with portalocker.Lock(SERVER_CONFIG_PATH, "r+") as f:
        config = json.load(f)
        config = ServerConfig(**config)
        already_running = config.is_running

        if not already_running:
            config.is_running = True
            config.port = port

            try:
                command = str(config.server_path)
                if port:
                    command += f" --port={port}"
                    url = f"http://localhost:{port}"
                else:
                    SERVER_PATH = str(Path.home() / ".godata.sock")
                    url = f"http+unix://{parse.quote(SERVER_PATH, safe='')}"

                config.server_url = url
                subprocess.Popen(command, close_fds=True, shell=True)
            except FileNotFoundError:
                raise FileNotFoundError(
                    "Unable to start godata server: could not find the server binary. "
                    "Please run `godata server install` first."
                )
            f.seek(0)
            f.truncate()
            f.write(config.model_dump_json(indent=2))
            time.sleep(0.1)

    return True


def stop():
    with portalocker.Lock(SERVER_CONFIG_PATH, "r+") as f:
        config = json.load(f)
        config = ServerConfig(**config)
        if not config.is_running:
            raise RuntimeError("Server is not running.")

        # check if the url is a unix socket or localhost
        local_urls = ["http://localhost", "http+unix://"]
        if not any(config.server_url.startswith(url) for url in local_urls):
            raise RuntimeError("Cannot stop server running on a remote host.")

        server_pid = subprocess.check_output(["pgrep", "godata_server"])
        os.kill(int(server_pid), signal.SIGINT)

        config.stop()
        f.seek(0)
        f.truncate()
        f.write(config.model_dump_json(indent=2))


__all__ = ["start", "stop", "install", "upgrade"]
