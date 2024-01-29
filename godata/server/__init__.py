import os
import signal
import subprocess
import time
from pathlib import Path
from urllib import parse

import psutil

from .install import SERVER_INSTALL_PATH, install, upgrade


def start(port: int = None):
    if os.name == "nt":
        start_windows(port)
    else:
        start_unix(port)


def stop():
    if os.name == "nt":
        stop_windows()
    else:
        stop_unix()


def start_unix(port: int = None):
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
        command = SERVER_INSTALL_PATH

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


def stop_unix():
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


def start_windows(port: int = 8080):
    # check if godata_server.exe is already running
    for p in psutil.process_iter(["pid", "name"]):
        if p.info["name"] == "godata_server.exe":
            print(
                f"Server is already running with PID {p.info['pid']}. "
                "Please stop the server before starting a new one."
            )
            return

    command = SERVER_INSTALL_PATH
    command += f"-p {port}"
    try:
        subprocess.Popen(command, close_fds=True, shell=True)
    except FileNotFoundError:
        raise FileNotFoundError(
            "Unable to start godata server: could not find the server binary. "
            "Please run `godata server install` first."
        )
    url = f"http://localhost:{port}"
    FILE_OUTPUT_PATH = Path.home() / ".godata_server"
    with open(FILE_OUTPUT_PATH, "w") as f:
        f.write(url)


def stop_windows():
    for p in psutil.process_iter(["pid", "name"]):
        if p.info["name"] == "godata_server.exe":
            server_pid = p.info["pid"]
            break
    else:
        print("Server is not running.")
        return
    # kill the server
    subprocess.Popen(f"taskkill /F /PID {int(server_pid)}", shell=True)
    # remove the file that stores the server url
    FILE_OUTPUT_PATH = Path.home() / ".godata_server"
    if FILE_OUTPUT_PATH.exists():
        FILE_OUTPUT_PATH.unlink()


__all__ = ["start", "stop", "install", "upgrade"]
