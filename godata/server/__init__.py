import os
import signal
import subprocess
import time

import requests

from godata.client import client

from .install import SERVER_INSTALL_PATH, install, upgrade


def start():
    try:
        client.check_server()
    except client.GodataClientError as e:
        # Server is running, but client has a bad version
        print(e)
        return
    except requests.exceptions.ConnectionError:
        is_running = False
    else:
        is_running = True
    if is_running:
        print("Server is already running.")
        return

    try:
        subprocess.Popen([f"{SERVER_INSTALL_PATH}"], close_fds=True, shell=True)
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

    os.kill(int(server_pid), signal.SIGINT)


__all__ = ["start", "stop", "install", "upgrade"]
