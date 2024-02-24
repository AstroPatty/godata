"""
The godata project module contains all of themain functionality for interacting with
projects. This includes creating, deleting, and listing projects, as well as adding,
removing, and listing files in projects.
"""


from __future__ import annotations

import atexit
import shutil
from pathlib import Path
from typing import Any

from loguru import logger

from godata.client import client
from godata.files import get_lock
from godata.files import utils as file_utils
from godata.io import find_writer, get_typekey, godataIoException, try_to_read
from godata.utils import sanitize_project_path

__all__ = ["load_project", "list_projects", "create_project", "GodataProjectError"]


class GodataProjectError(Exception):
    pass


class GodataProject:
    """
    A GodataProject object is the main interface for interacting with projects. It
    contains tools for adding, removing, and listing files in the project.

    The GodataProject is responsible for interfacing with the godata server via
    the client module. It is also responsible for delegating file reading and writing
    to the io module.

    You should not create a GodataProject object directly. Instead, use the
    create_projecct or load_project functions to create a new project or load an
    existing one, respectively.
    """

    def __init__(self, collection: str, name: str):
        self.collection = collection
        self.name = name
        self.active = True
        atexit.register(self.__del__)

    def __del__(self):
        if self.active:
            client.drop_project(self.collection, self.name)
            self.active = False

    @sanitize_project_path
    def link(
        self,
        file_path: str | Path,
        project_path: str,
        metadata: dict = {},
        recursive: bool = False,
        overwrite=False,
        verbose=True,
        _force=False,
    ) -> bool:
        """
        Link a pre-existing file or folder to the project. This will not actually move
        any data around. External data that is linked to the project will not
        be deleted from disk under any circumstances, including if the project itself
        is deleted. If you prefer to move the data into the project, use the
        :obj:`godata.project.GodataProject.store` method instead.

        You can link any file you want to the project, regardless of whether it is a
        known file type. If the file is of a known type, subsequent gets will return
        the file as a python object by default.

        Args:
            file_path (str | pathlib.Path): The path to the file or folder to
                link to the project.
            project_path (str): The path in the project to link the file to.
            metadata (dict, optional): A dictionary of metadata to attach to the file.
                This can be used to store additional information about the file.
            recursive (bool, optional): If the file_path is a folder, the link will
                recursively add data in all subfolders to the project as well. Otherwise
                only files within the folder will be included.
            overwrite (bool, optional): If a file or folder already exists at the given
                project path, this will overwrite it. Note that if an overwritten
                file was added to the project using the store method, the file will
                be deleted from disk.
            verbose (bool, optional): If set to True, this will print a message to the
                console indicating the result of the operation.
        """

        fpath = Path(file_path)
        if not fpath.exists() and not _force:
            raise FileNotFoundError(f"Nothing found at {file_path}")
        fpath = fpath.resolve()

        try:
            if fpath.is_dir():
                result = client.link_folder(
                    self.collection, self.name, project_path, str(fpath), recursive
                )
            else:
                result = client.link_file(
                    self.collection,
                    self.name,
                    project_path,
                    str(fpath),
                    metadata=metadata,
                    force=overwrite,
                )
        except client.AlreadyExists:
            raise GodataProjectError(
                f"Something already exists at {project_path}. Use overwrite=True to "
                "overwrite it."
            )
        if verbose:
            print(result["message"])
        file_utils.handle_overwrite(result)
        return True

    @sanitize_project_path
    def store(
        self,
        object: Any,
        project_path: str,
        overwrite=False,
        verbose=True,
        format: str | None = None,
        writer_kwargs: dict = {},
    ) -> bool:
        """
        Stores a given python object or file in godata's internal storage at the
        given project path. If the object is a path, the file will be copied
        into the project's storage.

        Store is different from link in that it will actually move data into
        godata's internal storage, rather than just creating a reference to the file.
        This also means that subsequent removals of the file from the project will
        actually delete the data from disk. This can be done either via an overwrite
        (with link or store), explicit removal with the GodataProject.remove method,
        or by deleting the project itself.

        The "format" and "writer_kwargs" arguments can be used to customize
        the writing process. For example, a pandas dataframe by default will be
        written as a csv file. If you want to write it as a parquet file instead,
        you can pass format=".parquet". The writer_kwargs argument will be passed
        directly to the underlying method, such as pd.DataFrame.to_csv.

        Args:
            object (Any): The object to store in the project. This can be any python
                object, or a path to a file.
            project_path (str): The path in the project to store the file.
            overwrite (bool, optional): If a file or folder already exists at the given
                project path, this will overwrite it. Note that if an overwritten
                file was added to the project using the store method, the file will
                be deleted from disk.
            verbose (bool, optional): If set to True, this will print a message to the
                console indicating the result of the operation.
            format (str, optional): The format to write the file in. If no format is
                given, godata will attempt to infer the format from the object.
                If the format is not known, this will throw an error.
            writer_kwargs (dict, optional): A dictionary of keyword arguments to pass to
                the writer function. This can be used to customize the writing process.

        Returns:
            bool: True if the object was stored successfully.

        Raises:
            GodataProjectError: If the object is not a known type and no format is
                given.
            godataIoException: If the object is a known type, but no writer is found.
        """
        # First, see if the object is a path
        try:
            to_read = Path(object)
        except TypeError:
            to_read = object

        class_name = type(object).__name__
        module_name = type(object).__module__
        type_key = f"{module_name}.{class_name}"
        metadata = {"obj_type": type_key}

        # We link first, because it's better to have be tracking a file that doesn't
        # exist than to have a file that exists but isn't tracked.

        if isinstance(to_read, Path):
            try:
                obj = try_to_read(to_read)  # This can be very slow... Could be improved
                writer_fn, suffix = find_writer(obj, to_read.suffix)

            except godataIoException as e:
                logger.warning(
                    f"Could not find a reader for file {to_read}. The file will still "
                    "be stored, but godata will only be able to return a path."
                    f"Error: {e}"
                )
                storage_path = client.generate_path(
                    self.collection, self.name, project_path
                )
                storage_path = Path(storage_path)
                storage_path = storage_path.with_suffix(to_read.suffix)
                storage_path.parent.mkdir(parents=True, exist_ok=True)

                self.link(
                    storage_path,
                    project_path,
                    overwrite=overwrite,
                    metadata=metadata,
                    _force=True,
                )
                shutil.copy(to_read, storage_path)
                return True
        else:
            obj = object
            writer_fn, suffix = find_writer(object, format)
            if writer_fn is None:
                raise godataIoException(
                    f"No writer found for object of type {type(object)}"
                )

        if suffix is None:
            raise godataIoException(
                f"No writer found for object of type {type(object)}"
            )

        storage_path = client.generate_path(self.collection, self.name, project_path)

        storage_path = Path(storage_path)
        storage_path = storage_path.with_suffix(suffix)
        self.link(
            storage_path,
            project_path,
            overwrite=overwrite,
            _force=True,
            verbose=verbose,
            metadata=metadata,
        )
        storage_path.parent.mkdir(parents=True, exist_ok=True)
        lock = get_lock(storage_path)
        with lock:
            writer_fn(obj, str(storage_path), **writer_kwargs)

        return True

    def get(
        self,
        project_path: str,
        as_path: bool = False,
        load_type: type | None = None,
        reader_kwargs: dict = {},
    ) -> Any:
        """
        Get an object at a given project path. This method will return a python object
        whenever possible. If godata doesn't know how to read in a file of this type,
        it will return a path. The path can also be returned explicitly by passing
        as_path = True.

        The "load_type" and "reader_kwargs" arguments can be used to customize
        the reading process. For example, a csv file is by default read into a pandas
        dataframe, but can also be read as a Polars dataframe by passing
        load_type = polars.DataFrame. Note this is the class itself, not a string.
        The reader_kwargs argument will be passed directly to the underlying read
        method.

        Args:
            project_path (str): The path in the project to get the file from.
            as_path (bool, optional): If set to True, this will return the path to the
                file, rather than the object itself.
            load_type (type, optional): The type to load the file as. This can be any
                python type, such as a pandas DataFrame, or a polars DataFrame.
            reader_kwargs (dict, optional): A dictionary of keyword arguments to pass to
                the reader function. This can be used to customize the reading process.

        Returns:
            Any: The object at the given project path, or the path to the file
                if as_path is set to True or the object cannot be read.

        Raises:
            godataIoException: If the object is a known type, but no reader is found.
            GodataProjectError: If the file does not exist in the project.
        """
        file_info = self.get_metadata(project_path)
        path_str = file_info["real_path"]
        path = Path(path_str)
        if as_path:
            return path
        try:
            if load_type is not None:
                format = get_typekey(load_type)
            else:
                format = file_info.get("obj_type")
            with get_lock(path):
                data = try_to_read(path, format, reader_kwargs)
            return data
        except godataIoException as e:
            logger.info(
                f"Could not find a reader for file {path}. Returning path instead."
                f"Error: {e}"
            )
            return path

    @sanitize_project_path
    def move(
        self,
        src_project_path: str,
        dest_project_path: str,
        overwrite: bool = False,
        verbose: bool = True,
    ) -> bool:
        """
        Move a file or folder from one location in the project to another. This will
        throw an error if the destination already exists. If you want to overwrite the
        destination, set overwrite to True. Note that overwriting a file that was added
        to the project using the store method will delete the file from disk.

        If the data being moved is stored in godata's internal storage, this will
        not necessarily move the data on disk.

        Args:
            src_project_path (str): The path in the project to move the file from.
            dest_project_path (str): The path in the project to move the file to.
            overwrite (bool, optional): If a file or folder already exists at the given
                project path, this will overwrite it. Note that if an overwritten
                file was added to the project using the store method, the file will
                be deleted from disk.
            verbose (bool, optional): If set to True, this will print a message to the
                console indicating the result of the operation.
        Returns:
            bool: True if the file was moved successfully.

        Raises:
            GodataProjectError: If the destination already exists and overwrite is not
                set to True.
        """
        try:
            result = client.move(
                self.collection,
                self.name,
                src_project_path,
                dest_project_path,
                overwrite,
            )
        except client.AlreadyExists:
            raise GodataProjectError(
                f"Something already exists at {dest_project_path}. Use overwrite=True "
                "to overwrite it."
            )
        if verbose:
            print(result["message"])
        return True

    @sanitize_project_path
    def remove(self, project_path: str) -> bool:
        """
        Remove a file or folder from the project. If this file exists outside of the
        project's storage and was added using the link method, this will not delete
        the file from disk. If the file was added to the project using the store method,
        this will delete the file from disk.

        Args:
            project_path (str): The path in the project to remove the file/folder from.

        Returns:
            bool: True if the file or folder was removed successfully.

        Raises:
            GodataProjectError: If the file does not exist in the project.
        """
        try:
            paths = client.remove_file(self.collection, self.name, project_path)
        except client.NotFound:
            raise GodataProjectError(
                f"File or folder {project_path} does not exist in project {self.name}"
            )
        file_utils.handle_removal(paths)
        # will raise an error if it cannot be removed
        return True

    @sanitize_project_path
    def get_metadata(self, project_path: str) -> dict:
        """
        Get the metadata for a given file. This will return a dictionary of metadata
        for the file. If the file does not exist, this will throw an error.
        """
        try:
            file_info = client.get_file(self.collection, self.name, project_path)
        except client.NotFound:
            raise GodataProjectError(
                f"File or folder {project_path} does not exist in project {self.name}"
            )
        return file_info

    @sanitize_project_path
    def list(self, project_path: str | None = None) -> dict[str, str]:
        """
        List the contents of a given project path. This will return a dictionary
        containing the files and folders at the given path. If no path is given,
        this will list the contents of the project root.

        The return will always be of the form

        ``{"files": ["file1", "file2", ...], "folders": ["folder1", "folder2", ...]}``

        Args:
            project_path (str, optional): The path in the project to list.
                If no path is given, this will list the contents of the project root.

        Returns:
            dict[str, list[str]]: A dictionary containing the names of the files and
            folders at the given path.

        Raises:
            GodataProjectError: If the given path does not exist in the project or
                is not a folder.

        """
        return client.list_project_contents(self.collection, self.name, project_path)

    @sanitize_project_path
    def ls(self, project_path: str | None = None) -> None:
        """
        Utility function for listing the contents of a given directory in a
        human-readable format. This is used for the godata CLI, or for working
        in a Jupyter notebook. Example output:

        .. code-block:: python

            >> project = load_project("my_project")
            >> project.ls("folder1")

                my_project/folder1:
                -------------------
                subfolder1/
                subfolder2/
                file1
                file2

        Args:
            project_path (str, optional): The path in the project to list the contents
                of. If no path is given, this will list the contents of the
                project root.

        Raises:
            GodataProjectError: If the given path does not exist in the project or
                is not a folder.
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

        Args:
            project_path (str): The path in the project to check for.

        Returns:
            bool: Whether the given path exists in the project.
        """
        if not project_path:
            return True
        return client.path_exists(self.collection, self.name, project_path)


def has_project(name: str, collection: str = "default") -> bool:
    """
    Check if a project exists in the given collection. If no collection is given, this
    will check if the project exists in the default collection.

    Args:
        name (str): The name of the project to check for
        collection (str, optional): The collection to check for the project in. If no
            collection is given, the project will be checked for in the default
            collection.
    Returns:
        bool: True if the project exists.
    """
    if not has_collection(collection):
        return False
    projects = list_projects(collection, True, False)
    return name in projects


def has_collection(name: str) -> bool:
    """
    Check if a collection exists. This will return True if the collection exists, and
    False if it does not.

    Args:
        name (str): The name of the collection to check for.
    Returns:
        bool: True if the collection exists.
    """
    try:
        collections = list_collections(True, False)
        n_projects = len(list_projects(name, True))
    except GodataProjectError:
        return False
    return name in collections and n_projects > 0


def create_project(
    name: str, collection: str | None = None, storage_location: str | None = None
) -> GodataProject:
    """
    Create a new project in the given collection. If no collection is given, this
    will create a project in the default collection. If the collection does not
    exist, it will be created. You can also specifiy a custom storage location for
    the project. If no storage location is given, the project will be stored in the
    default location.

    Godata supports hidden projects and collections, which are not listed by default.
    This can be useful if your building a tool on top of godata that uses godata
    to store information you don't intend a user to see. To reference a hidden
    hidden project or collection, just prepend the name with a period.

    Args:
        name (str): The name of the project to create
        collection (str, optional): The collection to create the project in. If no
            collection is given, the project will be created in the default collection.
        storage_location (str, optional): A custom storage location for the project.
            If no storage location is given, the project will be stored in the default
            location.

    Returns:
        GodataProject: The newly created project.

    Raises:
        GodataProjectError: If the project already exists in the given collection.
        FileNotFoundError: If the given storage location does not exist.
        NotADirectoryError: If the given storage location is not a directory.
    """

    # Note, the manager will throw an error if the project already exists
    if collection is None:
        collection = "default"

    if has_project(name, collection):
        raise GodataProjectError(
            f"Project {name} already exists in collection {collection}"
        )
    # If a custom storage location exsts, we need to make sure it's valid.
    if storage_location is not None:
        storage_path = Path(storage_location)
        if not storage_path.exists():
            raise FileNotFoundError(
                f"Storage location {storage_location} does not exist"
            )
        if not storage_path.is_dir():
            raise NotADirectoryError(
                f"Storage location {storage_location} is not a directory"
            )
        project_dir = storage_path / f"{collection}.{name}"
        if project_dir.exists():
            raise GodataProjectError(
                f"This project does not exist, but a file or directory with the "
                f"correct name already exists at {project_dir}. Please remove "
                "this file or directory and try again."
            )
        project_dir.mkdir(parents=True, exist_ok=True)
        storage_location = project_dir

    response = client.create_project(
        collection, name, force=True, storage_location=storage_location
    )
    print(response)
    return GodataProject(collection, name)


def load_project(name: str, collection: str = "default") -> GodataProject:
    """
    Load an existing project in the given collection. If no collection is given, this
    will load a project in the default collection. If the project does not exist, this
    will throw an error.

    Args:
        name (str): The name of the project to load
        collection (str, optional): The collection to load the project from. If no
            collection is given, the project will be loaded from the default collection.

    Returns:
        GodataProject: The loaded project.

    Raises:
        GodataProjectError: If the project does not exist in the given collection.
    """

    try:
        _ = client.load_project(collection, name)
    except client.NotFound:
        raise GodataProjectError(
            f"Project {name} does not exist in collection {collection}"
        )
    return GodataProject(collection, name)


def delete_project(name: str, collection: str = "default", force=False) -> bool:
    """
    Remove a project and all data stored in godata's internal storage. This will delete
    any data that was stored in the project using
    :obj:`godata.project.GodataProject.store`, but will not delete any data that was
    linked using :obj:`godata.project.GodataProject.link`.

    Args:
        name (str): The name of the project to delete
        collection (str, optional): The collection to delete the project from. If no
            collection is given, the project will be deleted from the default
            collection.
        force (bool, optional): Required to be set to True to delete the project. This
            is a safety measure to prevent accidental deletion of projects.
    Returns:
        bool: True if the project was deleted successfully.
    Raises:
        GodataProjectError: If the project does not exist in the given collection.

    """
    try:
        client.delete_project(collection, name, force)
    except client.NotFound:
        raise GodataProjectError(
            f"Project {name} does not exist in collection {collection}"
        )
    except client.Forbidden as e:
        raise GodataProjectError(f"{str(e)}")
    return True


def list_projects(
    collection: str = "default", show_hidden: bool = False, display: bool = False
) -> list[str]:
    """
    Return a list of projects in the given collection. If no collection is given, this
    will return a list of projects in the default collection.

    By default, hidden projects are not listed. This function outputs a list of strings
    of the project names. If display is set to True, this function will print the list
    of projects to the console. This is used for the godata CLI, or for working
    in a Jupyter notebook.

    Args:
        collection (str, optional): The collection to list the projects from. If no
            collection is given, the projects will be listed from the default
            collection.
        show_hidden (bool, optional): If set to True, hidden projects will be listed.
        display (bool, optional): If set to True, the list of projects will be printed
            to the console.

    Returns:
        list[str]: A list of project names in the given collection.
    """
    try:
        projects = client.list_projects(collection, show_hidden)
    except client.NotFound:
        raise GodataProjectError(f"Collection {collection} does not exist")
    if display:
        print(f"Projects in collection `{collection or 'default'}`:")
        for p in projects:
            print(f"  {p}")
    return projects


def list_collections(show_hidden=False, display=False) -> list[str]:
    """

    Return a list of collections. By default, hidden collections are not listed. This
    function outputs a list of strings of the collection names. If display is set to
    True, this function will print the list of collections to the console. This is used
    for the godata CLI, or for working in a Jupyter notebook.

    Args:
        show_hidden (bool, optional): If set to True, hidden collections will be listed.
        display (bool, optional): If set to True, the list of collections will be
            printed to the console.

    Returns:
        list[str]: A list of collection names.
    """
    collections = client.list_collections(show_hidden)
    if display:
        print("Collections:")
        for c in collections:
            print(f"  {c}")
    return collections
