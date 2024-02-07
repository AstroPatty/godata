import json
import os
import signal
import subprocess
import time
from pathlib import Path
from urllib import parse

import portalocker

from .config import SERVER_CONFIG_PATH, ServerConfig


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

        return not already_running


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
