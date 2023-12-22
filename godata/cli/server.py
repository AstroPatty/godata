import click

from godata import server as srv


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
        srv.start()
        print("Server started and listening...")
    except Exception as e:
        print(e)


@server.command()
def stop():
    """
    Stop the godata server if it is running.
    """
    try:
        srv.stop()
        print("Server stopped.")
    except Exception as e:
        print(e)


@server.command()
def install():
    srv.install()


@server.command()
def uninstall():
    pass
