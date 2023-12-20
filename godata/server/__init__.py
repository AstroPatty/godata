import subprocess

SERVER_INSTALL_PATH = "/usr/local/bin/godata_server"


def start():
    try:
        subprocess.Popen([f"{SERVER_INSTALL_PATH}", "&"])
    except FileNotFoundError:
        raise FileNotFoundError(
            "Unable to start godata server: could not find the server binary. "
            "Please run `godata server install` first."
        )


def stop():
    res = subprocess.run(["pkill", "godata_server"])
    if res.returncode != 0:
        raise Exception("Could not stop godata server. Perhaps it was not running?")
    else:
        return True


__all__ = ["start", "stop"]
