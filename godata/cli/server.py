from pathlib import Path

import click

from godata import server as srv


# top level godata command
@click.group()
def server():
    pass


# server subcommand
@server.command()
@click.option(
    "--port",
    "-p",
    help="Port to start the server on. Default is 8000",
    type=int,
)
def start(port: int = None):
    """
    Start the godata server. This will start the server in the background. By default,
    the server will be started on a unix socker, unless the --port option is used,
    in which case the server will be started on a TCP socket. If running on Windows,
    the server will always be started on a TCP socket.
    """
    srv.start(port=port)


@server.command()
def stop():
    """
    Stop the godata server if it is running.
    """
    try:
        srv.stop()
    except Exception as e:
        print(e)


@server.command()
@click.option(
    "--upgrade",
    "-u",
    is_flag=True,
    help="Upgrade the server to the latest version.",
)
@click.option(
    "--path",
    "-p",
    help="Path to install the server binary.",
    type=click.Path(exists=True, file_okay=False, resolve_path=True, path_type=Path),
    default=srv.SERVER_INSTALL_PATH,
)
def install(upgrade: bool = False, path: Path = srv.SERVER_INSTALL_PATH):
    if upgrade:
        srv.upgrade(path)
    else:
        srv.install(path)


@server.command()
def uninstall():
    pass
