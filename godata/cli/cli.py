import click

from .server import server


@click.group()
def main():
    """Command line interface for GoData."""
    pass


main.add_command(server)

if __name__ == "__main__":
    main()
