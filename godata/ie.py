"""
Utility functions for importing and exporting projects.

We do this by creating a new temporary project, then recursively storing all
files in the project being exported. This creates an exact copy of the original project,
but with all files stored (and linked!) in the temporary project's storage. Then
we just zip it up and we're done.

The only complications is that files in the tree store their full path, and this path
will obviously change when the projected is imported. We need to either fix that or 
change the server to only store relative paths when the file is internal.

"""
import zipfile
from pathlib import Path

from godata.client.client import export_tree, import_tree
from godata.project import (
    GodataProject,
    create_project,
    delete_project,
    has_project,
    load_project,
)


def export_project(
    project_name: str,
    collection_name: str = "default",
    output_location=None,
    verbose=False,
) -> Path:
    if output_location and not output_location.is_dir():
        raise ValueError("Output location must be a directory")
    if not output_location:
        output_location = Path.cwd()

    source_project = load_project(project_name, collection_name)
    target_project = create_project(
        project_name, ".temp", storage_location=output_location
    )
    print("Duplicating project for export")
    export_helper(source_project, target_project)
    # Now, zip up the temporary project
    zip_path = output_location / f"{project_name}.zip"
    expected_location = output_location / f".temp.{project_name}"
    if not expected_location.exists():
        raise RuntimeError("Something went wrong with the export")
    print("Exporting project file tree...")
    export_tree(collection_name, project_name, expected_location)
    print("Zipping up project...")
    with zipfile.ZipFile(zip_path, "w") as zip_file:
        # Recursively add all files in the temp project
        for f in expected_location.glob("**/*"):
            zip_file.write(f, f.relative_to(expected_location))
    # Clean up the temp project
    del target_project
    delete_project(project_name, ".temp", True)
    return zip_path


def export_helper(
    source_project: GodataProject,
    destination_project: GodataProject,
    project_path: str = None,
) -> None:
    print(f"Working on {project_path}...")
    folder_contents = source_project.list(project_path)
    files = folder_contents["files"]
    folders = folder_contents["folders"]
    if not project_path:
        project_path = ""
    for f in files:
        file_project_path = f"{project_path}/{f}"
        file_real_path = source_project.get(file_project_path, as_path=True)
        destination_project.store(file_real_path, file_project_path, verbose=False)
    for f in folders:
        export_helper(source_project, destination_project, f"{project_path}/{f}")


def import_project(
    zip_path: Path,
    project_name: str = None,
    collection_name: str = "default",
    output_location=None,
    verbose=False,
) -> None:
    if not zip_path.exists():
        raise ValueError("Zip file does not exist")
    if not project_name:
        project_name = zip_path.stem

    if has_project(project_name, collection_name):
        raise ValueError(
            f"Project {project_name} already exists in collection {collection_name}"
        )
    if not output_location:
        output_location = Path.cwd()
    if not output_location.is_dir():
        raise ValueError("Output location must be a directory")

    output_dir = output_location / f"{collection_name}.{project_name}"
    if output_dir.exists():
        raise ValueError(f"Output directory {output_dir} already exists")

    # unzip the project into the output location
    with zipfile.ZipFile(zip_path, "r") as zip_file:
        zip_file.extractall(output_dir)

    # Now, unzip the project
    # Now, import the project
    res = import_tree(collection_name, project_name, output_dir)
    if verbose:
        print(res)
