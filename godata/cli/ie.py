from pathlib import Path

import click

from godata import ie

from .utils import split_name


@click.command(name="import")
@click.argument("path", type=Path)
@click.argument("project_name", type=str)
@click.option("--storage", "-s", type=Path, default=None)
def import_project(path: Path, project_name: str, storage: Path):
    """
    Import a project from a directory.
    """
    name, collection = split_name(project_name)
    ie.import_project(path, name, collection, storage, verbose=True)


@click.command(name="export")
@click.argument("project_name", type=str)
@click.option("--output", "-o", type=Path, default=None)
def export_project(project_name: str, output: Path):
    """
    Export a project to a directory.
    """
    name, collection = split_name(project_name)
    output_path = ie.export_project(name, collection, output, verbose=True)
    click.echo(f"Project exported to {output_path}")
