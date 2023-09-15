from pathlib import Path
from typing import Any

from loguru import logger

from godata.godata import project

from .io import get_known_writers, godataIoException, try_to_read

manager = project.ProjectManager()
opened_projects = {}

__all__ = ["load_project", "list_projects", "create_project"]


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

    def __init__(self, _project, name):
        self._project = _project
        self.name = name

    def __getattr__(self, name):
        return getattr(self._project, name)

    def remove(self, project_path: str, recursive: bool = False):
        """
        Remove an file/folder at the given path. If a folder contains other
        files/folders, this will throw an error unless rucursive is set to True.
        """
        self._project.remove(project_path, recursive)

    def get(self, project_path: str, as_path=False):
        """
        Get an object at a given project path. This method will return a python object
        whenever possible. If godata doesn't know how to read in a file of this type,
        it will return a path.
        """
        path_str = self._project.get(project_path)
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

    def store(self, object: Any, project_path: str):
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

        if isinstance(to_read, Path):
            try:
                obj = try_to_read(to_read)
                writers = get_known_writers()
                writer_fn, suffix = writers.get(type(obj), (None, None))
                self._project.store(
                    object=obj,
                    project_path=project_path,
                    output_function=writer_fn,
                    suffix=suffix,
                )
            except godataIoException:
                raise godataIoException(
                    "When storing a path, the file at the given"
                    " path must be readable by godata. No reader was fond for file"
                    f" {to_read.suffix}. You can still add it to the project by using"
                    " the `link` method."
                )
        else:
            writers = get_known_writers()
            writer_fn, suffix = writers.get(type(object), (None, None))
            self._project.store(object, project_path, writer_fn, suffix)

    def link(self, file_path: Path, project_path: str):
        """
        Add a file to the project. This will not actually move any data, just create
        a reference to the file.
        """
        fp = str(file_path)
        self._project.add_file(fp, project_path)

    def ls(self, project_path: str = None):
        """
        A basic ls utility for looking at projects. If a path is given, this will
        perform the ls in the folder at the given path. Otherwise, it will perform
        it in the project root.
        """
        contents = self.list(project_path)
        files = contents["files"]
        folders = contents["folders"]
        if not files and not folders:
            print("No files or folders found at path {}".format(project_path))
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

    def list(self, project_path: str = None) -> dict:
        """
        A basic ls utility for looking at projects. If a path is given, this will
        perform the ls in the folder at the given path. Otherwise, it will perform
        it in the project root.
        """
        return self._project.list(project_path)


def has_project(name: str, collection: str = None) -> bool:
    """
    Check if a project exists in the given collection. If no collection is given,
    this will check the default collection.
    """
    projects = manager.list_projects(True, collection)
    return name in projects


def has_collection(name: str) -> bool:
    """
    Check if a collection exists.
    """
    try:
        collections = manager.list_collections(True)
    except project.GodataProjectError:
        return False
    n_projects = len(manager.list_projects(True, name))
    return name in collections and n_projects > 0


def create_project(name, collection=None):
    pname = collection or "default" + "." + name
    # Note, the manager will throw an error if the project already exists
    project = manager.create_project(name, collection)
    opened_projects[pname] = GodataProject(project, name)
    return opened_projects[pname]


def delete_project(name, collection=None):
    """
    Remove a project and all data stored in godata's internal storage. At present,
    this explicitly forces the user the suply True as an argument as a confirmation.
    In the future, we may implement an option to output the internal files somewhere.
    """
    manager.remove_project(name, collection)


def load_project(name, collection=None):
    pname = collection or "default" + "." + name
    if pname in opened_projects:
        return opened_projects[pname]

    project = manager.load_project(name, collection)
    opened_projects[pname] = GodataProject(project, name)
    return opened_projects[pname]


def list_projects(collection=None, show_hidden=False):
    projects = manager.list_projects(show_hidden, collection)
    print(f"Projects in collection `{collection or 'default'}`:")
    for p in projects:
        print(f"  {p}")


def list_collections(show_hidden=False):
    list_collections = manager.list_collections(show_hidden)
    print("Collections:")
    for c in list_collections:
        print(f"  {c}")
