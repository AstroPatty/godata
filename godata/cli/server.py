import click
from requests import exceptions

from godata import server as srv
from godata.client.client import GodataClientError, check_server


# top level godata command
@click.group()
def server():
    pass


# server subcommand
@server.command()
def start():
    """
    Start the godata server. This will start the server in the background and return
    """
    try:
        check_server()
    except GodataClientError as e:
        # Server is running, but client has a bad version
        print(e)
        return
    except exceptions.ConnectionError:
        # Server is not running
        srv.start()
        print("Server started.")
        return
    print("Server is already running.")


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
def install():
    srv.install()


@server.command()
def uninstall():
    pass
