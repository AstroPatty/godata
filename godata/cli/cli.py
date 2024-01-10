import click

from . import project
from .server import server


@click.group()
def main():
    """Command line interface for GoData."""
    pass


main.add_command(project.create)
main.add_command(project.link)
main.add_command(project.ls)
main.add_command(project.list)
main.add_command(server)


if __name__ == "__main__":
    main()
