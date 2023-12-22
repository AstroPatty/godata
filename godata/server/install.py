# from . import SERVER_INSTALL_PATH
import os
import platform
import shutil
import subprocess
import zipfile

import requests

ENDPOINT = "https://sqm13wjyaf.execute-api.us-west-2.amazonaws.com/godata/download"
SERVER_INSTALL_PATH = "/usr/local/bin/godata_server"


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
    with open("godata_server.zip", "wb") as f:
        f.write(result.content)
    # unzip the file
    with zipfile.ZipFile("godata_server.zip", "r") as zip_ref:
        zip_ref.extractall()
    # move the binary to the install path
    shutil.move("godata_server", SERVER_INSTALL_PATH)
    # remove the zip file
    os.remove("godata_server.zip")
    # make the binary executable
    os.chmod(SERVER_INSTALL_PATH, 0o755)
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
    try:
        return subprocess.check_output([f"{SERVER_INSTALL_PATH}", "--version"]).decode(
            "utf-8"
        )
    except FileNotFoundError:
        raise FileNotFoundError(
            "Unable to get godata server version: could not find the server binary. "
            "Please run `godata server install` first."
        )
