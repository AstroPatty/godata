import click


# top level godata command
@click.group()
def server():
    pass


# server subcommand
@server.command()
def start():
    pass


@server.command()
def stop():
    pass


@server.command()
def install():
    pass


@server.command()
def uninstall():
    pass
