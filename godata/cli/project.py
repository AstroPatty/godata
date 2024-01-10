from pathlib import Path

import click

from godata.project import create_project, load_project


def split_name(name: str) -> tuple:
    split = name.split("/")
    if len(split) == 1:
        project_name = split[0]
        collection = "default"
    elif len(split) == 2:
        project_name = split[1]
        collection = split[0]
    else:
        raise ValueError("Invalid project name.")
    return project_name, collection


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
