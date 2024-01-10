from pathlib import Path

import click

from godata.project import create_project


@click.group()
def project():
    pass


@project.command()
@click.argument("name")
@click.option(
    "--path",
    "-p",
    help="Path to the project's storage location. If not provided,"
    "the project will be created in the current directory.",
    default=lambda: Path.cwd(),
    type=Path,
)
@click.option(
    "--force",
    "-f",
    is_flag=True,
    help="Force creation of the project even if the path already exists.",
)
def create(name: str, path: Path, force: bool):
    """
    Create a project. The project's storage location will automatically be created.
    """
    split = name.split("/")
    if len(split) == 1:
        project_name = split[0]
        collection = None
    elif len(split) == 2:
        project_name = split[1]
        collection = split[0]
    else:
        raise ValueError("Invalid project name.")

    create_project(project_name, collection, path)
