from pathlib import Path

import click

from godata.project import create_project, list_collections, list_projects, load_project

from .utils import split_name


@click.command()
@click.argument("project_name")
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
def create(project_name: str, path: Path, force: bool):
    """
    Create a project. The project's storage location will automatically be created.
    """
    name, collection = split_name(project_name)
    create_project(name, collection, path)


@click.command()
@click.argument("project_name")
@click.argument("project_path", type=str)
@click.argument("path", type=Path)
@click.option(
    "--recursive",
    "-r",
    is_flag=True,
    help="Only applies if linking a folder. If not set, this command will only "
    "link files in this particular folder and not any subfolders.",
)
@click.option(
    "--overwrite",
    "-o",
    is_flag=True,
    help="Force creation of the project even if something already exists.",
)
def link(
    project_name: str, project_path: str, path: Path, recursive: bool, overwrite: bool
):
    """
    Link a file or folder into a project.
    """
    name, collection = split_name(project_name)
    p = load_project(name, collection)
    p.link(path, project_path, recursive=recursive, overwrite=overwrite)


@click.command()
@click.argument("project_name", type=str)
@click.argument("project_path", type=str, required=False)
def ls(project_name: str, project_path: str = None):
    """
    List the contents of a project.
    """
    name, collection = split_name(project_name)
    p = load_project(name, collection)
    p.ls(project_path)


@click.command()
@click.argument("collection_name", type=str, required=False)
@click.option(
    "--hidden",
    "-h",
    is_flag=True,
    help="Include hidden projects or collections in the list.",
)
def list(collection_name: str = None, hidden: bool = False):
    """
    List the known collections, or the projects in a given collection
    """
    if collection_name is None:
        _ = list_collections(hidden, True)
    else:
        _ = list_projects(collection_name, hidden, True)


@click.command()
@click.argument("project_name", type=str)
@click.argument("project_path", type=str)
def get(project_name: str, project_path: str):
    """
    Get the real location of a file in a project.
    """
    name, collection = split_name(project_name)
    p = load_project(name, collection)
    path = p.get(project_path, as_path=True)
    click.echo(path)
