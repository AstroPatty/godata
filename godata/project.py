from __future__ import annotations

import asyncio
from pathlib import Path
from typing import Any

from loguru import logger

from godata import client
from godata.files import utils as file_utils
from godata.io import get_known_writers, godataIoException, try_to_read
from godata.utils import sanitize_project_path

__all__ = ["load_project", "list_projects", "create_project", "GodataProjectError"]


class GodataProjectError(Exception):
    pass


class GodataProject:
    """
    This is at thin wrapper class for the associated Project struct in the rust library.
    In general, this class just calls the underlying rust methods. However, it does have
    to provide additional behavior in particular cases. For example, storing a python
    object requires a function that knows how to write the given object to a file, which
    will most likely be in python.

    Note that in most cases error handling is actually done by the rust library, so in
    almost all cases expect that an exception encountered while using this class is
    coming from there.

    This class also provides docstrings for the underlying methods, such that all
    user-facing documentation can be done with sphinx.
    """

    def __init__(self, collection, name) -> GodataProject:
        self.collection = collection
        self.name = name

    @sanitize_project_path
    def remove(self, project_path: str) -> bool:
        """
        Remove an file/folder at the given path. If a folder contains other
        files/folders, this will throw an error unless rucursive is set to True.
        """
        asyncio.run(client.remove_file(self.collection, self.name, project_path))
        # will raise an error if it cannot be removed
        return True

    @sanitize_project_path
    def get(self, project_path: str, as_path=False) -> Any:
        """
        Get an object at a given project path. This method will return a python object
        whenever possible. If godata doesn't know how to read in a file of this type,
        it will return a path. The path can also be returned explicitly by passing
        as_path = True.
        """
        path_str = asyncio.run(
            client.get_file(self.collection, self.name, project_path)
        )
        path = Path(path_str)
        if as_path:
            return path
        try:
            data = try_to_read(path)
            return data
        except godataIoException:
            logger.info(
                f"Could not find a reader for file {path}. Returning path instead."
            )
            return path

    @sanitize_project_path
    def store(self, object: Any, project_path: str, overwrite=True) -> bool:
        """
        Stores a given python object in godata's internal storage at the given path.
        Not having a writer defined in godata's python io module is not necessarily
        a failure case. Some objects can be converted easily into rust objects (or)
        actually ARE rust objects under the hood, and will be handled by the rust
        library. If a writer is not found by either python pyestor rust, this will throw
        an error.

        However one thing to note is that if a writer is found in python, it will
        always be used over a rust writer.
        """
        # First, see if the object is a path
        try:
            to_read = Path(object)
        except TypeError:
            to_read = object

        # We link first, because it's better to have be tracking a file that doesn't
        # exist than to have a file that exists but isn't tracked.

        if isinstance(to_read, Path):
            try:
                obj = try_to_read(to_read)  # This can be very slow... Could be improved
                writers = get_known_writers()
                writer_fn, suffix = writers.get(type(obj), (None, None))

            except godataIoException:
                raise godataIoException(
                    "When storing a path, the file at the given"
                    " path must be readable by godata. No reader was fond for file"
                    f" {to_read.suffix}. You can still add it to the project by using"
                    " the `link` method."
                )
        else:
            obj = object
            writers = get_known_writers()
            writer_fn, suffix = writers.get(type(object), (None, None))
            if writer_fn is None:
                self.remove(project_path)
                raise godataIoException(
                    f"No writer found for object of type {type(object)}"
                )

        if suffix is None:
            raise godataIoException(
                f"No writer found for object of type {type(object)}"
            )

        storage_path = asyncio.run(
            client.generate_path(self.collection, self.name, project_path)
        )
        storage_path = Path(storage_path)
        storage_path = storage_path.with_suffix("." + suffix)
        storage_path.parent.mkdir(parents=True, exist_ok=True)
        self.link(storage_path, project_path, overwrite=overwrite, _force=True)
        writer_fn(obj, storage_path)

        return True

    @sanitize_project_path
    def link(
        self,
        file_path: str,
        project_path: str,
        recursive: bool = False,
        overwrite=False,
        _force=False,
    ) -> bool:
        """
        Add a file to the project. This will not actually move any data, just create
        a reference to the file.
        """

        fpath = Path(file_path)
        if not fpath.exists() and not _force:
            raise FileNotFoundError(f"Nothing found at {file_path}")
        fpath = fpath.resolve()

        if fpath.is_dir():
            result = asyncio.run(
                client.link_folder(
                    self.collection, self.name, project_path, str(fpath), recursive
                )
            )
        else:
            result = asyncio.run(
                client.link_file(
                    self.collection, self.name, project_path, str(fpath), overwrite
                )
            )
        print(result["message"])
        file_utils.handle_overwrite(result)
        return True

    def ls(self, project_path: str = None) -> None:
        """
        A basic ls utility for looking at projects. If a path is given, this will
        perform the ls in the folder at the given path. Otherwise, it will perform
        it in the project root.

        Just prints
        """
        contents = self.list(project_path)
        files = contents["files"]
        folders = contents["folders"]
        if not files and not folders:
            if project_path is None:
                print(f"No files or folders found in project '{self.name}'")
            else:
                print("No files or folders found at path '{}'".format(project_path))
            return

        if not project_path:
            header_string = f"Project `{self.name}` root:"
        else:
            header_string = f"{self.name}/{project_path}:"
        print(header_string)
        print("-" * len(header_string))
        for folder in folders:
            print(f"  {folder}/")
        for file in files:
            print(f"  {file}")

    @sanitize_project_path
    def has_path(self, project_path: str) -> bool:
        """
        Check if a given path exists in the project.
        """
        if not project_path:
            return True
        return asyncio.run(client.path_exists(self.collection, self.name, project_path))

    @sanitize_project_path
    def list(self, project_path: str = None) -> dict[str, str]:
        """
        A basic ls utility for looking at projects. If a path is given, this will
        perform the ls in the folder at the given path. Otherwise, it will perform
        it in the project root.
        """
        return asyncio.run(
            client.list_project_contents(self.collection, self.name, project_path)
        )


def has_project(name: str, collection: str = "default") -> bool:
    """
    Check if a project exists in the given collection. If no collection is given,
    this will check the default collection.
    """
    projects = list_projects(collection, True, False)
    return name in projects


def has_collection(name: str) -> bool:
    """
    Check if a collection exists.
    """
    try:
        collections = list_collections(True, False)
        n_projects = len(list_projects(True, name))
    except GodataProjectError:
        return False
    return name in collections and n_projects > 0


def create_project(
    name: str, collection: str = "default", storage_location: str = None
) -> GodataProject:
    """
    Create a new project in the given collection. If no collection is given, this
    will create a project in the default collection. If the collection does not
    exist, it will be created.

    """

    # Note, the manager will throw an error if the project already exists
    try:
        response = asyncio.run(
            client.create_project(
                collection, name, force=True, storage_location=storage_location
            )
        )
    except client.AlreadyExists:
        raise GodataProjectError(
            f"Project {name} already exists in collection {collection}"
        )
    print(response)
    return GodataProject(collection, name)


def delete_project(name, collection="default", force=False) -> bool:
    """
    Remove a project and all data stored in godata's internal storage. At present,
    this explicitly forces the user the suply True as an argument as a confirmation.
    In the future, we may implement an option to output the internal files somewhere.
    """
    asyncio.run(client.delete_project(collection, name, force))
    return True


def load_project(name, collection="default") -> GodataProject:
    known_projects = list_projects(collection, True, False)
    if name not in known_projects:
        raise GodataProjectError(f"Project {name} not found in collection {collection}")
    return GodataProject(collection, name)


def list_projects(collection="default", show_hidden=False, display=True) -> list[str]:
    projects = asyncio.run(client.list_projects(collection, show_hidden))
    if display:
        print(f"Projects in collection `{collection or 'default'}`:")
        for p in projects:
            print(f"  {p}")
    return projects


def list_collections(show_hidden=False, display=True) -> list[str]:
    collections = asyncio.run(client.list_collections(show_hidden))
    if display:
        print("Collections:")
        for c in collections:
            print(f"  {c}")
    return collections
