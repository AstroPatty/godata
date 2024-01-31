import os
import pickle
import signal
import subprocess
import time
from functools import cache
from pathlib import Path
from urllib import parse

import appdirs

from .install import DEFAULT_SERVER_INSTALL_LOCATION, install, upgrade


@cache
def get_server_path():
    config_path = Path(appdirs.user_config_dir("godata")) / "server_path"
    if not config_path.exists():
        return DEFAULT_SERVER_INSTALL_LOCATION / "godata_server"

    with open(config_path, "rb") as f:
        return pickle.load(f)


@cache
def get_server_location():
    full_path = get_server_path()
    return full_path.parent


def set_server_location(path: Path):
    server_path = path / "godata_server"
    config_path = Path(appdirs.user_config_dir("godata"))
    config_path.mkdir(parents=True, exist_ok=True)
    path_path = config_path / "server_path"
    with open(path_path, "wb") as f:
        pickle.dump(server_path, f)


def start(port: int = None):
    # check if a godata_server process is already running

    try:
        server_pid = subprocess.check_output(["pgrep", "godata_server"])
        print(
            f"Server is already running with PID {int(server_pid)}. "
            "Please stop the server before starting a new one."
        )
        return
    except subprocess.CalledProcessError:
        pass

    try:
        command = str(get_server_path())
        if port:
            command += f" --port={port}"
            url = f"http://localhost:{port}"
        else:
            SERVER_PATH = str(Path.home() / ".godata.sock")
            url = f"http+unix://{parse.quote(SERVER_PATH, safe='')}"
        FILE_OUTPUT_PATH = Path.home() / ".godata_server"
        with open(FILE_OUTPUT_PATH, "w") as f:
            f.write(url)
        subprocess.Popen(command, close_fds=True, shell=True)
    except FileNotFoundError:
        raise FileNotFoundError(
            "Unable to start godata server: could not find the server binary. "
            "Please run `godata server install` first."
        )
    time.sleep(0.5)
    return True


def stop():
    try:
        server_pid = subprocess.check_output(["pgrep", "godata_server"])
    except subprocess.CalledProcessError:
        print("Server is not running.")
        return
    # kill the server
    os.kill(int(server_pid), signal.SIGINT)
    # remove the file that stores the server url
    FILE_OUTPUT_PATH = Path.home() / ".godata_server"
    if FILE_OUTPUT_PATH.exists():
        FILE_OUTPUT_PATH.unlink()


__all__ = ["start", "stop", "install", "upgrade"]
