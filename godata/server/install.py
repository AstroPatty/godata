# from . import SERVER_INSTALL_PATH
import os
import platform
import shutil
import subprocess
import zipfile
from pathlib import Path

import requests

from godata import server

ENDPOINT = "https://sqm13wjyaf.execute-api.us-west-2.amazonaws.com/godata/download"
DEFAULT_SERVER_INSTALL_LOCATION = Path.home() / ".local" / "bin"


def install(upgrade=False, version=None):
    """Install the godata server binary to /usr/local/bin/godata_server."""
    # detect the os this script is running on
    if upgrade and version is None:
        raise ValueError("Must specify the current version when upgrading.")

    params = {
        "install_type": "upgrade" if upgrade else "install",
        "os": platform.system().lower(),
    }

    if upgrade:
        params["current_version"] = version
    arch = platform.machine()
    if arch == "arm64":
        params["architecture"] = "aarch64"
    else:
        params["architecture"] = arch
    result = requests.get(ENDPOINT, params=params)
    if result.status_code == 200:
        response = result.json()
    else:
        print("Error: ", result.text)
        return

    url = response["url"]
    result = requests.get(url)
    # download the zip file
    # Stop the server if it is already running
    print("Stopping server if it is running...")
    server.stop()

    with open("godata_server.zip", "wb") as f:
        f.write(result.content)
    # unzip the file
    with zipfile.ZipFile("godata_server.zip", "r") as zip_ref:
        zip_ref.extractall()
    # move the binary to the install path
    install_path = server.get_server_path()

    shutil.move("godata_server", install_path)
    # remove the zip file
    os.remove("godata_server.zip")
    # make the binary executable
    os.chmod(install_path, 0o755)
    print("Restarting server...")
    server.start()
    if version is None:
        print(f"Successfully installed godata server version {response['version']}")
    else:
        print(
            f"Successfully upgraded godata server to version {response['version']} "
            f"from version {version}"
        )


def upgrade():
    """Upgrade the godata server binary to the latest version."""
    current_version = get_version().strip("\n")
    install(upgrade=True, version=current_version)


def get_version():
    install_path = server.get_server_path()
    try:
        return subprocess.check_output([f"{install_path}", "--version"]).decode("utf-8")
    except FileNotFoundError:
        raise FileNotFoundError(
            "Unable to get godata server version: could not find the server binary. "
            "Please run `godata server install` first."
        )
