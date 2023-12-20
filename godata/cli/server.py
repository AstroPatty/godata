import click

from godata import server as srv


# top level godata command
@click.group()
def server():
    pass


# server subcommand
@server.command()
def start():
    try:
        srv.start()
        print("Server started and listening...")
    except Exception as e:
        print(e)


@server.command()
def stop():
    try:
        srv.stop()
        print("Server stopped.")
    except Exception as e:
        print(e)


@server.command()
def install():
    pass


@server.command()
def uninstall():
    pass
