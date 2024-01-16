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
    srv.start()


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
def install(upgrade):
    if upgrade:
        srv.upgrade()
    else:
        srv.install()


@server.command()
def uninstall():
    pass
